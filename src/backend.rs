// TODO: Make these modules private
pub mod config;
pub mod farmer;
mod networking;
pub mod node;
mod utils;

use crate::backend::config::{Config, ConfigError, RawConfig};
use crate::backend::farmer::maybe_node_client::MaybeNodeRpcClient;
use crate::backend::farmer::{
    DiskFarm, Farmer, FarmerNotification, FarmerOptions, InitialFarmState,
};
use crate::backend::networking::{create_network, NetworkOptions};
use crate::backend::node::{
    dsn_bootstrap_nodes, BlockImported, ChainInfo, ChainSpec, ConsensusNode,
    ConsensusNodeCreationError, SyncState, GENESIS_HASH,
};
use backoff::ExponentialBackoff;
use future::FutureExt;
use futures::channel::mpsc;
use futures::{future, select, SinkExt, StreamExt};
use parking_lot::Mutex;
use sc_subspace_chain_specs::GEMINI_3H_CHAIN_SPEC;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::pin::pin;
use std::sync::{Arc, Weak};
use std::time::Duration;
use subspace_core_primitives::crypto::kzg::{embedded_kzg_settings, Kzg};
use subspace_core_primitives::{BlockNumber, Piece, PieceIndex, PublicKey};
use subspace_farmer::farmer_cache::{FarmerCache, FarmerCacheWorker};
use subspace_farmer::single_disk_farm::SingleDiskFarm;
use subspace_farmer::utils::farmer_piece_getter::{
    DsnCacheRetryPolicy, FarmerPieceGetter, WeakFarmerPieceGetter,
};
use subspace_farmer::utils::piece_validator::SegmentCommitmentPieceValidator;
use subspace_farmer::utils::plotted_pieces::PlottedPieces;
use subspace_farmer::utils::run_future_in_dedicated_thread;
use subspace_farmer_components::PieceGetter;
use subspace_networking::libp2p::identity::ed25519::{Keypair, SecretKey};
use subspace_networking::libp2p::multiaddr::Protocol;
use subspace_networking::libp2p::Multiaddr;
use subspace_networking::utils::piece_provider::PieceProvider;
use subspace_networking::{Node, NodeRunner};
use subspace_runtime_primitives::Balance;
use subspace_service::sync_from_dsn::DsnSyncPieceGetter;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Handle;
use tokio::sync::Semaphore;
use tracing::{error, info_span, warn, Instrument};

/// Get piece retry attempts number.
const PIECE_GETTER_MAX_RETRIES: u16 = 7;
/// Global limit on combined piece getter, a nice number that should result in enough pieces
/// downloading successfully during DSN sync
const PIECE_GETTER_MAX_CONCURRENCY: usize = 512;
/// Defines initial duration between get_piece calls.
const GET_PIECE_INITIAL_INTERVAL: Duration = Duration::from_secs(5);
/// Defines max duration between get_piece calls.
const GET_PIECE_MAX_INTERVAL: Duration = Duration::from_secs(40);

#[derive(Debug, Clone)]
struct PieceGetterWrapper {
    farmer_piece_getter:
        FarmerPieceGetter<SegmentCommitmentPieceValidator<MaybeNodeRpcClient>, MaybeNodeRpcClient>,
    semaphore: Arc<Semaphore>,
}

#[async_trait::async_trait]
impl DsnSyncPieceGetter for PieceGetterWrapper {
    async fn get_piece(
        &self,
        piece_index: PieceIndex,
    ) -> Result<Option<Piece>, Box<dyn Error + Send + Sync + 'static>> {
        let _permit = self.semaphore.acquire().await;
        Ok(self.farmer_piece_getter.get_piece_fast(piece_index).await)
    }
}

#[async_trait::async_trait]
impl PieceGetter for PieceGetterWrapper {
    async fn get_piece(
        &self,
        piece_index: PieceIndex,
    ) -> Result<Option<Piece>, Box<dyn Error + Send + Sync + 'static>> {
        let _permit = self.semaphore.acquire().await;
        self.farmer_piece_getter.get_piece(piece_index).await
    }
}

