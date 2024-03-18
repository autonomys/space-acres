use crate::backend::node::SLOT_PROBABILITY;
use event_listener_primitives::Bag;
use std::sync::Arc;
use subspace_core_primitives::{Record, SolutionRange};

pub(super) type HandlerFn<A> = Arc<dyn Fn(&A) + Send + Sync + 'static>;
pub(super) type Handler<A> = Bag<HandlerFn<A>, A>;

// defined as constant in `subspace-farmer-components`, `subspace-runtime`.
// https://github.com/subspace/subspace/blob/df8d33b65fff6a88d77fa8090533879199bcb422/crates/subspace-runtime/src/lib.rs#L104
pub(crate) const MAX_PIECES_IN_SECTOR: u16 = 1000;

/// Computes the following:
/// ```
/// MAX * slot_probability / (pieces_in_sector * chunks / s_buckets) / solution_range
/// ```
pub(crate) const fn solution_range_to_sectors(solution_range: SolutionRange) -> u64 {
    let sectors = SolutionRange::MAX
        // Account for slot probability
        / SLOT_PROBABILITY.1 * SLOT_PROBABILITY.0
        // Now take sector size and probability of hitting occupied s-bucket in sector into account
        / (MAX_PIECES_IN_SECTOR as u64 * Record::NUM_CHUNKS as u64 / Record::NUM_S_BUCKETS as u64);

    // Take solution range into account
    sectors / solution_range
}
