use crate::backend::utils::{Handler, HandlerFn};
use crate::PosTable;
use atomic::Atomic;
use event_listener_primitives::HandlerId;
use futures::{select, FutureExt, StreamExt};
use names::{Generator, Name};
use sc_client_api::client::BlockchainEvents;
use sc_client_api::HeaderBackend;
use sc_client_db::{DatabaseSource, PruningMode};
use sc_consensus_slots::SlotProportion;
use sc_network::config::{Ed25519Secret, NetworkConfiguration, NodeKeyConfig, SyncMode};
use sc_service::config::{KeystoreConfig, OffchainWorkerConfig};
use sc_service::{
    BasePath, BlocksPruning, Configuration, NativeExecutionDispatch, Role, RpcMethods,
};
use sc_storage_monitor::{StorageMonitorParams, StorageMonitorService};
use sc_subspace_chain_specs::ConsensusChainSpec;
use sp_core::crypto::Ss58AddressFormat;
use sp_runtime::traits::Header;
use std::fmt;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use subspace_core_primitives::BlockNumber;
use subspace_networking::libp2p::identity::ed25519::Keypair;
use subspace_networking::libp2p::Multiaddr;
use subspace_networking::Node;
use subspace_runtime::{RuntimeApi, RuntimeGenesisConfig};
use subspace_service::{FullClient, NewFull, SubspaceConfiguration, SubspaceNetworking};
use tokio::runtime::Handle;
use tokio::time::MissedTickBehavior;

pub(super) const GENESIS_HASH: &str =
    "418040fc282f5e5ddd432c46d05297636f6f75ce68d66499ff4cbda69ccd180b";
pub(super) const RPC_PORT: u16 = 9944;
const SYNC_STATUS_EVENT_INTERVAL: Duration = Duration::from_secs(5);

/// The maximum number of characters for a node name.
const NODE_NAME_MAX_LENGTH: usize = 64;

pub(super) struct ChainSpec(ConsensusChainSpec<RuntimeGenesisConfig>);

impl fmt::Debug for ChainSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ChainSpec").finish_non_exhaustive()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SyncKind {
    Dsn,
    Regular,
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum SyncState {
    #[default]
    Unknown,
    Syncing {
        kind: SyncKind,
        target: BlockNumber,
        /// Sync speed in blocks/s
        speed: Option<f32>,
    },
    Idle,
}

#[derive(Default, Debug)]
struct Handlers {
    sync_state_change: Handler<SyncState>,
    block_imported: Handler<BlockNumber>,
}

pub(super) struct ConsensusNode {
    full_node: NewFull<FullClient<RuntimeApi, ExecutorDispatch>>,
    sync_mode: Arc<Atomic<SyncMode>>,
    handlers: Handlers,
}

impl fmt::Debug for ConsensusNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ConsensusNode").finish_non_exhaustive()
    }
}

impl ConsensusNode {
    fn new(
        full_node: NewFull<FullClient<RuntimeApi, ExecutorDispatch>>,
        sync_mode: Arc<Atomic<SyncMode>>,
    ) -> Self {
        Self {
            full_node,
            sync_mode,
            handlers: Handlers::default(),
        }
    }

    pub(super) async fn run(mut self) -> Result<(), sc_service::Error> {
        self.full_node.network_starter.start_network();

        let task_manager = self.full_node.task_manager.future();
        let block_import_notifications_fut = async {
            let mut block_import_stream = self.full_node.client.every_import_notification_stream();

            while let Some(block_import) = block_import_stream.next().await {
                // Nothing else to do
                if block_import.is_new_best {
                    self.handlers
                        .block_imported
                        .call_simple(block_import.header.number());
                }
            }
        };
        let sync_status_notifications_fut = async {
            let mut sync_status_interval = tokio::time::interval(SYNC_STATUS_EVENT_INTERVAL);
            sync_status_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            let mut last_sync_state = SyncState::Unknown;
            self.handlers
                .sync_state_change
                .call_simple(&last_sync_state);

            loop {
                sync_status_interval.tick().await;

                if let Ok(sync_status) = self.full_node.sync_service.status().await {
                    let sync_state = if sync_status.state.is_major_syncing() {
                        SyncState::Syncing {
                            kind: match self.sync_mode.load(Ordering::Acquire) {
                                SyncMode::Paused => {
                                    // We are pausing Substrate's sync during sync from DNS
                                    SyncKind::Dsn
                                }
                                _ => SyncKind::Regular,
                            },
                            target: sync_status.best_seen_block.unwrap_or_default(),
                            // TODO: Sync speed
                            speed: None,
                        }
                    } else if sync_status.num_connected_peers > 0 {
                        SyncState::Idle
                    } else {
                        SyncState::Unknown
                    };

                    if sync_state != last_sync_state {
                        self.handlers.sync_state_change.call_simple(&sync_state);

                        last_sync_state = sync_state;
                    }
                }
            }
        };

        select! {
            result = task_manager.fuse() => {
                result?;
            }
            _ = block_import_notifications_fut.fuse() => {
                // Nothing else to do
            }
            _ = sync_status_notifications_fut.fuse() => {
                // Nothing else to do
            }
        }

        Ok(())
    }