impl PieceGetterWrapper {
    fn new(
        farmer_piece_getter: FarmerPieceGetter<
            SegmentCommitmentPieceValidator<MaybeNodeRpcClient>,
            MaybeNodeRpcClient,
        >,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(PIECE_GETTER_MAX_CONCURRENCY));
        Self {
            farmer_piece_getter,
            semaphore,
        }
    }

    fn downgrade(&self) -> WeakPieceGetterWrapper {
        WeakPieceGetterWrapper {
            farmer_piece_getter: self.farmer_piece_getter.downgrade(),
            semaphore: Arc::downgrade(&self.semaphore),
        }
    }
}

#[derive(Debug, Clone)]
struct WeakPieceGetterWrapper {
    farmer_piece_getter: WeakFarmerPieceGetter<
        SegmentCommitmentPieceValidator<MaybeNodeRpcClient>,
        MaybeNodeRpcClient,
    >,
    semaphore: Weak<Semaphore>,
}

#[async_trait::async_trait]
impl PieceGetter for WeakPieceGetterWrapper {
    async fn get_piece(
        &self,
        piece_index: PieceIndex,
    ) -> Result<Option<Piece>, Box<dyn Error + Send + Sync + 'static>> {
        let Some(semaphore) = self.semaphore.upgrade() else {
            return Ok(None);
        };
        let _permit = semaphore.acquire().await;
        self.farmer_piece_getter.get_piece(piece_index).await
    }
}

/// Major steps in application loading progress
#[derive(Debug, Clone)]
pub enum LoadingStep {
    LoadingConfiguration,
    ReadingConfiguration,
    ConfigurationReadSuccessfully {
        /// Whether configuration exists, `false` on the first start
        configuration_exists: bool,
    },
    CheckingConfiguration,
    ConfigurationIsValid,
    DecodingChainSpecification,
    DecodedChainSpecificationSuccessfully,
    CheckingNodePath,
    CreatingNodePath,
    NodePathReady,
    PreparingNetworkingStack,
    ReadingNetworkKeypair,
    GeneratingNetworkKeypair,
    WritingNetworkKeypair,
    InstantiatingNetworkingStack,
    NetworkingStackCreatedSuccessfully,
    CreatingConsensusNode,
    ConsensusNodeCreatedSuccessfully,
    CreatingFarmer,
    FarmerCreatedSuccessfully,
    WipingFarm {
        farm_index: u8,
        path: PathBuf,
    },
    WipingNode {
        path: PathBuf,
    },
}

#[derive(Debug)]
enum LoadedConsensusChainNode {
    Compatible(ConsensusNode),
    Incompatible { compatible_chain: String },
}

#[derive(Debug, Clone)]
pub enum NodeNotification {
    SyncStateUpdate(SyncState),
    BlockImported(BlockImported),
}

/// Notification messages send from backend about its operation
#[derive(Debug)]
pub enum BackendNotification {
    /// Application loading progress
    Loading {
        /// Major loading step
        step: LoadingStep,
        // TODO: Set this to non-zero where it is used
        /// Progress in %: 0.0..=100.0
        progress: f32,
    },
    IncompatibleChain {
        raw_config: RawConfig,
        compatible_chain: String,
    },
    NotConfigured,
    // TODO: Indicate what is invalid so that UI can render it properly
    ConfigurationIsInvalid {
        config: RawConfig,
        error: ConfigError,
    },
    ConfigSaveResult(anyhow::Result<()>),
    Running {
        config: Config,
        raw_config: RawConfig,
        best_block_number: BlockNumber,
        reward_address_balance: Balance,
        initial_farm_states: Vec<InitialFarmState>,
        farm_during_initial_plotting: bool,
        chain_info: ChainInfo,
    },
    Node(NodeNotification),
    Farmer(FarmerNotification),
    Stopped {
        /// Error in case stopped due to error
        error: Option<anyhow::Error>,
    },
    IrrecoverableError {
        /// Error that happened
        error: anyhow::Error,
    },
}

