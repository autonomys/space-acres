// TODO: Make these modules private
mod config;
mod farmer;
mod networking;
mod node;

use crate::backend::config::{Config, ConfigError, RawConfig};
use crate::backend::farmer::maybe_node_client::MaybeNodeRpcClient;
use crate::backend::farmer::{DiskFarm, Farmer, FarmerOptions};
use crate::backend::networking::{create_network, NetworkOptions};
use crate::backend::node::{dsn_bootstrap_nodes, ChainSpec, ConsensusNode, GENESIS_HASH, RPC_PORT};
use future::FutureExt;
use futures::channel::mpsc;
use futures::{future, select, SinkExt};
use parking_lot::Mutex;
use sc_subspace_chain_specs::GEMINI_3G_CHAIN_SPEC;
use std::path::Path;
use std::pin::pin;
use std::sync::Arc;
use subspace_core_primitives::PublicKey;
use subspace_farmer::piece_cache::{CacheWorker, PieceCache};
use subspace_farmer::utils::readers_and_pieces::ReadersAndPieces;
use subspace_farmer::utils::run_future_in_dedicated_thread;
use subspace_farmer::NodeRpcClient;
use subspace_networking::libp2p::identity::ed25519::{Keypair, SecretKey};
use subspace_networking::{Node, NodeRunner};
use tokio::fs;
use tracing::{error, info_span, Instrument};

/// Major steps in application loading progress
#[derive(Debug)]
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

/// Notification messages send from backend about its operation
#[derive(Debug)]
pub enum BackendNotification {
    /// Application loading progress
    Loading {
        /// Major loading step
        step: LoadingStep,
        // TODO: Set this to non-zero use cases
        /// Progress in %: 0.0..=100.0
        progress: f32,
    },
    NotConfigured,
    ConfigurationIsInvalid {
        error: ConfigError,
    },
    Running,
    Stopped {
        /// Error in case stopped due to error
        error: Option<anyhow::Error>,
    },
    IrrecoverableError {
        /// Error that happened
        error: anyhow::Error,
    },
}

// NOTE: this is an async function, but it might do blocking operations and should be running on a
// dedicated CPU core
pub async fn create(mut notifications_sender: mpsc::Sender<BackendNotification>) {
    let (consensus_node, farmer, node_runner) = match load(&mut notifications_sender).await {
        Ok(Some(result)) => result,
        Ok(None) => {
            if let Err(error) = notifications_sender
                .send(BackendNotification::NotConfigured)
                .await
            {
                error!(%error, "Failed to send not configured notification");
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
        consensus_node,
        farmer,
        node_runner,
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
) -> anyhow::Result<Option<(ConsensusNode, Farmer, NodeRunner<PieceCache>)>> {
    let Some(config) = load_configuration(notifications_sender).await? else {
        return Ok(None);
    };

    let Some(config) = check_configuration(&config, notifications_sender).await? else {
        // TODO: Handle invalid configuration by allow user to fix it
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
        &config.node_path,
        GENESIS_HASH.to_string(),
        &chain_spec,
        notifications_sender,
    )
    .await?;

    let consensus_node = create_consensus_node(
        &network_keypair,
        &config.node_path,
        chain_spec,
        node.clone(),
        notifications_sender,
    )
    .await?;

    let farmer = create_farmer(
        config.reward_address,
        config.farms,
        node,
        readers_and_pieces,
        piece_cache,
        piece_cache_worker,
        &maybe_node_client,
        notifications_sender,
    )
    .await?;

    Ok(Some((consensus_node, farmer, node_runner)))
}

async fn run(
    consensus_node: ConsensusNode,
    farmer: Farmer,
    mut node_runner: NodeRunner<PieceCache>,
    notifications_sender: &mut mpsc::Sender<BackendNotification>,
) -> anyhow::Result<()> {
    let networking_fut = run_future_in_dedicated_thread(
        Box::pin({
            let span = info_span!("Network");
            let _enter = span.enter();

            async move { node_runner.run().await }.in_current_span()
        }),
        "networking".to_string(),
    )?;

    notifications_sender
        .send(BackendNotification::Running)
        .await?;

    let consensus_node_fut = pin!(consensus_node.run());
    let farmer_fut = pin!(farmer.run());
    let networking_fut = pin!(networking_fut);

    let result: anyhow::Result<()> = select! {
        result = consensus_node_fut.fuse() => {
            result.map_err(anyhow::Error::from)
        }
        result = farmer_fut.fuse() => {
            result.map_err(anyhow::Error::from)
        }
        result = networking_fut.fuse() => {
            result.map_err(|_cancelled| anyhow::anyhow!("Networking exited"))
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
) -> anyhow::Result<Option<RawConfig>> {
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

    Ok(maybe_config)
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
                .send(BackendNotification::ConfigurationIsInvalid { error })
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

    let chain_spec = node::load_chain_specification(GEMINI_3G_CHAIN_SPEC)
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
    node_path: &Path,
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

    let network_path = node_path.join("network");
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

        fs::write(&keypair_path, network_keypair.secret())
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
    node_path: &Path,
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
        node::create_consensus_node(network_keypair, node_path, chain_spec, node).await?;

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