    pub(super) fn best_block_number(&self) -> BlockNumber {
        self.full_node.client.info().best_number
    }

    pub(super) fn on_sync_state_change(&self, callback: HandlerFn<SyncState>) -> HandlerId {
        self.handlers.sync_state_change.add(callback)
    }

    pub(super) fn on_block_imported(&self, callback: HandlerFn<BlockNumber>) -> HandlerId {
        self.handlers.block_imported.add(callback)
    }
}

/// Executor dispatch for subspace runtime
pub(super) struct ExecutorDispatch;

impl NativeExecutionDispatch for ExecutorDispatch {
    type ExtendHostFunctions = (
        sp_consensus_subspace::consensus::HostFunctions,
        sp_domains_fraud_proof::HostFunctions,
    );

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        subspace_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        subspace_runtime::native_version()
    }
}

pub(super) fn load_chain_specification(chain_spec: &'static [u8]) -> Result<ChainSpec, String> {
    ConsensusChainSpec::from_json_bytes(chain_spec).map(ChainSpec)
}

fn set_default_ss58_version(chain_spec: &ChainSpec) {
    let maybe_ss58_address_format = chain_spec
        .0
        .properties()
        .get("ss58Format")
        .map(|v| {
            v.as_u64()
                .expect("ss58Format must always be an unsigned number; qed")
        })
        .map(|v| {
            v.try_into()
                .expect("ss58Format must always be within u16 range; qed")
        })
        .map(Ss58AddressFormat::custom);

    if let Some(ss58_address_format) = maybe_ss58_address_format {
        sp_core::crypto::set_default_ss58_version(ss58_address_format);
    }
}

fn pot_external_entropy(chain_spec: &ChainSpec) -> Result<Vec<u8>, sc_service::Error> {
    let maybe_chain_spec_pot_external_entropy = chain_spec
        .0
        .properties()
        .get("potExternalEntropy")
        .map(|d| serde_json::from_value(d.clone()))
        .transpose()
        .map_err(|error| {
            sc_service::Error::Other(format!("Failed to decode PoT initial key: {error:?}"))
        })?
        .flatten();
    Ok(maybe_chain_spec_pot_external_entropy.unwrap_or_default())
}

pub(super) fn dsn_bootstrap_nodes(
    chain_spec: &ChainSpec,
) -> Result<Vec<Multiaddr>, sc_service::Error> {
    Ok(chain_spec
        .0
        .properties()
        .get("dsnBootstrapNodes")
        .map(|d| serde_json::from_value(d.clone()))
        .transpose()
        .map_err(|error| {
            sc_service::Error::Other(format!("Failed to decode DSN bootstrap nodes: {error:?}"))
        })?
        .unwrap_or_default())
}

/// Generate a valid random name for the node
pub(super) fn generate_node_name() -> String {
    loop {
        let node_name = Generator::with_naming(Name::Numbered)
            .next()
            .expect("RNG is available on all supported platforms; qed");
        let count = node_name.chars().count();

        if count < NODE_NAME_MAX_LENGTH {
            return node_name;
        }
    }
}