/// Control action messages sent to backend to control its behavior
#[derive(Debug)]
pub enum BackendAction {
    /// Config was created or updated
    NewConfig { raw_config: RawConfig },
}

struct LoadedBackend {
    config: Config,
    raw_config: RawConfig,
    config_file_path: PathBuf,
    consensus_node: ConsensusNode,
    farmer: Farmer,
    node_runner: NodeRunner<FarmerCache>,
}

enum BackendLoadingResult {
    Success(LoadedBackend),
    IncompatibleChain {
        raw_config: RawConfig,
        compatible_chain: String,
    },
}

// NOTE: this is an async function, but it might do blocking operations and should be running on a
// dedicated CPU core
pub async fn create(
    mut backend_action_receiver: mpsc::Receiver<BackendAction>,
    mut notifications_sender: mpsc::Sender<BackendNotification>,
) {
    let loading_result = try {
        'load: loop {
            if let Some(backend_loaded) = load(&mut notifications_sender).await? {
                break backend_loaded;
            }

            if let Err(error) = notifications_sender
                .send(BackendNotification::NotConfigured)
                .await
            {
                error!(%error, "Failed to send not configured notification");
                return;
            }

            // Remove suppression once we have more actions for backend
            #[allow(clippy::never_loop)]
            while let Some(backend_action) = backend_action_receiver.next().await {
                match backend_action {
                    BackendAction::NewConfig { raw_config } => {
                        if let Err(error) = Config::try_from_raw_config(&raw_config).await {
                            notifications_sender
                                .send(BackendNotification::ConfigurationIsInvalid {
                                    config: raw_config.clone(),
                                    error,
                                })
                                .await?;
                        }

                        let config_file_path = RawConfig::default_path().await?;
                        raw_config
                            .write_to_path(&config_file_path)
                            .await
                            .map_err(|error| {
                                anyhow::anyhow!(
                                    "Failed to write config to \"{}\": {}",
                                    config_file_path.display(),
                                    error
                                )
                            })?;

                        // Try to load config and start again
                        continue 'load;
                    }
                }
            }

            return;
        }
    };

    let loaded_backend = match loading_result {
        Ok(BackendLoadingResult::Success(loaded_backend)) => {
            // Loaded successfully
            loaded_backend
        }
        Ok(BackendLoadingResult::IncompatibleChain {
            raw_config,
            compatible_chain,
        }) => {
            if let Err(error) = notifications_sender
                .send(BackendNotification::IncompatibleChain {
                    raw_config,
                    compatible_chain,
                })
                .await
            {
                error!(%error, "Failed to send incompatible chain notification");
            }
            return;
        }
        Err(error) => {
            if let Err(error) = notifications_sender
                .send(BackendNotification::IrrecoverableError { error })
                .await
            {
                error!(%error, "Failed to send error notification");
            }
            return;
        }
    };

    let run_fut = run(
        loaded_backend,
        &mut backend_action_receiver,
        &mut notifications_sender,
    );
    if let Err(error) = run_fut.await {
        if let Err(error) = notifications_sender
            .send(BackendNotification::IrrecoverableError { error })
            .await
        {
            error!(%error, "Failed to send run error notification");
        }
    }
}

