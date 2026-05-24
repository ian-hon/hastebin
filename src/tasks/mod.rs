pub mod cleanup;
pub mod sync_cache;

use api::{Config, PasteCache};
use sqlx::PgPool;

// just like how we do it in routing
pub fn spawn_all_tasks(pool: PgPool, cache: PasteCache, config: Config) {
    cleanup::spawn_cleanup_task(pool.clone(), config.clone());
    sync_cache::spawn_cache_sync_task(pool, cache, config);
}
