pub mod cleanup;

use api::Config;
use sqlx::PgPool;

// just like how we do it in routing
pub fn spawn_all_tasks(pool: PgPool, config: Config) {
    cleanup::spawn_cleanup_task(pool, config);
}