async fn load(
    notifications_sender: &mut mpsc::Sender<BackendNotification>,
) -> anyhow::Result<Option<BackendLoadingResult>> {
    let (config_file_path, Some(raw_config)) = load_configuration(notifications_sender).await?
    else {
        return Ok(None);
    };

    let Some(config) = check_configuration(&raw_config, notifications_sender).await? else {
        return Ok(None);
    };

    let chain_spec = load_chain_specification(notifications_sender).await?;

    preparing_node_path(&config.node_path, notifications_sender).await?;

    let (
        maybe_node_client,
        node,
        node_runner,
        network_keypair,
        plotted_pieces,
        farmer_cache,
        farmer_cache_worker,
    ) = create_networking_stack(
        &config,
        GENESIS_HASH.to_string(),
        &chain_spec,
        notifications_sender,
    )
    .await?;

    let kzg = Kzg::new(embedded_kzg_settings());
    let piece_provider = PieceProvider::new(
        node.clone(),
        Some(SegmentCommitmentPieceValidator::new(
            node.clone(),
            maybe_node_client.clone(),
            kzg.clone(),
        )),
    );

    let piece_getter = PieceGetterWrapper::new(FarmerPieceGetter::new(
        piece_provider,
        farmer_cache.clone(),
        maybe_node_client.clone(),
        Arc::clone(&plotted_pieces),
        DsnCacheRetryPolicy {
            max_retries: PIECE_GETTER_MAX_RETRIES,
            backoff: ExponentialBackoff {
                initial_interval: GET_PIECE_INITIAL_INTERVAL,
                max_interval: GET_PIECE_MAX_INTERVAL,
                // Try until we get a valid piece
                max_elapsed_time: None,
                multiplier: 1.75,
                ..ExponentialBackoff::default()
            },
        },
    ));

    let create_consensus_node_fut = create_consensus_node(
        &network_keypair,
        config.node_path.clone(),
        config.network.substrate_port,
        chain_spec,
        Arc::new(piece_getter.clone()),
        node.clone(),
        &maybe_node_client,
        notifications_sender,
    );
    let consensus_node = match create_consensus_node_fut.await? {
        LoadedConsensusChainNode::Compatible(consensus_node) => consensus_node,
        LoadedConsensusChainNode::Incompatible { compatible_chain } => {
            return Ok(Some(BackendLoadingResult::IncompatibleChain {
                raw_config,
                compatible_chain,
            }));
        }
    };

    let farmer = create_farmer(
        config.reward_address,
        config.farms.clone(),
        plotted_pieces,
        farmer_cache,
        farmer_cache_worker,
        maybe_node_client,
        kzg,
        piece_getter,
        notifications_sender,
    )
    .await?;

    Ok(Some(BackendLoadingResult::Success(LoadedBackend {
        config,
        raw_config,
        config_file_path,
        consensus_node,
        farmer,
        node_runner,
    })))
}

