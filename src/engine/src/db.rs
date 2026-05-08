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

#[warn(dead_code)]
pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    // stolen lol
    // we dont use migrations so consider removing
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
