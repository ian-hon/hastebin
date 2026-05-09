use serde::Deserialize;
use sqlx::{Pool, Postgres};

mod handlers;
pub mod router;
mod routes;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub port: u16,

    // the expiry applies only to pastes over the limit
    pub size_soft_limit: usize, // in bytes
    pub default_expiry_days: u32,
}

#[derive(Clone)]
pub struct AppState {
    // in a perfect world, we would do the same for the pool,
    // but differing providers have differing apis
    pub db: Pool<Postgres>,
    pub config: Config,
    // consider caching pastes and comments
}

impl AppState {
    pub fn new(db: Pool<Postgres>, config: Config) -> Self {
        Self { db, config }
    }
}