async fn run(
    loaded_backend: LoadedBackend,
    backend_action_receiver: &mut mpsc::Receiver<BackendAction>,
    notifications_sender: &mut mpsc::Sender<BackendNotification>,
) -> anyhow::Result<()> {
    let LoadedBackend {
        config,
        raw_config,
        config_file_path,
        consensus_node,
        farmer,
        mut node_runner,
    } = loaded_backend;
    let networking_fut = run_future_in_dedicated_thread(
        {
            let span = info_span!("Network");
            let future = async move { node_runner.run().await }.instrument(span);

            move || future
        },
        "networking".to_string(),
    )?;

    notifications_sender
        .send(BackendNotification::Running {
            config: config.clone(),
            raw_config,
            best_block_number: consensus_node.best_block_number(),
            reward_address_balance: consensus_node.account_balance(&config.reward_address),
            initial_farm_states: farmer.initial_farm_states().to_vec(),
            farm_during_initial_plotting: farmer.farm_during_initial_plotting(),
            chain_info: consensus_node.chain_info().clone(),
        })
        .await?;

    let _on_sync_state_change_handler_id = consensus_node.on_sync_state_change({
        let notifications_sender = notifications_sender.clone();

        Arc::new(move |&sync_state| {
            let notification = NodeNotification::SyncStateUpdate(sync_state);

            let mut notifications_sender = notifications_sender.clone();

            if let Err(error) = notifications_sender
                .try_send(BackendNotification::Node(notification))
                .or_else(|error| {
                    tokio::task::block_in_place(|| {
                        Handle::current().block_on(notifications_sender.send(error.into_inner()))
                    })
                })
            {
                warn!(%error, "Failed to send sync state backend notification");
            }
        })
    });
    let _on_imported_block_handler_id = consensus_node.on_block_imported({
        let notifications_sender = notifications_sender.clone();
        // let reward_address_storage_key = account_storage_key(&config.reward_address);

        Arc::new(move |&block_imported| {
            let notification = NodeNotification::BlockImported(block_imported);

            let mut notifications_sender = notifications_sender.clone();

            if let Err(error) = notifications_sender
                .try_send(BackendNotification::Node(notification))
                .or_else(|error| {
                    tokio::task::block_in_place(|| {
                        Handle::current().block_on(notifications_sender.send(error.into_inner()))
                    })
                })
            {
                warn!(%error, "Failed to send imported block backend notification");
            }
        })
    });
    let _on_farmer_notification_handler_id = farmer.on_notification({
        let notifications_sender = notifications_sender.clone();

        Arc::new(move |notification| {
            let mut notifications_sender = notifications_sender.clone();

            if let Err(error) = notifications_sender
                .try_send(BackendNotification::Farmer(notification.clone()))
                .or_else(|error| {
                    tokio::task::block_in_place(|| {
                        Handle::current().block_on(notifications_sender.send(error.into_inner()))
                    })
                })
            {
                warn!(%error, "Failed to send farmer backend notification");
            }
        })
    });

    // Order is important here, we want to destroy dependents first and only then corresponding
    // dependencies to avoid unnecessary errors and warnings in logs
    let networking_fut = networking_fut;
    let consensus_node_fut = consensus_node.run(&config.reward_address);
    let farmer_fut = farmer.run();
    let process_backend_actions_fut = {
        let mut notifications_sender = notifications_sender.clone();

        async move {
            process_backend_actions(
                &config_file_path,
                backend_action_receiver,
                &mut notifications_sender,
            )
            .await
        }
    };

    let networking_fut = pin!(networking_fut);
    let consensus_node_fut = pin!(consensus_node_fut);
    let farmer_fut = pin!(farmer_fut);
    let process_backend_actions_fut = pin!(process_backend_actions_fut);

    let result: anyhow::Result<()> = select! {
        result = networking_fut.fuse() => {
            result.map_err(|error| anyhow::anyhow!("Networking exited: {error}"))
        }
        result = consensus_node_fut.fuse() => {
            result.map_err(|error| anyhow::anyhow!("Consensus node exited: {error}"))
        }
        result = farmer_fut.fuse() => {
            result.map_err(|error| anyhow::anyhow!("Farm exited: {error}"))
        }
        _ = process_backend_actions_fut.fuse() => {
            Ok(())
        }
    };

    notifications_sender
        .send(BackendNotification::Stopped {
            error: result.err(),
        })
        .await?;

    Ok(())
}

async fn load_configuration(
    notifications_sender: &mut mpsc::Sender<BackendNotification>,
) -> anyhow::Result<(PathBuf, Option<RawConfig>)> {
    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::LoadingConfiguration,
            progress: 0.0,
        })
        .await?;

    let config_file_path = RawConfig::default_path().await?;

    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::ReadingConfiguration,
            progress: 0.0,
        })
        .await?;

    // TODO: Make configuration errors recoverable
    let maybe_config = RawConfig::read_from_path(&config_file_path).await?;

    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::ConfigurationReadSuccessfully {
                configuration_exists: maybe_config.is_some(),
            },
            progress: 0.0,
        })
        .await?;

    Ok((config_file_path, maybe_config))
}

/// Returns `Ok(None)` if configuration failed validation
async fn check_configuration(
    config: &RawConfig,
    notifications_sender: &mut mpsc::Sender<BackendNotification>,
) -> anyhow::Result<Option<Config>> {
    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::CheckingConfiguration,
            progress: 0.0,
        })
        .await?;

    match Config::try_from_raw_config(config).await {
        Ok(config) => {
            notifications_sender
                .send(BackendNotification::Loading {
                    step: LoadingStep::ConfigurationIsValid,
                    progress: 0.0,
                })
                .await?;
            Ok(Some(config))
        }
        Err(error) => {
            notifications_sender
                .send(BackendNotification::ConfigurationIsInvalid {
                    config: config.clone(),
                    error,
                })
                .await?;

            Ok(None)
        }
    }
}

