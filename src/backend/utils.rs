use event_listener_primitives::Bag;
use std::sync::Arc;

pub(super) type HandlerFn<A> = Arc<dyn Fn(&A) + Send + Sync + 'static>;
pub(super) type Handler<A> = Bag<HandlerFn<A>, A>;
