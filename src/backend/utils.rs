use event_listener_primitives::Bag;
use std::sync::Arc;
use subspace_core_primitives::{Record, SolutionRange};

pub(super) type HandlerFn<A> = Arc<dyn Fn(&A) + Send + Sync + 'static>;
pub(super) type Handler<A> = Bag<HandlerFn<A>, A>;

// TODO: pointing to source code: https://github.com/subspace/subspace/blob/df8d33b65fff6a88d77fa8090533879199bcb422/crates/subspace-runtime/src/lib.rs#L222-L235
//      Needs to be de-duplicated
/// Computes the following: https://github.com/subspace/subspace/blob/df8d33b65fff6a88d77fa8090533879199bcb422/crates/subspace-runtime/src/lib.rs#L222-L235
/// ```
/// MAX * slot_probability / (pieces_in_sector * chunks / s_buckets) / solution_range
/// ```
#[warn(dead_code)]
pub(super) const fn solution_range_to_sectors(
    solution_range: SolutionRange,
    slot_probability: (u64, u64),
    max_pieces_in_sector: u16,
) -> u64 {
    let sectors = SolutionRange::MAX
        // Account for slot probability
        / slot_probability.1 * slot_probability.0
        // Now take sector size and probability of hitting occupied s-bucket in sector into account
        / (max_pieces_in_sector as u64 * Record::NUM_CHUNKS as u64 / Record::NUM_S_BUCKETS as u64);

    // Take solution range into account
    sectors / solution_range
}