async fn load_chain_specification(
    notifications_sender: &mut mpsc::Sender<BackendNotification>,
) -> anyhow::Result<ChainSpec> {
    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::DecodingChainSpecification,
            progress: 0.0,
        })
        .await?;

    let chain_spec = node::load_chain_specification(GEMINI_3H_CHAIN_SPEC.as_bytes())
        .map_err(|error| anyhow::anyhow!(error))?;

    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::DecodedChainSpecificationSuccessfully,
            progress: 0.0,
        })
        .await?;

    Ok(chain_spec)
}

async fn preparing_node_path(
    node_path: &Path,
    notifications_sender: &mut mpsc::Sender<BackendNotification>,
) -> anyhow::Result<()> {
    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::CheckingNodePath,
            progress: 0.0,
        })
        .await?;

    let node_path_exists = fs::try_exists(node_path).await.map_err(|error| {
        anyhow::anyhow!(
            "Node path \"{}\" doesn't exist and can't be created: {error:?}",
            node_path.display()
        )
    })?;

    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::CreatingNodePath,
            progress: 0.0,
        })
        .await?;

    if !node_path_exists {
        fs::create_dir(node_path).await.map_err(|error| {
            anyhow::anyhow!(
                "Node path \"{}\" didn't exist and creation failed: {error:?}",
                node_path.display()
            )
        })?;
    }

    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::NodePathReady,
            progress: 0.0,
        })
        .await?;

    Ok(())
}

