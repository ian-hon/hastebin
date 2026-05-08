use axum::Router;
use std::net::SocketAddr;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;

// mod api;
// mod config;
// mod db;
// mod models;
// mod services;
// mod error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env()?;
    // if the db was locally hosted, we would be doing migrations here
    let db_pool = engine::db::create_pool(&config.database_url).await?;

    let app_state = api::AppState::new(db_pool, config.clone());

    let app = Router::new()
        .merge(api::routes::create_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // host on localhost
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    tracing::info!("hastebin on address: {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
