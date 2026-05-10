use axum::Router;
use std::net::SocketAddr;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::from_env()?;

    let db_pool = engine::db::create_pool(&config.database_url).await?;
    // sqlx::migrate!("./migrations").run(&db_pool).await?;

    let app_state = api::AppState::new(db_pool, config.clone());

    let app = Router::new()
        // https://www.reddit.com/r/node/comments/bol0fq/comment/enhhc3k
        .merge(api::router::create_routes())
        // https://docs.rs/tower-http/latest/tower_http/trace/struct.TraceLayer.html#method.new_for_http
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new()) // TODO: benchmark
        // https://docs.rs/tower-http/latest/src/tower_http/cors/mod.rs.html#151
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // host on localhost
    // https://doc.rust-lang.org/std/net/enum.SocketAddr.html#examples
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    tracing::info!("hastebin on address: {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
