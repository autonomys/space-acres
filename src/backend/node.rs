mod utils;

use crate::backend::farmer::maybe_node_client::MaybeNodeRpcClient;
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
use sc_informant::OutputFormat;
use sc_network::config::{Ed25519Secret, NodeKeyConfig, NonReservedPeerMode, SetConfig};
use sc_service::{BlocksPruning, Configuration, GenericChainSpec};
use sc_storage_monitor::{StorageMonitorParams, StorageMonitorService};
use serde_json::Value;
use sp_core::crypto::Ss58AddressFormat;
use sp_core::storage::StorageKey;
use sp_core::H256;
use sp_runtime::traits::Header;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use subspace_core_primitives::{BlockNumber, PublicKey};
use subspace_fake_runtime_api::RuntimeApi;
use subspace_farmer::NodeRpcClient;
use subspace_networking::libp2p::identity::ed25519::Keypair;
use subspace_networking::libp2p::Multiaddr;
use subspace_networking::Node;
use subspace_runtime_primitives::{Balance, Nonce};
use subspace_service::config::{
    SubspaceConfiguration, SubspaceNetworking, SubstrateConfiguration,
    SubstrateNetworkConfiguration, SubstrateRpcConfiguration,
};
use subspace_service::sync_from_dsn::DsnSyncPieceGetter;
use subspace_service::{FullClient, NewFull};
use subxt::config::PolkadotConfig;
use subxt::OnlineClient;
use tokio::fs;
use tokio::time::MissedTickBehavior;
use tracing::error;

pub(super) const GENESIS_HASH: &str =
    "0c121c75f4ef450f40619e1fca9d1e8e7fbabc42c895bc4790801e85d5a91c34";
pub(super) const RPC_PORT: u16 = 19944;
const SYNC_STATUS_EVENT_INTERVAL: Duration = Duration::from_secs(5);

/// The maximum number of characters for a node name.
const NODE_NAME_MAX_LENGTH: usize = 64;

