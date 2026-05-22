use api::Config;
use sqlx::PgPool;
use tokio::time::{Duration, interval};

// cleanup task to delete
pub fn spawn_cleanup_task(pool: PgPool, config: Config) {
    println!("SPAWNING TASK");
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(config.cleanup_interval));

        loop {
            println!("TRYING TO CLEANUP");
            match engine::cleanup::run_cleanup(&pool).await {
                Ok(stats) => {
                    println!("CLEANUP");
                    println!("Time : {}", stats.timestamp);
                    println!("Pastes deleted: {}", stats.pastes_deleted);
                    // println!("Total bytes freed: {}", stats.total_bytes_freed);
                    // note this isnt the actual size, because of double compression
                    println!(
                        "Estimated size freed: {}",
                        human_bytes::human_bytes(stats.total_bytes_freed as f64)
                    );
                    println!(
                        "Total size: {}",
                        human_bytes::human_bytes(stats.total_bytes as f64)
                    )
                }
                Err(e) => {
                    // jialat lo
                    eprintln!("Error during cleanup: {:?}", e);
                }
            }

            interval.tick().await;
        }
    });
}
