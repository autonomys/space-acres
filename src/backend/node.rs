mod utils;

use crate::backend::farmer::direct_node_client::{DirectNodeClient, NodeClientConfig};
use crate::backend::farmer::maybe_node_client::MaybeNodeClient;
use crate::backend::node::utils::account_storage_key;
use crate::backend::utils::{Handler, HandlerFn};
use crate::PosTable;
use event_listener_primitives::HandlerId;
use frame_system::AccountInfo;
use futures::{select, FutureExt, StreamExt};
use names::{Generator, Name};
use pallet_balances::AccountData;
use parity_scale_codec::Decode;
use sc_client_api::client::BlockchainEvents;
use sc_client_api::{HeaderBackend, StorageProvider};
use sc_client_db::PruningMode;
use sc_consensus_slots::SlotProportion;
use sc_network::config::{Ed25519Secret, NodeKeyConfig, NonReservedPeerMode, SetConfig};
use sc_service::{BlocksPruning, Configuration, GenericChainSpec, NoExtension};
use sc_storage_monitor::{StorageMonitorParams, StorageMonitorService};
use serde_json::Value;
use sp_api::ProvideRuntimeApi;
use sp_consensus_subspace::{ChainConstants, SubspaceApi};
use sp_core::crypto::Ss58AddressFormat;
use sp_core::storage::StorageKey;
use sp_core::H256;
use sp_runtime::traits::Header;
use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use subspace_core_primitives::solutions::SolutionRange;
use subspace_core_primitives::{BlockNumber, PublicKey};
use subspace_fake_runtime_api::RuntimeApi;
use subspace_networking::libp2p::identity::ed25519::Keypair;
use subspace_networking::libp2p::Multiaddr;
use subspace_networking::Node;
use subspace_runtime_primitives::{Balance, Nonce};
use subspace_service::config::{
    ChainSyncMode, SubspaceConfiguration, SubspaceNetworking, SubstrateConfiguration,
    SubstrateNetworkConfiguration, SubstrateRpcConfiguration,
};
use subspace_service::sync_from_dsn::DsnSyncPieceGetter;
use subspace_service::{FullClient, NewFull};
use tokio::time::MissedTickBehavior;
use tracing::{error, info_span};

pub(super) const GENESIS_HASH: &str =
    "66455a580aabff303720aa83adbe6c44502922251c03ba73686d5245da9e21bd";
const SYNC_STATUS_EVENT_INTERVAL: Duration = Duration::from_secs(5);
/// Roughly 138k empty blocks can fit into one archived segment, hence we need to not allow to prune
/// more blocks that this
const MIN_STATE_PRUNING: BlockNumber = 140_000;

/// The maximum number of characters for a node name.
const NODE_NAME_MAX_LENGTH: usize = 64;

#[derive(Debug, thiserror::Error)]
pub(super) enum ConsensusNodeCreationError {
    /// Substrate service error
    #[error("Substrate service error: {0}")]
    Service(#[from] sc_service::Error),
    /// Incompatible chain
    #[error("Incompatible chain, only {compatible_chain} is supported")]
    IncompatibleChain { compatible_chain: String },
}

pub(super) struct ChainSpec(Box<dyn sc_service::ChainSpec>);

impl From<ChainSpec> for Box<dyn sc_service::ChainSpec> {
    fn from(value: ChainSpec) -> Self {
        value.0
    }
}

impl fmt::Debug for ChainSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ChainSpec").finish_non_exhaustive()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ChainInfo {
    pub chain_name: String,
    pub protocol_id: String,
    pub token_symbol: String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SyncKind {
    Dsn,
    Regular,
}

impl SyncKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncKind::Dsn => "dsn",
            SyncKind::Regular => "regular",
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum SyncState {
    #[default]
    Unknown,
    Syncing {
        kind: SyncKind,
        target: BlockNumber,
    },
    Idle,
}

impl SyncState {
    pub fn is_synced(&self) -> bool {
        matches!(self, SyncState::Idle)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BlockImportedNotification {
    pub number: BlockNumber,
    pub reward_address_balance: Balance,
    pub solution_range: SolutionRange,
    pub voting_solution_range: SolutionRange,
}

#[derive(Default, Debug)]
struct Handlers {
    sync_state_change: Handler<SyncState>,
    block_imported: Handler<BlockImportedNotification>,
}

pub(super) struct ConsensusNode {
    full_node: NewFull<FullClient<RuntimeApi>>,
    pause_sync: Arc<AtomicBool>,
    chain_info: ChainInfo,
    chain_constants: ChainConstants,
    handlers: Handlers,
}

impl fmt::Debug for ConsensusNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ConsensusNode").finish_non_exhaustive()
    }
}

impl ConsensusNode {
    fn new(
        full_node: NewFull<FullClient<RuntimeApi>>,
        pause_sync: Arc<AtomicBool>,
        chain_info: ChainInfo,
        chain_constants: ChainConstants,
    ) -> Self {
        Self {
            full_node,
            pause_sync,
            chain_info,
            chain_constants,
            handlers: Handlers::default(),
        }
    }