async fn create_networking_stack(
    config: &Config,
    protocol_prefix: String,
    chain_spec: &ChainSpec,
    notifications_sender: &mut mpsc::Sender<BackendNotification>,
) -> anyhow::Result<(
    MaybeNodeRpcClient,
    Node,
    NodeRunner<FarmerCache>,
    Keypair,
    Arc<Mutex<Option<PlottedPieces>>>,
    FarmerCache,
    FarmerCacheWorker<MaybeNodeRpcClient>,
)> {
    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::PreparingNetworkingStack,
            progress: 0.0,
        })
        .await?;

    let bootstrap_nodes = dsn_bootstrap_nodes(chain_spec)?;

    let network_path = config.node_path.join("network");
    let keypair_path = network_path.join("secret_ed25519");
    let keypair_exists = fs::try_exists(&keypair_path).await.map_err(|error| {
        anyhow::anyhow!(
            "Keypair path \"{}\" doesn't exist and can't be created: {error:?}",
            keypair_path.display()
        )
    })?;

    let network_keypair = if keypair_exists {
        notifications_sender
            .send(BackendNotification::Loading {
                step: LoadingStep::ReadingNetworkKeypair,
                progress: 0.0,
            })
            .await?;

        let mut secret_bytes = fs::read(&keypair_path).await.map_err(|error| {
            anyhow::anyhow!(
                "Failed to read keypair from \"{}\": {error:?}",
                keypair_path.display()
            )
        })?;
        let secret_key = SecretKey::try_from_bytes(&mut secret_bytes)?;

        Keypair::from(secret_key)
    } else {
        notifications_sender
            .send(BackendNotification::Loading {
                step: LoadingStep::GeneratingNetworkKeypair,
                progress: 0.0,
            })
            .await?;

        let network_keypair = Keypair::generate();

        notifications_sender
            .send(BackendNotification::Loading {
                step: LoadingStep::WritingNetworkKeypair,
                progress: 0.0,
            })
            .await?;

        if !fs::try_exists(&network_path).await.map_err(|error| {
            anyhow::anyhow!(
                "Network path \"{}\" doesn't exist and can't be created: {error:?}",
                network_path.display()
            )
        })? {
            fs::create_dir(&network_path).await.map_err(|error| {
                anyhow::anyhow!(
                    "Failed to create network path \"{}\": {error:?}",
                    network_path.display()
                )
            })?;
        }

        let mut options = OpenOptions::new();
        options.write(true).truncate(true).create(true);
        #[cfg(unix)]
        options.mode(0o600);
        options
            .open(&keypair_path)
            .await?
            .write_all(network_keypair.secret().as_ref())
            .await
            .map_err(|error| {
                anyhow::anyhow!(
                    "Failed to write keypair to \"{}\": {error:?}",
                    keypair_path.display()
                )
            })?;

        network_keypair
    };

    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::InstantiatingNetworkingStack,
            progress: 0.0,
        })
        .await?;

    let mut network_options = NetworkOptions {
        keypair: network_keypair.clone(),
        bootstrap_nodes,
        listen_on: vec![
            Multiaddr::from(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
                .with(Protocol::Udp(config.network.subspace_port))
                .with(Protocol::QuicV1),
            Multiaddr::from(IpAddr::V6(Ipv6Addr::UNSPECIFIED))
                .with(Protocol::Udp(config.network.subspace_port))
                .with(Protocol::QuicV1),
            Multiaddr::from(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
                .with(Protocol::Tcp(config.network.subspace_port)),
            Multiaddr::from(IpAddr::V6(Ipv6Addr::UNSPECIFIED))
                .with(Protocol::Tcp(config.network.subspace_port)),
        ],
        ..NetworkOptions::default()
    };
    if config.network.faster_networking {
        network_options.in_connections = 500;
        network_options.out_connections = 500;
        network_options.pending_in_connections = 500;
        network_options.pending_out_connections = 500;
    }
    let plotted_pieces = Arc::<Mutex<Option<PlottedPieces>>>::default();
    let maybe_node_client = MaybeNodeRpcClient::default();

    let weak_plotted_pieces = Arc::downgrade(&plotted_pieces);
    let (farmer_cache, farmer_cache_worker) = FarmerCache::new(
        maybe_node_client.clone(),
        subspace_networking::libp2p::identity::PublicKey::from(network_keypair.public())
            .to_peer_id(),
    );

    let (node, node_runner) = create_network(
        protocol_prefix,
        &network_path,
        network_options,
        weak_plotted_pieces,
        maybe_node_client.clone(),
        farmer_cache.clone(),
    )?;

    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::NetworkingStackCreatedSuccessfully,
            progress: 0.0,
        })
        .await?;

    Ok((
        maybe_node_client,
        node,
        node_runner,
        network_keypair,
        plotted_pieces,
        farmer_cache,
        farmer_cache_worker,
    ))
}

#[allow(clippy::too_many_arguments)]
async fn create_consensus_node(
    network_keypair: &Keypair,
    node_path: PathBuf,
    substrate_port: u16,
    chain_spec: ChainSpec,
    piece_getter: Arc<dyn DsnSyncPieceGetter + Send + Sync + 'static>,
    node: Node,
    maybe_node_rpc_client: &MaybeNodeRpcClient,
    notifications_sender: &mut mpsc::Sender<BackendNotification>,
) -> anyhow::Result<LoadedConsensusChainNode> {
    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::CreatingConsensusNode,
            progress: 0.0,
        })
        .await?;

    let create_consensus_node_fut = node::create_consensus_node(
        network_keypair,
        node_path,
        substrate_port,
        chain_spec,
        piece_getter,
        node,
        maybe_node_rpc_client,
    );
    let consensus_node = match create_consensus_node_fut.await {
        Ok(consensus_node) => consensus_node,
        Err(ConsensusNodeCreationError::Service(error)) => {
            return Err(error.into());
        }
        Err(ConsensusNodeCreationError::IncompatibleChain { compatible_chain }) => {
            return Ok(LoadedConsensusChainNode::Incompatible { compatible_chain });
        }
    };

    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::ConsensusNodeCreatedSuccessfully,
            progress: 0.0,
        })
        .await?;

    Ok(LoadedConsensusChainNode::Compatible(consensus_node))
}

