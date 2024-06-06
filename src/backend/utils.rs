use event_listener_primitives::Bag;
use std::sync::Arc;

pub(super) type HandlerFn<A> = Arc<dyn Fn(&A) + Send + Sync + 'static>;
pub(super) type Handler<A> = Bag<HandlerFn<A>, A>;

// TODO: Expose this via chain constants.
// https://github.com/subspace/subspace/blob/e1322171af8eb2440ae8ab9e28c7e22ef42f6f6c/crates/subspace-runtime/src/lib.rs#L186
pub(super) const EXPECTED_VOTES_PER_BLOCK: u32 = 9;
