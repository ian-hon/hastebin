use axum::{Json, Router, http::header, response::IntoResponse, routing::get};

use crate::routes;

async fn openapi_spec() -> Json<serde_json::Value> {
    // need to canonicalise these?
    let spec = include_str!("../../../openapi.json");
    let json: serde_json::Value = serde_json::from_str(spec).unwrap();
    Json(json)
}

async fn ai_plugin() -> Json<serde_json::Value> {
    let plugin = include_str!("../../../.well-known/ai-plugin.json");
    let json: serde_json::Value = serde_json::from_str(plugin).unwrap();
    Json(json)
}

async fn ai_instructions() -> impl IntoResponse {
    let instructions = include_str!("../../../docs/ai/README.md");
    (
        [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
        instructions,
    )
}

pub fn create_routes() -> Router<crate::AppState> {
    Router::new()
        .route("/health", get(|| async { "hastebin at your service" }))
        .route("/openapi.json", get(openapi_spec))
        .route("/.well-known/ai-plugin.json", get(ai_plugin))
        .route("/ai-instructions", get(ai_instructions))
        .nest("/paste", routes::paste::router())
        .nest("/comment", routes::comment::router())
}
