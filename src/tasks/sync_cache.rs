use api::{Config, PasteCache};
use sqlx::PgPool;
use tokio::time::{Duration, interval};

pub fn spawn_cache_sync_task(pool: PgPool, cache: PasteCache, config: Config) {
    println!(
        "spawning cache sync task (interval: {}s)",
        config.cache_sync_interval
    );
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(config.cache_sync_interval));

        loop {
            interval.tick().await;
            println!("synchronising cache views to database");
            cache.synchronise(&pool).await;
        }
    });
}
