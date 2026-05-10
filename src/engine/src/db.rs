use sqlx::{PgPool, postgres::PgPoolOptions};

// but i wonder, should this be inside the engine itself?
// maybe can be somewhere better
pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    Ok(pool)
}
