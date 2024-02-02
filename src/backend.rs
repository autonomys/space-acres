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
    dsn_bootstrap_nodes, BlockImported, ChainInfo, ChainSpec, ConsensusNode, SyncState,
    GENESIS_HASH, RPC_PORT,
};
use future::FutureExt;
use futures::channel::mpsc;
use futures::{future, select, SinkExt, StreamExt};
use parking_lot::Mutex;
use sc_subspace_chain_specs::GEMINI_3H_CHAIN_SPEC;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::pin::pin;
use std::sync::Arc;
use subspace_core_primitives::{BlockNumber, PublicKey};
use subspace_farmer::piece_cache::{CacheWorker, PieceCache};
use subspace_farmer::utils::readers_and_pieces::ReadersAndPieces;
use subspace_farmer::utils::run_future_in_dedicated_thread;
use subspace_farmer::NodeRpcClient;
use subspace_networking::libp2p::identity::ed25519::{Keypair, SecretKey};
use subspace_networking::libp2p::multiaddr::Protocol;
use subspace_networking::libp2p::Multiaddr;
use subspace_networking::{Node, NodeRunner};
use subspace_runtime_primitives::Balance;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Handle;
use tracing::{error, info_span, warn, Instrument};

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
    node_runner: NodeRunner<PieceCache>,
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
        Ok(loaded_backend) => {
            // Loaded successfully
            loaded_backend
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
) -> anyhow::Result<Option<LoadedBackend>> {
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
        readers_and_pieces,
        piece_cache,
        piece_cache_worker,
    ) = create_networking_stack(
        &config,
        GENESIS_HASH.to_string(),
        &chain_spec,
        notifications_sender,
    )
    .await?;

    let consensus_node = create_consensus_node(
        &network_keypair,
        config.node_path.clone(),
        config.network.substrate_port,
        chain_spec,
        node.clone(),
        notifications_sender,
    )
    .await?;

    let farmer = create_farmer(
        config.reward_address,
        config.farms.clone(),
        node,
        readers_and_pieces,
        piece_cache,
        piece_cache_worker,
        &maybe_node_client,
        notifications_sender,
    )
    .await?;

    Ok(Some(LoadedBackend {
        config,
        raw_config,
        config_file_path,
        consensus_node,
        farmer,
        node_runner,
    }))
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
            result.map_err(anyhow::Error::from)
        }
        result = consensus_node_fut.fuse() => {
            result.map_err(anyhow::Error::from)
        }
        result = farmer_fut.fuse() => {
            result.map_err(|_cancelled| anyhow::anyhow!("Networking exited"))
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
    NodeRunner<PieceCache>,
    Keypair,
    Arc<Mutex<Option<ReadersAndPieces>>>,
    PieceCache,
    CacheWorker<MaybeNodeRpcClient>,
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
        options.mode(0x600);
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

    let network_options = NetworkOptions {
        // TODO: Persist keypair on disk
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
    let readers_and_pieces = Arc::<Mutex<Option<ReadersAndPieces>>>::default();
    let maybe_node_client = MaybeNodeRpcClient::default();

    let weak_readers_and_pieces = Arc::downgrade(&readers_and_pieces);
    let (piece_cache, piece_cache_worker) = PieceCache::new(
        maybe_node_client.clone(),
        subspace_networking::libp2p::identity::PublicKey::from(network_keypair.public())
            .to_peer_id(),
    );

    let (node, node_runner) = create_network(
        protocol_prefix,
        &network_path,
        network_options,
        weak_readers_and_pieces,
        maybe_node_client.clone(),
        piece_cache.clone(),
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
        readers_and_pieces,
        piece_cache,
        piece_cache_worker,
    ))
}

async fn create_consensus_node(
    network_keypair: &Keypair,
    node_path: PathBuf,
    substrate_port: u16,
    chain_spec: ChainSpec,
    node: Node,
    notifications_sender: &mut mpsc::Sender<BackendNotification>,
) -> anyhow::Result<ConsensusNode> {
    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::CreatingConsensusNode,
            progress: 0.0,
        })
        .await?;

    let consensus_node =
        node::create_consensus_node(network_keypair, node_path, substrate_port, chain_spec, node)
            .await?;

    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::ConsensusNodeCreatedSuccessfully,
            progress: 0.0,
        })
        .await?;

    Ok(consensus_node)
}

#[allow(clippy::too_many_arguments)]
async fn create_farmer(
    reward_address: PublicKey,
    disk_farms: Vec<DiskFarm>,
    node: Node,
    readers_and_pieces: Arc<Mutex<Option<ReadersAndPieces>>>,
    piece_cache: PieceCache,
    piece_cache_worker: CacheWorker<MaybeNodeRpcClient>,
    maybe_node_client: &MaybeNodeRpcClient,
    notifications_sender: &mut mpsc::Sender<BackendNotification>,
) -> anyhow::Result<Farmer> {
    notifications_sender
        .send(BackendNotification::Loading {
            step: LoadingStep::CreatingFarmer,
            progress: 0.0,
        })
        .await?;

    let node_client = NodeRpcClient::new(&format!("ws://127.0.0.1:{RPC_PORT}")).await?;

    // Inject working node client into wrapper we have created before such that networking can respond to incoming
    // requests properly
    maybe_node_client.inject(node_client.clone());

    let farmer_options = FarmerOptions {
        reward_address,
        disk_farms,
        node_client,
        node,
        readers_and_pieces,
        piece_cache,
        piece_cache_worker,
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
