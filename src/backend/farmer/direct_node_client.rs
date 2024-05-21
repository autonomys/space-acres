use async_lock::Mutex as AsyncMutex;
use futures::channel::mpsc;
use futures::{future, FutureExt, Stream, StreamExt};
use parity_scale_codec::{Decode, Encode};
use parking_lot::Mutex;
use sc_client_api::{AuxStore, BlockBackend, HeaderBackend};
use sc_consensus_subspace::archiver::{
    recreate_genesis_segment, ArchivedSegmentNotification, SegmentHeadersStore,
};
use sc_consensus_subspace::notification::SubspaceNotificationStream;
use sc_consensus_subspace::slot_worker::{NewSlotNotification, RewardSigningNotification};
use sc_rpc::SubscriptionTaskExecutor;
use sc_utils::mpsc::TracingUnboundedSender;
use schnellru::{ByLength, LruMap};
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_consensus::SyncOracle;
use sp_consensus_subspace::{ChainConstants, FarmerPublicKey, FarmerSignature, SubspaceApi};
use sp_core::crypto::ByteArray;
use sp_core::H256;
use sp_objects::ObjectsApi;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};
use std::time::Duration;
use subspace_archiving::archiver::NewArchivedSegment;
use subspace_core_primitives::crypto::kzg::Kzg;
use subspace_core_primitives::{
    BlockHash, HistorySize, Piece, PieceIndex, PublicKey, SegmentHeader, SegmentIndex, SlotNumber,
    Solution,
};
use subspace_farmer::node_client::{Error, NodeClient, NodeClientExt};
use subspace_farmer_components::FarmerProtocolInfo;
use subspace_networking::libp2p::Multiaddr;
use subspace_rpc_primitives::{
    FarmerAppInfo, RewardSignatureResponse, RewardSigningInfo, SlotInfo, SolutionResponse,
};
use subspace_runtime_primitives::opaque::Block;
use tracing::{debug, error, warn};

/// This is essentially equal to expected number of votes per block, one more is added implicitly by
/// the fact that channel sender exists
const SOLUTION_SENDER_CHANNEL_CAPACITY: usize = 9;
const REWARD_SIGNING_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Default)]
struct BlockSignatureSenders {
    current_hash: H256,
    senders: Vec<async_oneshot::Sender<RewardSignatureResponse>>,
}

/// In-memory cache of last archived segment, such that when request comes back right after
/// archived segment notification, RPC server is able to answer quickly.
///
/// We store weak reference, such that archived segment is not persisted for longer than
/// necessary occupying RAM.
enum CachedArchivedSegment {
    /// Special case for genesis segment when requested over RPC
    Genesis(Arc<NewArchivedSegment>),
    Weak(Weak<NewArchivedSegment>),
}

impl CachedArchivedSegment {
    fn get(&self) -> Option<Arc<NewArchivedSegment>> {
        match self {
            CachedArchivedSegment::Genesis(archived_segment) => Some(Arc::clone(archived_segment)),
            CachedArchivedSegment::Weak(weak_archived_segment) => weak_archived_segment.upgrade(),
        }
    }
}

#[derive(Default)]
struct ArchivedSegmentHeaderAcknowledgementSenders {
    segment_index: SegmentIndex,
    senders: HashMap<u64, TracingUnboundedSender<()>>,
}

/// Configuration for `DirectNodeClient`
pub(in super::super) struct NodeClientConfig<Client> {
    /// Substrate client
    pub client: Arc<Client>,
    /// Task executor that is being used by RPC subscriptions
    pub subscription_executor: SubscriptionTaskExecutor,
    /// New slot notification stream
    pub new_slot_notification_stream: SubspaceNotificationStream<NewSlotNotification>,
    /// Reward signing notification stream
    pub reward_signing_notification_stream: SubspaceNotificationStream<RewardSigningNotification>,
    /// Archived segment notification stream
    pub archived_segment_notification_stream:
        SubspaceNotificationStream<ArchivedSegmentNotification>,
    /// DSN bootstrap nodes
    pub dsn_bootstrap_nodes: Vec<Multiaddr>,
    /// Segment headers store
    pub segment_headers_store: SegmentHeadersStore<Client>,
    /// Subspace sync oracle
    pub sync_oracle: Arc<dyn SyncOracle + Send + Sync>,
    /// Kzg instance
    pub kzg: Kzg,
}

