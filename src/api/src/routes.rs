use axum::{Router, routing::get};

// 'static is to guarantee config lives forever
// Clone because AppState needs to be cloneable
//      if not clonable, AppState must be put into a RwLock, Mutex or ArcLock to prevent race conditions
//      so why not those? because its a pastebin app not a payment gateway lol
// Send + Sync are thread-safety imposed
pub fn create_routes<C: 'static + Clone + Send + Sync>() -> Router<crate::AppState<C>> {
    Router::new().route("/", get(|| async { "hastebin at your service" }))
}