#[derive(Debug, thiserror::Error)]
pub(super) enum ConsensusNodeCreationError {
    /// Substrate service error
    #[error("Substate service error: {0}")]
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
pub struct BlockImported {
    pub number: BlockNumber,
    pub reward_address_balance: Balance,
}

#[derive(Default, Debug)]
struct Handlers {
    sync_state_change: Handler<SyncState>,
    block_imported: Handler<BlockImported>,
}

pub(super) struct ConsensusNode {
    full_node: NewFull<FullClient<RuntimeApi>>,
    pause_sync: Arc<AtomicBool>,
    chain_info: ChainInfo,
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
    ) -> Self {
        Self {
            full_node,
            pause_sync,
            chain_info,
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
                            self.handlers.block_imported.call_simple(&BlockImported {
                                number: *block_import.header.number(),
                                // TODO: This is not pretty that we do it here, but not clear what would be a
                                //  nicer API
                                reward_address_balance: get_total_account_balance(
                                    &client,
                                    block_import.header.hash(),
                                    &reward_address_storage_key,
                                )
                                .unwrap_or_default(),
                            });
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

    pub(super) fn on_sync_state_change(&self, callback: HandlerFn<SyncState>) -> HandlerId {
        self.handlers.sync_state_change.add(callback)
    }

    pub(super) fn on_block_imported(&self, callback: HandlerFn<BlockImported>) -> HandlerId {
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

// defined here: https://github.com/subspace/subspace/blob/5d8b65740ff054b01ebcbaf5a905e74274c1a5d0/crates/subspace-core-primitives/src/pieces.rs#L803
const PIECE_SIZE: usize = 1048672/* the actual piece size from your runtime */;

// set as default
const WS_URL: &str = "ws://127.0.0.1:19944";

// Generate subspace node metadata by running node first.
// And then note down the addr from the terminal log.
// Finally, use the websocket url in this command:
// ```sh
// subxt metadata --url ws://127.0.0.1:19944 > metadata/subspace_metadata.scale
// ```
#[subxt::subxt(runtime_metadata_path = "metadata/subspace_metadata.scale")]
pub mod subspace {}

// Subspace runtime API
type SubspaceApi = subspace::runtime_apis::subspace_api::SubspaceApi;

// Had to define separately as the inferred type differs from the original `sp_consensus_subspace::ChainConstants`.
type RuntimeChainConstants = subspace::runtime_types::sp_consensus_subspace::ChainConstants;

async fn get_current_solution_range(
    api: &OnlineClient<PolkadotConfig>,
    subspace_api: &SubspaceApi,
) -> anyhow::Result<u64> {
    let solution_ranges_runtime_api_call = subspace_api.solution_ranges();

    // get the solution ranges from the latest block
    let current_solution_range = api
        .runtime_api()
        .at_latest()
        .await?
        .call(solution_ranges_runtime_api_call)
        .await?
        .current;

    Ok(current_solution_range)
}

async fn get_slot_probability(
    api: &OnlineClient<PolkadotConfig>,
    subspace_api: &SubspaceApi,
) -> anyhow::Result<(u64, u64)> {
    let chain_constants_runtime_api_call = subspace_api.chain_constants();

    // get the chain constants from the latest block
    let current_chain_constants = api
        .runtime_api()
        .at_latest()
        .await?
        .call(chain_constants_runtime_api_call)
        .await?;

    let RuntimeChainConstants::V0 {
        slot_probability, ..
    } = current_chain_constants;

    Ok(slot_probability)
}

/// Get the latest total space pledged
async fn get_total_space_pledged() -> anyhow::Result<u128> {
    let api = OnlineClient::<PolkadotConfig>::from_url(WS_URL)
        .await
        .map_err(|e| anyhow::Error::new(e).context("Failed to create OnlineClient from WS_URL"))?;

    let subspace_api: SubspaceApi = subspace::apis().subspace_api();

    let current_solution_range = get_current_solution_range(&api, &subspace_api).await?;

    let slot_probability = get_slot_probability(&api, &subspace_api).await?;

    // Calculate the total space pledged
    let total_space_pledged: u128 = u128::from(u64::MAX)
        .checked_mul(PIECE_SIZE as u128)
        .expect("Multiplication with piece size works; qed")
        .checked_mul(u128::from(slot_probability.0))
        .expect("Multiplication with slot probability_0 works; qed")
        .checked_div(u128::from(current_solution_range))
        .expect("Division by current solution range works; qed")
        .checked_div(u128::from(slot_probability.1))
        .expect("Division by slot probability_1 works; qed");

    Ok(total_space_pledged)
}

pub(super) fn load_chain_specification(chain_spec: &'static [u8]) -> Result<ChainSpec, String> {
    GenericChainSpec::<()>::from_json_bytes(chain_spec)
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
            Value::String(s) => Ok(s),
            Value::Null => Ok(String::new()),
            _ => Err(sc_service::Error::Other(
                "Failed to decode PoT initial key".to_string(),
            )),
        })
        .transpose()?;
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
) -> Configuration {
    let telemetry_endpoints = chain_spec.0.telemetry_endpoints().clone();

    let consensus_chain_config = SubstrateConfiguration {
        impl_name: env!("CARGO_PKG_NAME").to_string(),
        impl_version: env!("CARGO_PKG_VERSION").to_string(),
        farmer: true,
        base_path,
        transaction_pool: Default::default(),
        network: SubstrateNetworkConfiguration {
            listen_on: vec![
                sc_network::Multiaddr::from(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
                    .with(sc_network::multiaddr::Protocol::Tcp(substrate_port)),
                sc_network::Multiaddr::from(IpAddr::V6(Ipv6Addr::UNSPECIFIED))
                    .with(sc_network::multiaddr::Protocol::Tcp(substrate_port)),
            ],
            public_addresses: Vec::new(),
            bootstrap_nodes: chain_spec.0.boot_nodes().to_vec(),
            node_key: NodeKeyConfig::Ed25519(Ed25519Secret::Input(
                libp2p_identity_substate::ed25519::SecretKey::try_from_bytes(
                    keypair.secret().as_ref().to_vec(),
                )
                .expect("Correct keypair, just libp2p version is different; qed"),
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
            force_synced: false,
        },
        state_pruning: PruningMode::ArchiveCanonical,
        blocks_pruning: BlocksPruning::Some(256),
        rpc_options: SubstrateRpcConfiguration {
            listen_on: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, RPC_PORT)),
            // Substrate's default
            max_connections: 100,
            // TODO: Replace with `Some(Vec::new())` once node client for farmer is rewritten
            cors: Some(vec![
                "http://localhost:*".to_string(),
                "http://127.0.0.1:*".to_string(),
                "https://localhost:*".to_string(),
                "https://127.0.0.1:*".to_string(),
                "https://polkadot.js.org".to_string(),
            ]),
            methods: Default::default(),
            // Substrate's default
            rate_limit: None,
            max_subscriptions_per_connection: 1024,
            message_buffer_capacity_per_connection: 64,
            disable_batch_requests: false,
            max_batch_request_len: None,
        },
        prometheus_listen_on: None,
        telemetry_endpoints,
        force_authoring: false,
        chain_spec: chain_spec.into(),
        informant_output_format: OutputFormat {
            enable_color: false,
        },
    };

    Configuration::from(consensus_chain_config)
}

pub(super) async fn create_consensus_node(
    keypair: &Keypair,
    base_path: PathBuf,
    substrate_port: u16,
    chain_spec: ChainSpec,
    piece_getter: Arc<dyn DsnSyncPieceGetter + Send + Sync + 'static>,
    node: Node,
    maybe_node_rpc_client: &MaybeNodeRpcClient,
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
    let pause_sync = Arc::clone(&consensus_chain_config.network.pause_sync);

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
            },
            dsn_piece_getter: Some(piece_getter),
            sync_from_dsn: true,
            is_timekeeper: false,
            timekeeper_cpu_cores: Default::default(),
        };

        // TODO: Remove once support for upgrade from Gemini 3g is no longer necessary
        if fs::try_exists(base_path.join("paritydb"))
            .await
            .unwrap_or_default()
        {
            return Err(ConsensusNodeCreationError::IncompatibleChain {
                compatible_chain: consensus_chain_config.base.chain_spec.name().to_string(),
            });
        }

        let partial_components = subspace_service::new_partial::<PosTable, RuntimeApi>(
            &consensus_chain_config.base,
            &pot_external_entropy,
        )
        .map_err(|error| {
            sc_service::Error::Other(format!("Failed to build a full subspace node: {error:?}"))
        })?;

        if hex::encode(partial_components.client.info().genesis_hash) != GENESIS_HASH {
            return Err(ConsensusNodeCreationError::IncompatibleChain {
                compatible_chain: consensus_chain_config.base.chain_spec.name().to_string(),
            });
        }

        subspace_service::new_full::<PosTable, _>(
            consensus_chain_config,
            partial_components,
            None,
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
        base_path,
        &consensus_node.task_manager.spawn_essential_handle(),
    )
    .map_err(|error| {
        sc_service::Error::Other(format!("Failed to start storage monitor: {error:?}"))
    })?;

    let node_client = NodeRpcClient::new(&format!("ws://127.0.0.1:{RPC_PORT}"))
        .await
        .map_err(|error| {
            sc_service::Error::Application(
                format!("Failed to connect to internal node RPC: {error}").into(),
            )
        })?;

    // Inject working node client into wrapper we have created before such that networking can
    // respond to incoming requests properly
    maybe_node_rpc_client.inject(node_client);

    Ok(ConsensusNode::new(consensus_node, pause_sync, chain_info))
}