/// Direct node client that is injected into wrapper `MaybeNodeClient` once it is fully initialized
pub(in super::super) struct DirectNodeClient<Client> {
    client: Arc<Client>,
    subscription_executor: SubscriptionTaskExecutor,
    new_slot_notification_stream: SubspaceNotificationStream<NewSlotNotification>,
    reward_signing_notification_stream: SubspaceNotificationStream<RewardSigningNotification>,
    archived_segment_notification_stream: SubspaceNotificationStream<ArchivedSegmentNotification>,
    #[allow(clippy::type_complexity)]
    solution_response_senders:
        Arc<Mutex<LruMap<SlotNumber, mpsc::Sender<Solution<PublicKey, PublicKey>>>>>,
    reward_signature_senders: Arc<Mutex<BlockSignatureSenders>>,
    dsn_bootstrap_nodes: Vec<Multiaddr>,
    segment_headers_store: SegmentHeadersStore<Client>,
    cached_archived_segment: Arc<AsyncMutex<Option<CachedArchivedSegment>>>,
    archived_segment_acknowledgement_senders:
        Arc<Mutex<ArchivedSegmentHeaderAcknowledgementSenders>>,
    next_subscription_id: Arc<AtomicU64>,
    sync_oracle: Arc<dyn SyncOracle + Send + Sync>,
    genesis_hash: BlockHash,
    chain_constants: ChainConstants,
    max_pieces_in_sector: u16,
    kzg: Kzg,
}

impl<Client> Debug for DirectNodeClient<Client> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirectNodeClient").finish()
    }
}

impl<Client> DirectNodeClient<Client>
where
    Client: HeaderBackend<Block> + ProvideRuntimeApi<Block> + BlockBackend<Block> + 'static,
    Client::Api: SubspaceApi<Block, FarmerPublicKey> + ObjectsApi<Block> + 'static,
{
    pub fn new(config: NodeClientConfig<Client>) -> Result<Self, ApiError> {
        let info = config.client.info();
        let best_hash = info.best_hash;
        let genesis_hash = BlockHash::try_from(best_hash.as_ref().to_vec())
            .expect("Genesis hash must always be convertable into BlockHash; qed");
        let runtime_api = config.client.runtime_api();
        let chain_constants = runtime_api.chain_constants(best_hash)?;
        // While the number can technically change in runtime, farmer will not adjust to it on the
        // fly and previous value will remain valid (number only expected to increase), so it is
        // fine to query it only once
        let max_pieces_in_sector = runtime_api.max_pieces_in_sector(best_hash)?;
        let block_authoring_delay = u64::from(chain_constants.block_authoring_delay());
        let block_authoring_delay = usize::try_from(block_authoring_delay)
            .expect("Block authoring delay will never exceed usize on any platform; qed");
        let solution_response_senders_capacity = u32::try_from(block_authoring_delay)
            .expect("Always a tiny constant in the protocol; qed");

        Ok(Self {
            client: config.client,
            subscription_executor: config.subscription_executor,
            new_slot_notification_stream: config.new_slot_notification_stream,
            reward_signing_notification_stream: config.reward_signing_notification_stream,
            archived_segment_notification_stream: config.archived_segment_notification_stream,
            solution_response_senders: Arc::new(Mutex::new(LruMap::new(ByLength::new(
                solution_response_senders_capacity,
            )))),
            reward_signature_senders: Arc::default(),
            dsn_bootstrap_nodes: config.dsn_bootstrap_nodes,
            segment_headers_store: config.segment_headers_store,
            cached_archived_segment: Arc::default(),
            archived_segment_acknowledgement_senders: Arc::default(),
            next_subscription_id: Arc::new(AtomicU64::new(0)),
            sync_oracle: config.sync_oracle,
            genesis_hash,
            max_pieces_in_sector,
            chain_constants,
            kzg: config.kzg,
        })
    }
}

