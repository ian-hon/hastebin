use serde::Deserialize;
use sqlx::{Pool, Postgres};

mod cache;
mod handlers;
pub mod router;
mod routes;

pub use cache::PasteCache;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub port: u16,

    // the expiry applies only to pastes over the limit
    pub size_soft_limit: usize, // in bytes
    pub default_expiry_days: u32,

    // how often to delete expired pastes, in seconds
    pub cleanup_interval: u64,

    // number of pastes to cache in memory
    pub cache_size: usize,
    // how often to sync cache and db, in seconds
    pub cache_sync_interval: u64,
}

#[derive(Clone)]
pub struct AppState {
    // in a perfect world, we would do the same for the pool,
    // but differing providers have differing apis
    pub db: Pool<Postgres>,
    pub config: Config,
    pub cache: PasteCache,
}

impl AppState {
    pub fn new(db: Pool<Postgres>, cache: PasteCache, config: Config) -> Self {
        Self { db, config, cache }
    }
}
