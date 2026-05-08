use axum::{Router, routing::get};

use crate::routes;

pub fn create_routes() -> Router<crate::AppState> {
    Router::new()
        .route("/health", get(|| async { "hastebin at your service" }))
        .nest("/paste", routes::paste::router())
        .nest("/comment", routes::comment::router())
}