#[async_trait::async_trait]
impl<Client> NodeClient for DirectNodeClient<Client>
where
    Client: ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + BlockBackend<Block>
        + AuxStore
        + Send
        + Sync
        + 'static,
    Client::Api: SubspaceApi<Block, FarmerPublicKey> + ObjectsApi<Block> + 'static,
{
    async fn farmer_app_info(&self) -> Result<FarmerAppInfo, Error> {
        let last_segment_index = self
            .segment_headers_store
            .max_segment_index()
            .unwrap_or(SegmentIndex::ZERO);

        let farmer_app_info: Result<FarmerAppInfo, ApiError> = try {
            let chain_constants = &self.chain_constants;
            let protocol_info = FarmerProtocolInfo {
                history_size: HistorySize::from(last_segment_index),
                max_pieces_in_sector: self.max_pieces_in_sector,
                recent_segments: chain_constants.recent_segments(),
                recent_history_fraction: chain_constants.recent_history_fraction(),
                min_sector_lifetime: chain_constants.min_sector_lifetime(),
            };

            FarmerAppInfo {
                genesis_hash: self.genesis_hash,
                dsn_bootstrap_nodes: self.dsn_bootstrap_nodes.clone(),
                syncing: self.sync_oracle.is_major_syncing(),
                farming_timeout: chain_constants
                    .slot_duration()
                    .as_duration()
                    .mul_f64(SlotNumber::from(chain_constants.block_authoring_delay()) as f64),
                protocol_info,
            }
        };

        farmer_app_info.map_err(|error| {
            error!("Failed to get data from runtime API: {}", error);
            "Internal error".to_string().into()
        })
    }

    async fn submit_solution_response(
        &self,
        solution_response: SolutionResponse,
    ) -> Result<(), Error> {
        let slot = solution_response.slot_number;
        let mut solution_response_senders = self.solution_response_senders.lock();

        let success = solution_response_senders
            .peek_mut(&slot)
            .and_then(|sender| sender.try_send(solution_response.solution).ok())
            .is_some();

        if !success {
            warn!(
                %slot,
                "Solution was ignored, likely because farmer was too slow"
            );

            return Err("Solution was ignored".to_string().into());
        }

        Ok(())
    }
    async fn subscribe_slot_info(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = SlotInfo> + Send + 'static>>, Error> {
        let executor = self.subscription_executor.clone();
        let solution_response_senders = self.solution_response_senders.clone();

        let handle_slot_notification = move |new_slot_notification| {
            let NewSlotNotification {
                new_slot_info,
                mut solution_sender,
            } = new_slot_notification;

            let slot_number = SlotNumber::from(new_slot_info.slot);

            // Store solution sender so that we can retrieve it when solution comes from
            // the farmer
            let mut solution_response_senders = solution_response_senders.lock();
            if solution_response_senders.peek(&slot_number).is_none() {
                let (response_sender, mut response_receiver) =
                    mpsc::channel(SOLUTION_SENDER_CHANNEL_CAPACITY);

                solution_response_senders.insert(slot_number, response_sender);

                // Wait for solutions and transform proposed proof of space solutions
                // into data structure `sc-consensus-subspace` expects
                let forward_solution_fut = async move {
                    while let Some(solution) = response_receiver.next().await {
                        let public_key = FarmerPublicKey::from_slice(solution.public_key.as_ref())
                            .expect("Always correct length; qed");
                        let reward_address =
                            FarmerPublicKey::from_slice(solution.reward_address.as_ref())
                                .expect("Always correct length; qed");

                        let sector_index = solution.sector_index;

                        let solution = Solution {
                            public_key: public_key.clone(),
                            reward_address,
                            sector_index,
                            history_size: solution.history_size,
                            piece_offset: solution.piece_offset,
                            record_commitment: solution.record_commitment,
                            record_witness: solution.record_witness,
                            chunk: solution.chunk,
                            chunk_witness: solution.chunk_witness,
                            proof_of_space: solution.proof_of_space,
                        };

                        if solution_sender.try_send(solution).is_err() {
                            warn!(
                                slot = %slot_number,
                                %sector_index,
                                %public_key,
                                "Solution receiver is closed, likely because farmer was too slow"
                            );
                        }
                    }
                };

                executor.spawn(
                    "subspace-slot-info-forward",
                    Some("rpc"),
                    Box::pin(forward_solution_fut),
                );
            }

            let global_challenge = new_slot_info
                .proof_of_time
                .derive_global_randomness()
                .derive_global_challenge(slot_number);

            // This will be sent to the farmer
            SlotInfo {
                slot_number,
                global_challenge,
                solution_range: new_slot_info.solution_range,
                voting_solution_range: new_slot_info.voting_solution_range,
            }
        };
        let stream = self
            .new_slot_notification_stream
            .subscribe()
            .map(handle_slot_notification);

        Ok(Box::pin(stream))
    }

    async fn subscribe_reward_signing(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = RewardSigningInfo> + Send + 'static>>, Error> {
        let executor = self.subscription_executor.clone();
        let reward_signature_senders = self.reward_signature_senders.clone();

        let stream = self.reward_signing_notification_stream.subscribe().map(
            move |reward_signing_notification| {
                let RewardSigningNotification {
                    hash,
                    public_key,
                    signature_sender,
                } = reward_signing_notification;

                let (response_sender, response_receiver) = async_oneshot::oneshot();

                // Store signature sender so that we can retrieve it when solution comes from
                // the farmer
                {
                    let mut reward_signature_senders = reward_signature_senders.lock();

                    if reward_signature_senders.current_hash != hash {
                        reward_signature_senders.current_hash = hash;
                        reward_signature_senders.senders.clear();
                    }

                    reward_signature_senders.senders.push(response_sender);
                }

                // Wait for solutions and transform proposed proof of space solutions into
                // data structure `sc-consensus-subspace` expects
                let forward_signature_fut = async move {
                    if let Ok(reward_signature) = response_receiver.await {
                        if let Some(signature) = reward_signature.signature {
                            match FarmerSignature::decode(&mut signature.encode().as_ref()) {
                                Ok(signature) => {
                                    let _ = signature_sender.unbounded_send(signature);
                                }
                                Err(error) => {
                                    warn!(
                                        "Failed to convert signature of length {}: {}",
                                        signature.len(),
                                        error
                                    );
                                }
                            }
                        }
                    }
                };

                // Run above future with timeout
                executor.spawn(
                    "subspace-block-signing-forward",
                    Some("rpc"),
                    future::select(
                        futures_timer::Delay::new(REWARD_SIGNING_TIMEOUT),
                        Box::pin(forward_signature_fut),
                    )
                    .map(|_| ())
                    .boxed(),
                );

                // This will be sent to the farmer
                RewardSigningInfo {
                    hash: hash.into(),
                    public_key: public_key
                        .as_slice()
                        .try_into()
                        .expect("Public key is always 32 bytes; qed"),
                }
            },
        );

        Ok(Box::pin(stream))
    }

    async fn submit_reward_signature(
        &self,
        reward_signature: RewardSignatureResponse,
    ) -> Result<(), Error> {
        let reward_signature_senders = self.reward_signature_senders.clone();

        // TODO: This doesn't track what client sent a solution, allowing some clients to send
        //  multiple (https://github.com/paritytech/jsonrpsee/issues/452)
        let mut reward_signature_senders = reward_signature_senders.lock();

        if reward_signature_senders.current_hash == reward_signature.hash.into() {
            if let Some(mut sender) = reward_signature_senders.senders.pop() {
                let _ = sender.send(reward_signature);
            }
        }

        Ok(())
    }

    async fn subscribe_archived_segment_headers(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = SegmentHeader> + Send + 'static>>, Error> {
        let archived_segment_acknowledgement_senders =
            self.archived_segment_acknowledgement_senders.clone();

        let cached_archived_segment = Arc::clone(&self.cached_archived_segment);
        let subscription_id = self.next_subscription_id.fetch_add(1, Ordering::Relaxed);

        let stream = self
            .archived_segment_notification_stream
            .subscribe()
            .filter_map(move |archived_segment_notification| {
                let ArchivedSegmentNotification {
                    archived_segment,
                    acknowledgement_sender,
                } = archived_segment_notification;

                let segment_index = archived_segment.segment_header.segment_index();
                let cached_archived_segment_clone = Arc::clone(&cached_archived_segment);

                // Store acknowledgment sender so that we can retrieve it when acknowledgement
                // comes from the farmer, but only if unsafe APIs are allowed
                let maybe_archived_segment_header = {
                    let mut archived_segment_acknowledgement_senders =
                        archived_segment_acknowledgement_senders.lock();

                    if archived_segment_acknowledgement_senders.segment_index != segment_index {
                        archived_segment_acknowledgement_senders.segment_index = segment_index;
                        archived_segment_acknowledgement_senders.senders.clear();
                    }

                    let maybe_archived_segment_header =
                        match archived_segment_acknowledgement_senders
                            .senders
                            .entry(subscription_id)
                        {
                            Entry::Occupied(_) => {
                                // No need to do anything, farmer is processing request
                                None
                            }
                            Entry::Vacant(entry) => {
                                entry.insert(acknowledgement_sender);

                                // This will be sent to the farmer
                                Some(archived_segment.segment_header)
                            }
                        };

                    maybe_archived_segment_header
                };

                Box::pin(async move {
                    cached_archived_segment_clone.lock().await.replace(
                        CachedArchivedSegment::Weak(Arc::downgrade(&archived_segment)),
                    );

                    maybe_archived_segment_header
                })
            });

        let archived_segment_acknowledgement_senders =
            self.archived_segment_acknowledgement_senders.clone();
        let fut = async move {
            let mut archived_segment_acknowledgement_senders =
                archived_segment_acknowledgement_senders.lock();

            archived_segment_acknowledgement_senders
                .senders
                .remove(&subscription_id);
        };

        self.subscription_executor.spawn(
            "subspace-archived-segment-header-subscription",
            Some("rpc"),
            fut.boxed(),
        );

        Ok(Box::pin(stream))
    }

    async fn acknowledge_archived_segment_header(
        &self,
        segment_index: SegmentIndex,
    ) -> Result<(), Error> {
        let archived_segment_acknowledgement_senders =
            self.archived_segment_acknowledgement_senders.clone();

        let maybe_sender = {
            let mut archived_segment_acknowledgement_senders_guard =
                archived_segment_acknowledgement_senders.lock();

            (archived_segment_acknowledgement_senders_guard.segment_index == segment_index)
                .then(|| {
                    let last_key = *archived_segment_acknowledgement_senders_guard
                        .senders
                        .keys()
                        .next()?;

                    archived_segment_acknowledgement_senders_guard
                        .senders
                        .remove(&last_key)
                })
                .flatten()
        };

        if let Some(sender) = maybe_sender {
            if let Err(error) = sender.unbounded_send(()) {
                if !error.is_closed() {
                    warn!("Failed to acknowledge archived segment: {error}");
                }
            }
        }

        debug!(%segment_index, "Acknowledged archived segment.");

        Ok(())
    }

    async fn piece(&self, piece_index: PieceIndex) -> Result<Option<Piece>, Error> {
        let archived_segment = {
            let mut cached_archived_segment = self.cached_archived_segment.lock().await;

            match cached_archived_segment
                .as_ref()
                .and_then(CachedArchivedSegment::get)
            {
                Some(archived_segment) => archived_segment,
                None => {
                    if piece_index > SegmentIndex::ZERO.last_piece_index() {
                        return Ok(None);
                    }

                    debug!(%piece_index, "Re-creating genesis segment on demand");

                    let client = Arc::clone(&self.client);
                    let kzg = self.kzg.clone();
                    // Try to re-create genesis segment on demand
                    match tokio::task::spawn_blocking(move || {
                        recreate_genesis_segment(&*client, kzg).map_err(|error| {
                            format!("Failed to re-create genesis segment: {}", error)
                        })
                    })
                    .await
                    {
                        Ok(Ok(Some(archived_segment))) => {
                            let archived_segment = Arc::new(archived_segment);
                            cached_archived_segment.replace(CachedArchivedSegment::Genesis(
                                Arc::clone(&archived_segment),
                            ));
                            archived_segment
                        }
                        Ok(Ok(None)) => {
                            return Ok(None);
                        }
                        Ok(Err(error)) => {
                            error!(%error, "Failed to re-create genesis segment");

                            return Err("Failed to re-create genesis segment".to_string().into());
                        }
                        Err(error) => {
                            error!(%error, "Blocking task failed to re-create genesis segment");

                            return Err("Blocking task failed to re-create genesis segment"
                                .to_string()
                                .into());
                        }
                    }
                }
            }
        };

        let pieces = &archived_segment.pieces;
        if piece_index.segment_index() == archived_segment.segment_header.segment_index() {
            return Ok(Some((&pieces[piece_index.position() as usize]).into()));
        }

        Ok(None)
    }

    async fn segment_headers(
        &self,
        segment_indexes: Vec<SegmentIndex>,
    ) -> Result<Vec<Option<SegmentHeader>>, Error> {
        Ok(segment_indexes
            .into_iter()
            .map(|segment_index| self.segment_headers_store.get_segment_header(segment_index))
            .collect())
    }
}

#[async_trait::async_trait]
impl<Client> NodeClientExt for DirectNodeClient<Client>
where
    Client: ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + BlockBackend<Block>
        + AuxStore
        + Send
        + Sync
        + 'static,
    Client::Api: SubspaceApi<Block, FarmerPublicKey> + ObjectsApi<Block> + 'static,
{
    async fn last_segment_headers(&self, limit: u64) -> Result<Vec<Option<SegmentHeader>>, Error> {
        let last_segment_index = self
            .segment_headers_store
            .max_segment_index()
            .unwrap_or(SegmentIndex::ZERO);

        let last_segment_headers = (SegmentIndex::ZERO..=last_segment_index)
            .rev()
            .take(limit as usize)
            .map(|segment_index| self.segment_headers_store.get_segment_header(segment_index))
            .collect::<Vec<_>>();

        Ok(last_segment_headers)
    }
}
