use arc_swap::ArcSwapOption;
use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;
use subspace_core_primitives::pieces::{Piece, PieceIndex};
use subspace_core_primitives::segments::{SegmentHeader, SegmentIndex};
use subspace_farmer::node_client::{NodeClient, NodeClientExt};
use subspace_rpc_primitives::{
    FarmerAppInfo, RewardSignatureResponse, RewardSigningInfo, SlotInfo, SolutionResponse,
};

/// Wrapper node client that allows injecting real inner `NodeClientExt` implementation.
#[derive(Debug, Clone, Default)]
pub(in super::super) struct MaybeNodeClient {
    inner: Arc<ArcSwapOption<Box<dyn NodeClientExt>>>,
}

#[async_trait::async_trait]
impl NodeClient for MaybeNodeClient {
    async fn farmer_app_info(&self) -> anyhow::Result<FarmerAppInfo> {
        match &*self.inner.load() {
            Some(inner) => inner.farmer_app_info().await,
            None => Err(anyhow::anyhow!("Inner node client not injected yet")),
        }
    }

    async fn subscribe_slot_info(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = SlotInfo> + Send + 'static>>> {
        match &*self.inner.load() {
            Some(inner) => inner.subscribe_slot_info().await,
            None => Err(anyhow::anyhow!("Inner node client not injected yet")),
        }
    }

    async fn submit_solution_response(
        &self,
        solution_response: SolutionResponse,
    ) -> anyhow::Result<()> {
        match &*self.inner.load() {
            Some(inner) => inner.submit_solution_response(solution_response).await,
            None => Err(anyhow::anyhow!("Inner node client not injected yet")),
        }
    }

    async fn subscribe_reward_signing(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = RewardSigningInfo> + Send + 'static>>> {
        match &*self.inner.load() {
            Some(inner) => inner.subscribe_reward_signing().await,
            None => Err(anyhow::anyhow!("Inner node client not injected yet")),
        }
    }

    async fn submit_reward_signature(
        &self,
        reward_signature: RewardSignatureResponse,
    ) -> anyhow::Result<()> {
        match &*self.inner.load() {
            Some(inner) => inner.submit_reward_signature(reward_signature).await,
            None => Err(anyhow::anyhow!("Inner node client not injected yet")),
        }
    }

    async fn subscribe_archived_segment_headers(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = SegmentHeader> + Send + 'static>>> {
        match &*self.inner.load() {
            Some(inner) => inner.subscribe_archived_segment_headers().await,
            None => Err(anyhow::anyhow!("Inner node client not injected yet")),
        }
    }

    async fn segment_headers(
        &self,
        segment_indexes: Vec<SegmentIndex>,
    ) -> anyhow::Result<Vec<Option<SegmentHeader>>> {
        match &*self.inner.load() {
            Some(inner) => inner.segment_headers(segment_indexes).await,
            None => Err(anyhow::anyhow!("Inner node client not injected yet")),
        }
    }

    async fn piece(&self, piece_index: PieceIndex) -> anyhow::Result<Option<Piece>> {
        match &*self.inner.load() {
            Some(inner) => inner.piece(piece_index).await,
            None => Err(anyhow::anyhow!("Inner node client not injected yet")),
        }
    }

    async fn acknowledge_archived_segment_header(
        &self,
        segment_index: SegmentIndex,
    ) -> anyhow::Result<()> {
        match &*self.inner.load() {
            Some(inner) => {
                inner
                    .acknowledge_archived_segment_header(segment_index)
                    .await
            }
            None => Err(anyhow::anyhow!("Inner node client not injected yet")),
        }
    }
}

#[async_trait::async_trait]
impl NodeClientExt for MaybeNodeClient {
    async fn cached_segment_headers(
        &self,
        segment_indexes: Vec<SegmentIndex>,
    ) -> anyhow::Result<Vec<Option<SegmentHeader>>> {
        match &*self.inner.load() {
            Some(inner) => inner.cached_segment_headers(segment_indexes).await,
            None => Err(anyhow::anyhow!("Inner node client not injected yet")),
        }
    }

    async fn last_segment_headers(&self, limit: u32) -> anyhow::Result<Vec<Option<SegmentHeader>>> {
        match &*self.inner.load() {
            Some(inner) => inner.last_segment_headers(limit).await,
            None => Err(anyhow::anyhow!("Inner node client not injected yet")),
        }
    }
}

impl MaybeNodeClient {
    pub(in super::super) fn inject(&self, inner: Box<dyn NodeClientExt>) {
        self.inner.store(Some(Arc::new(inner)))
    }
}