    pub(super) async fn run(mut self, reward_address: &PublicKey) -> Result<(), sc_service::Error> {
        self.full_node.network_starter.start_network();

        let spawn_essential_handle = self.full_node.task_manager.spawn_essential_handle();
        spawn_essential_handle.spawn_blocking(
            "block-import-notifications",
            Some("space-acres-node"),
            {
                let client = self.full_node.client.clone();
                let reward_address_storage_key = account_storage_key(reward_address);

                async move {
                    let mut block_import_stream = client.every_import_notification_stream();

                    while let Some(block_import) = block_import_stream.next().await {
                        if block_import.is_new_best {
                            let best_hash = client.info().best_hash;
                            let runtime_api = client.runtime_api();
                            let solution_ranges =
                                runtime_api.solution_ranges(best_hash).unwrap_or_default();

                            let block_imported_notification = BlockImportedNotification {
                                number: *block_import.header.number(),
                                // TODO: This is not pretty that we do it here, but not clear what
                                //  would be a nicer API
                                reward_address_balance: get_total_account_balance(
                                    &client,
                                    block_import.header.hash(),
                                    &reward_address_storage_key,
                                )
                                .unwrap_or_default(),
                                solution_range: solution_ranges.current,
                                voting_solution_range: solution_ranges.voting_current,
                            };
                            self.handlers
                                .block_imported
                                .call_simple(&block_imported_notification);
                        }
                    }
                }
            },
        );
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
                            kind: if self.pause_sync.load(Ordering::Acquire) {
                                // We are pausing Substrate's sync during sync from DNS
                                SyncKind::Dsn
                            } else {
                                SyncKind::Regular
                            },
                            target: sync_status.best_seen_block.unwrap_or_default(),
                        }
                    } else if self.full_node.sync_service.num_connected_peers() > 0 {
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

        let task_manager = self.full_node.task_manager.future();

        select! {
            result = task_manager.fuse() => {
                result?;
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

    pub(super) fn account_balance(&self, account: &PublicKey) -> Balance {
        let reward_address_storage_key = account_storage_key(account);

        get_total_account_balance(
            &self.full_node.client,
            self.full_node.client.info().best_hash,
            &reward_address_storage_key,
        )
        .unwrap_or_default()
    }

    pub(super) fn chain_info(&self) -> &ChainInfo {
        &self.chain_info
    }

    pub(super) fn chain_constants(&self) -> &ChainConstants {
        &self.chain_constants
    }

    pub(super) fn on_sync_state_change(&self, callback: HandlerFn<SyncState>) -> HandlerId {
        self.handlers.sync_state_change.add(callback)
    }

    pub(super) fn on_block_imported(
        &self,
        callback: HandlerFn<BlockImportedNotification>,
    ) -> HandlerId {
        self.handlers.block_imported.add(callback)
    }
}

fn get_total_account_balance(
    client: &FullClient<RuntimeApi>,
    block_hash: H256,
    address_storage_key: &StorageKey,
) -> Option<Balance> {
    let encoded_account_info = match client.storage(block_hash, address_storage_key) {
        Ok(maybe_encoded_account_info) => maybe_encoded_account_info?,
        Err(error) => {
            error!(%error, "Failed to query account balance");
            return None;
        }
    };

    let account_info = match AccountInfo::<Nonce, AccountData<Balance>>::decode(
        &mut encoded_account_info.0.as_slice(),
    ) {
        Ok(account_info) => account_info,
        Err(error) => {
            error!(%error, "Failed to decode account info");
            return None;
        }
    };

    let account_data = account_info.data;
    Some(account_data.free + account_data.reserved + account_data.frozen)
}

pub(super) fn load_chain_specification(chain_spec: &'static [u8]) -> Result<ChainSpec, String> {
    GenericChainSpec::<NoExtension, ()>::from_json_bytes(chain_spec)
        .map(|chain_spec| ChainSpec(Box::new(chain_spec)))
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
        .map(|d| match d.clone() {
            Value::String(s) => Ok(Some(s)),
            Value::Null => Ok(None),
            _ => Err(sc_service::Error::Other(
                "Failed to decode PoT initial key".to_string(),
            )),
        })
        .transpose()?
        .flatten();
    Ok(maybe_chain_spec_pot_external_entropy
        .unwrap_or_default()
        .into_bytes())
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
    base_path: PathBuf,
    substrate_port: u16,
    chain_spec: ChainSpec,
) -> SubstrateConfiguration {
    let telemetry_endpoints = chain_spec.0.telemetry_endpoints().clone();

    SubstrateConfiguration {
        impl_name: env!("CARGO_PKG_NAME").to_string(),
        impl_version: env!("CARGO_PKG_VERSION").to_string(),
        farmer: true,
        base_path,
        transaction_pool: Default::default(),
        network: SubstrateNetworkConfiguration {
            listen_on: vec![
                sc_network::Multiaddr::from(sc_network::multiaddr::Protocol::Ip4(
                    Ipv4Addr::UNSPECIFIED,
                ))
                .with(sc_network::multiaddr::Protocol::Tcp(substrate_port)),
                sc_network::Multiaddr::from(sc_network::multiaddr::Protocol::Ip6(
                    Ipv6Addr::UNSPECIFIED,
                ))
                .with(sc_network::multiaddr::Protocol::Tcp(substrate_port)),
            ],
            public_addresses: Vec::new(),
            bootstrap_nodes: chain_spec.0.boot_nodes().to_vec(),
            node_key: NodeKeyConfig::Ed25519(Ed25519Secret::Input(
                sc_network_types::ed25519::SecretKey::try_from_bytes(
                    keypair.secret().as_ref().to_vec(),
                )
                .expect("Correct secret; qed"),
            )),
            default_peers_set: SetConfig {
                // Substrate's default
                in_peers: 8,
                // Substrate's default
                out_peers: 32,
                reserved_nodes: Vec::new(),
                non_reserved_mode: NonReservedPeerMode::Accept,
            },
            node_name: generate_node_name(),
            allow_private_ips: false,
            sync_mode: ChainSyncMode::Snap,
            force_synced: false,
        },
        state_pruning: PruningMode::blocks_pruning(MIN_STATE_PRUNING),
        blocks_pruning: BlocksPruning::Some(256),
        // TODO: Make the whole `rpc_options` optional instead
        rpc_options: SubstrateRpcConfiguration {
            listen_on: None,
            // Substrate's default
            max_connections: 100,
            cors: Some(Vec::new()),
            methods: Default::default(),
            // Substrate's default
            rate_limit: None,
            rate_limit_whitelisted_ips: Vec::new(),
            rate_limit_trust_proxy_headers: false,
            max_subscriptions_per_connection: 1024,
            message_buffer_capacity_per_connection: 64,
            disable_batch_requests: false,
            max_batch_request_len: None,
        },
        prometheus_listen_on: None,
        telemetry_endpoints,
        force_authoring: false,
        chain_spec: chain_spec.into(),
    }
}

pub(super) async fn create_consensus_node(
    keypair: &Keypair,
    base_path: PathBuf,
    substrate_port: u16,
    chain_spec: ChainSpec,
    piece_getter: Arc<dyn DsnSyncPieceGetter + Send + Sync + 'static>,
    node: Node,
    maybe_node_client: &MaybeNodeClient,
) -> Result<ConsensusNode, ConsensusNodeCreationError> {
    set_default_ss58_version(&chain_spec);

    let pot_external_entropy = pot_external_entropy(&chain_spec)?;
    let dsn_bootstrap_nodes = dsn_bootstrap_nodes(&chain_spec)?;

    let chain_info = ChainInfo {
        chain_name: chain_spec.0.name().to_string(),
        protocol_id: chain_spec.0.protocol_id().unwrap_or_default().to_string(),
        token_symbol: chain_spec
            .0
            .properties()
            .get("tokenSymbol")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
    };

    let consensus_chain_config =
        create_consensus_chain_config(keypair, base_path.clone(), substrate_port, chain_spec);
    let sync = consensus_chain_config.network.sync_mode;
    let consensus_chain_config = Configuration::from(consensus_chain_config);
    let pause_sync = Arc::clone(&consensus_chain_config.network.pause_sync);

    let (consensus_node, direct_node_client) = {
        let span = info_span!("Node");
        let _enter = span.enter();

        let consensus_chain_config = SubspaceConfiguration {
            base: consensus_chain_config,
            // Domain node needs slots notifications for bundle production
            force_new_slot_notifications: false,
            create_object_mappings: false,
            subspace_networking: SubspaceNetworking::Reuse {
                node,
                bootstrap_nodes: dsn_bootstrap_nodes,
            },
            dsn_piece_getter: Some(piece_getter),
            is_timekeeper: false,
            timekeeper_cpu_cores: Default::default(),
            sync,
        };

        let partial_components_result = subspace_service::new_partial::<PosTable, RuntimeApi>(
            &consensus_chain_config.base,
            match consensus_chain_config.sync {
                ChainSyncMode::Full => false,
                ChainSyncMode::Snap => true,
            },
            &pot_external_entropy,
        );

        let partial_components = match partial_components_result {
            Ok(partial_components) => partial_components,
            Err(error) => {
                // TODO: This is a workaround to what and how initialization does, remove this at
                //  some point in the future once upgrade from Gemini networks is no longer needed
                if error.to_string().contains(
                    "env:ext_fraud_proof_runtime_interface_derive_bundle_digest_version_2",
                ) {
                    return Err(ConsensusNodeCreationError::IncompatibleChain {
                        compatible_chain: consensus_chain_config.base.chain_spec.name().to_string(),
                    });
                } else {
                    return Err(sc_service::Error::Other(format!(
                        "Failed to build a full subspace node: {error:?}"
                    ))
                    .into());
                }
            }
        };

        if hex::encode(partial_components.client.info().genesis_hash) != GENESIS_HASH {
            return Err(ConsensusNodeCreationError::IncompatibleChain {
                compatible_chain: consensus_chain_config.base.chain_spec.name().to_string(),
            });
        }

        let client = partial_components.client.clone();
        let segment_headers_store = partial_components.other.segment_headers_store.clone();
        let kzg = partial_components.other.subspace_link.kzg().clone();
        let erasure_coding = partial_components
            .other
            .subspace_link
            .erasure_coding()
            .clone();

        let consensus_node = subspace_service::new_full::<PosTable, _>(
            consensus_chain_config,
            partial_components,
            None,
            true,
            SlotProportion::new(3f32 / 4f32),
        )
        .await
        .map_err(|error| {
            sc_service::Error::Other(format!("Failed to build a full subspace node 3: {error:?}"))
        })?;

        let direct_node_client = DirectNodeClient::new(NodeClientConfig {
            client,
            segment_headers_store,
            subscription_executor: Arc::new(consensus_node.task_manager.spawn_handle()),
            new_slot_notification_stream: consensus_node.new_slot_notification_stream.clone(),
            reward_signing_notification_stream: consensus_node
                .reward_signing_notification_stream
                .clone(),
            archived_segment_notification_stream: consensus_node
                .archived_segment_notification_stream
                .clone(),
            dsn_bootstrap_nodes: vec![],
            sync_oracle: consensus_node.sync_service.clone(),
            kzg,
            erasure_coding,
        })
        .map_err(|error| {
            sc_service::Error::Other(format!("Failed to build a node client: {error:?}"))
        })?;

        (consensus_node, direct_node_client)
    };

    StorageMonitorService::try_spawn(
        StorageMonitorParams {
            // Substrate's default, in MiB
            threshold: 1024,
            // Substrate's default, in seconds
            polling_period: 5,
        },
        base_path,
        &consensus_node.task_manager.spawn_essential_handle(),
    )
    .map_err(|error| {
        sc_service::Error::Other(format!("Failed to start storage monitor: {error:?}"))
    })?;

    // Inject working node client into wrapper we have created before such that networking can
    // respond to incoming requests properly
    maybe_node_client.inject(Box::new(direct_node_client));

    let chain_constants = consensus_node
        .client
        .runtime_api()
        .chain_constants(consensus_node.client.info().best_hash)
        .map_err(|error| {
            sc_service::Error::Other(format!(
                "Failed to get chain constants from client: {error:?}"
            ))
        })?;

    Ok(ConsensusNode::new(
        consensus_node,
        pause_sync,
        chain_info,
        chain_constants,
    ))
}