#[allow(clippy::too_many_arguments)]
async fn create_farmer(
    reward_address: PublicKey,
    disk_farms: Vec<DiskFarm>,
    plotted_pieces: Arc<Mutex<Option<PlottedPieces>>>,
    farmer_cache: FarmerCache,
    farmer_cache_worker: FarmerCacheWorker<MaybeNodeRpcClient>,
    node_client: MaybeNodeRpcClient,
    kzg: Kzg,
    piece_getter: PieceGetterWrapper,
    notifications_sender: &mut mpsc::Sender<BackendNotification>,
) -> anyhow::Result<Farmer> {
    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::CreatingFarmer,
            progress: 0.0,
        })
        .await?;

    let farmer_options = FarmerOptions {
        reward_address,
        disk_farms,
        node_client,
        plotted_pieces,
        farmer_cache,
        farmer_cache_worker,
        kzg,
        piece_getter,
    };

    let farmer = farmer::create_farmer(farmer_options).await?;

    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::FarmerCreatedSuccessfully,
            progress: 0.0,
        })
        .await?;

    Ok(farmer)
}

async fn process_backend_actions(
    config_file_path: &Path,
    backend_action_receiver: &mut mpsc::Receiver<BackendAction>,
    notifications_sender: &mut mpsc::Sender<BackendNotification>,
) {
    while let Some(action) = backend_action_receiver.next().await {
        match action {
            BackendAction::NewConfig { raw_config } => {
                let result = raw_config
                    .write_to_path(config_file_path)
                    .await
                    .map_err(|error| {
                        anyhow::anyhow!(
                            "Failed to write config to \"{}\": {}",
                            config_file_path.display(),
                            error
                        )
                    });
                if let Err(error) = notifications_sender
                    .send(BackendNotification::ConfigSaveResult(result))
                    .await
                {
                    error!(%error, "Failed to send config save result notification");
                }
            }
        }
    }
}

pub async fn wipe(
    raw_config: &RawConfig,
    notifications_sender: &mut mpsc::Sender<BackendNotification>,
) -> anyhow::Result<()> {
    let farms = raw_config.farms();
    for (farm_index, farm) in farms.iter().enumerate() {
        let path = &farm.path;
        notifications_sender
            .send(BackendNotification::Loading {
                step: LoadingStep::WipingFarm {
                    farm_index: farm_index as u8,
                    path: path.to_path_buf(),
                },
                progress: 0.0,
            })
            .await?;

        let wipe_fut = tokio::task::spawn_blocking({
            let path = path.to_path_buf();

            move || SingleDiskFarm::wipe(&path)
        });

        match wipe_fut.await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                notifications_sender
                    .send(BackendNotification::IrrecoverableError {
                        error: anyhow::anyhow!(
                            "Failed to wipe farm {farm_index} at {}: {error}",
                            path.display()
                        ),
                    })
                    .await?
            }
            Err(error) => {
                notifications_sender
                    .send(BackendNotification::IrrecoverableError {
                        error: anyhow::anyhow!(
                            "Failed to wipe farm {farm_index} at {}: {error}",
                            path.display()
                        ),
                    })
                    .await?
            }
        }
    }

    {
        let path = &raw_config.node_path();
        notifications_sender
            .send(BackendNotification::Loading {
                step: LoadingStep::WipingNode {
                    path: path.to_path_buf(),
                },
                progress: 0.0,
            })
            .await?;

        // TODO: Remove "paritydb" once support for upgrade from Gemini 3g is no longer necessary
        for subdirectory in &["db", "network", "paritydb"] {
            let path = path.join(subdirectory);

            if fs::try_exists(&path).await.unwrap_or(true) {
                if let Err(error) = fs::remove_dir_all(&path).await {
                    notifications_sender
                        .send(BackendNotification::IrrecoverableError {
                            error: anyhow::anyhow!(
                                "Failed to node subdirectory at {}: {error}",
                                path.display()
                            ),
                        })
                        .await?;
                }
            }
        }
    }

    Ok(())
}
