use std::{env, net::SocketAddr};

use axum::{extract::{FromRef, State}, http::StatusCode, response::{IntoResponse, Response}, routing::{get, post}, Router};
use dotenv::dotenv;
use tower_http::cors::{Any, CorsLayer};
use sqlx::{postgres::{PgPoolOptions, PgRow}, PgPool, Pool, Postgres, Row};

mod utils;
mod extractor_error;

mod paste;

pub async fn not_implemented_yet() -> Response {
    (StatusCode::NOT_IMPLEMENTED, "not implemented yet chill".to_string()).into_response()
}

pub async fn testing(
    State(app_state): State<AppState>
) -> String {
    format!("{:?}", 
        sqlx::query("select * from hastebin.user;")
            .fetch_all(&app_state.db)
            .await.unwrap()
    )
}

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db: Pool<Postgres>
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let app = Router::new()
        .route("/", get(|| async { "hastebin at your service" }))

        .route("/fetch/:id", get(paste::fetch))
        .route("/create", post(paste::create))

        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_origin(Any)
                .allow_headers(Any)
        )

        .with_state(
            AppState {
                db: PgPool::connect(env::var("PG_ADDRESS").unwrap().to_string().replace("[YOUR-PASSWORD]", env::var("PG_PASSWORD").unwrap().as_str()).to_string().as_str()).await.unwrap()
            }
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8521").await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
