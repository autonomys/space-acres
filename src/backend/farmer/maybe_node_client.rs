use arc_swap::ArcSwapOption;
use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;
use subspace_core_primitives::SegmentHeader;
use subspace_farmer::node_client::{Error, NodeClientExt};
use subspace_farmer::{NodeClient, NodeRpcClient};
use subspace_rpc_primitives::{
    FarmerAppInfo, RewardSignatureResponse, RewardSigningInfo, SlotInfo, SolutionResponse,
};

// TODO: Replace RPC client with a client that can work with node directly
/// Wrapper node client that allows injecting real inner node RPC client after construction
#[derive(Debug, Clone, Default)]
pub(in super::super) struct MaybeNodeRpcClient {
    inner: Arc<ArcSwapOption<NodeRpcClient>>,
}

#[async_trait::async_trait]
impl NodeClient for MaybeNodeRpcClient {
    async fn farmer_app_info(&self) -> Result<FarmerAppInfo, Error> {
        match &*self.inner.load() {
            Some(inner) => inner.farmer_app_info().await,
            None => Err("Inner node client not injected yet".into()),
        }
    }

    async fn subscribe_slot_info(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = SlotInfo> + Send + 'static>>, Error> {
        match &*self.inner.load() {
            Some(inner) => inner.subscribe_slot_info().await,
            None => Err("Inner node client not injected yet".into()),
        }
    }

    async fn submit_solution_response(
        &self,
        solution_response: SolutionResponse,
    ) -> Result<(), Error> {
        match &*self.inner.load() {
            Some(inner) => inner.submit_solution_response(solution_response).await,
            None => Err("Inner node client not injected yet".into()),
        }
    }

    async fn subscribe_reward_signing(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = RewardSigningInfo> + Send + 'static>>, Error> {
        match &*self.inner.load() {
            Some(inner) => inner.subscribe_reward_signing().await,
            None => Err("Inner node client not injected yet".into()),
        }
    }

    async fn submit_reward_signature(
        &self,
        reward_signature: RewardSignatureResponse,
    ) -> Result<(), Error> {
        match &*self.inner.load() {
            Some(inner) => inner.submit_reward_signature(reward_signature).await,
            None => Err("Inner node client not injected yet".into()),
        }
    }

    async fn subscribe_archived_segment_headers(
        &self,
    ) -> Result<
        Pin<Box<dyn Stream<Item = subspace_core_primitives::SegmentHeader> + Send + 'static>>,
        Error,
    > {
        match &*self.inner.load() {
            Some(inner) => inner.subscribe_archived_segment_headers().await,
            None => Err("Inner node client not injected yet".into()),
        }
    }

    async fn segment_headers(
        &self,
        segment_indexes: Vec<subspace_core_primitives::SegmentIndex>,
    ) -> Result<Vec<Option<subspace_core_primitives::SegmentHeader>>, Error> {
        match &*self.inner.load() {
            Some(inner) => inner.segment_headers(segment_indexes).await,
            None => Err("Inner node client not injected yet".into()),
        }
    }

    async fn piece(
        &self,
        piece_index: subspace_core_primitives::PieceIndex,
    ) -> Result<Option<subspace_core_primitives::Piece>, Error> {
        match &*self.inner.load() {
            Some(inner) => inner.piece(piece_index).await,
            None => Err("Inner node client not injected yet".into()),
        }
    }

    async fn acknowledge_archived_segment_header(
        &self,
        segment_index: subspace_core_primitives::SegmentIndex,
    ) -> Result<(), Error> {
        match &*self.inner.load() {
            Some(inner) => {
                inner
                    .acknowledge_archived_segment_header(segment_index)
                    .await
            }
            None => Err("Inner node client not injected yet".into()),
        }
    }
}

#[async_trait::async_trait]
impl NodeClientExt for MaybeNodeRpcClient {
    async fn last_segment_headers(&self, limit: u64) -> Result<Vec<Option<SegmentHeader>>, Error> {
        match &*self.inner.load() {
            Some(inner) => inner.last_segment_headers(limit).await,
            None => Err("Inner node client not injected yet".into()),
        }
    }
}

impl MaybeNodeRpcClient {
    pub(in super::super) fn inject(&self, inner: NodeRpcClient) {
        self.inner.store(Some(Arc::new(inner)))
    }
}