fn create_consensus_chain_config(
    keypair: &Keypair,
    base_path: &Path,
    chain_spec: ChainSpec,
) -> Configuration {
    let telemetry_endpoints = chain_spec.0.telemetry_endpoints().clone();

    Configuration {
        impl_name: env!("CARGO_PKG_NAME").to_string(),
        impl_version: env!("CARGO_PKG_VERSION").to_string(),
        role: Role::Authority,
        tokio_handle: Handle::current(),
        transaction_pool: Default::default(),
        network: {
            let mut network = NetworkConfiguration::new(
                generate_node_name(),
                format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
                NodeKeyConfig::Ed25519(Ed25519Secret::Input(
                    libp2p_identity_substate::ed25519::SecretKey::try_from_bytes(
                        keypair.secret().as_ref().to_vec(),
                    )
                    .expect("Correct keypair, just libp2p version is different; qed"),
                )),
                None,
            );

            network.boot_nodes = chain_spec.0.boot_nodes().to_vec();
            // Substrate's default
            network.default_peers_set.out_peers = 8;
            // Substrate's default
            network.default_peers_set.in_peers = 32;

            network
        },
        keystore: KeystoreConfig::InMemory,
        database: DatabaseSource::ParityDb {
            path: base_path.join("paritydb"),
        },
        // Substrate's default
        trie_cache_maximum_size: Some(64 * 1024 * 1024),
        state_pruning: Some(PruningMode::ArchiveCanonical),
        blocks_pruning: BlocksPruning::Some(256),
        chain_spec: Box::new(chain_spec.0),
        wasm_method: Default::default(),
        wasm_runtime_overrides: None,
        rpc_addr: None,
        // Substrate's default
        rpc_max_connections: 100,
        // TODO: Replace with `Some(Vec::new())` once node client for farmer is rewritten
        rpc_cors: Some(vec![
            "http://localhost:*".to_string(),
            "http://127.0.0.1:*".to_string(),
            "https://localhost:*".to_string(),
            "https://127.0.0.1:*".to_string(),
            "https://polkadot.js.org".to_string(),
        ]),
        // TODO: Disable unsafe methods once node client for farmer is rewritten
        rpc_methods: RpcMethods::Unsafe,
        // Substrate's default, in MiB
        rpc_max_request_size: 15,
        // Substrate's default, in MiB
        rpc_max_response_size: 15,
        rpc_id_provider: None,
        // Substrate's default
        rpc_max_subs_per_conn: 1024,
        // Substrate's default
        rpc_port: RPC_PORT,
        prometheus_config: None,
        telemetry_endpoints,
        default_heap_pages: None,
        // Substrate's default
        offchain_worker: OffchainWorkerConfig {
            enabled: true,
            indexing_enabled: false,
        },
        force_authoring: false,
        disable_grandpa: false,
        dev_key_seed: None,
        tracing_targets: None,
        tracing_receiver: Default::default(),
        // Substrate's default
        max_runtime_instances: 8,
        // Substrate's default
        announce_block: true,
        data_path: Default::default(),
        base_path: BasePath::new(base_path),
        informant_output_format: Default::default(),
        // Substrate's default
        runtime_cache_size: 2,
    }
}

pub(super) async fn create_consensus_node(
    keypair: &Keypair,
    base_path: &Path,
    chain_spec: ChainSpec,
    node: Node,
) -> Result<ConsensusNode, sc_service::Error> {
    set_default_ss58_version(&chain_spec);

    let pot_external_entropy = pot_external_entropy(&chain_spec)?;
    let dsn_bootstrap_nodes = dsn_bootstrap_nodes(&chain_spec)?;

    let consensus_chain_config = create_consensus_chain_config(keypair, base_path, chain_spec);
    let sync_mode = Arc::clone(&consensus_chain_config.network.sync_mode);

    let database_source = consensus_chain_config.database.clone();

    let consensus_node = {
        let span = tracing::info_span!("Node");
        let _enter = span.enter();

        let consensus_chain_config = SubspaceConfiguration {
            base: consensus_chain_config,
            // Domain node needs slots notifications for bundle production
            force_new_slot_notifications: false,
            subspace_networking: SubspaceNetworking::Reuse {
                node,
                bootstrap_nodes: dsn_bootstrap_nodes,
                metrics_registry: None,
            },
            sync_from_dsn: true,
            enable_subspace_block_relay: true,
            is_timekeeper: false,
            timekeeper_cpu_cores: Default::default(),
        };

        let partial_components = subspace_service::new_partial::<
            PosTable,
            RuntimeApi,
            ExecutorDispatch,
        >(&consensus_chain_config.base, &pot_external_entropy)
        .map_err(|error| {
            sc_service::Error::Other(format!("Failed to build a full subspace node: {error:?}"))
        })?;

        subspace_service::new_full::<PosTable, _, _>(
            consensus_chain_config,
            partial_components,
            true,
            SlotProportion::new(3f32 / 4f32),
        )
        .await
        .map_err(|error| {
            sc_service::Error::Other(format!("Failed to build a full subspace node: {error:?}"))
        })?
    };

    StorageMonitorService::try_spawn(
        StorageMonitorParams {
            // Substrate's default, in MiB
            threshold: 1024,
            // Substrate's default, in seconds
            polling_period: 5,
        },
        database_source,
        &consensus_node.task_manager.spawn_essential_handle(),
    )
    .map_err(|error| {
        sc_service::Error::Other(format!("Failed to start storage monitor: {error:?}"))
    })?;

    Ok(ConsensusNode::new(consensus_node, sync_mode))
}
