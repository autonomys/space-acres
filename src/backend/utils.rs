use event_listener_primitives::Bag;
use std::sync::Arc;

pub(super) type HandlerFn<A> = Arc<dyn Fn(&A) + Send + Sync + 'static>;
pub(super) type Handler<A> = Bag<HandlerFn<A>, A>;
pub(super) type Handler2Fn<A, B> = Arc<dyn Fn(&A, &B) + Send + Sync + 'static>;
pub(super) type Handler2<A, B> = Bag<Handler2Fn<A, B>, A, B>;
