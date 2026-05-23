use sqlx::PgPool;
use std::sync::OnceLock;

static ENV_LOADED: OnceLock<()> = OnceLock::new();

/// Loads `.env.test` from the workspace root exactly once per test process.
///
/// Uses `env!("CARGO_MANIFEST_DIR")` so the path is correct regardless of the
/// working directory the test binary is launched from.
pub fn load_test_env() {
    ENV_LOADED.get_or_init(|| {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let env_path = manifest_dir.join("../../.env.test");
        let canonical = std::fs::canonicalize(&env_path).unwrap_or_else(|e| {
            panic!(
                "cannot resolve .env.test path `{}`: {e}",
                env_path.display()
            )
        });
        let file = std::fs::File::open(&canonical)
            .unwrap_or_else(|e| panic!("cannot open `{}`: {e}", canonical.display()));
        dotenvy::from_read_override(file)
            .unwrap_or_else(|e| panic!("failed to parse `{}`: {e}", canonical.display()));
    });
}

/// Runs database migrations from `/migrations/` directory.
///
/// This is called once per test process to ensure the schema is set up.
async fn run_migrations_once(pool: &PgPool) {
    // Check if migrations have been run using a simple query
    // If _sqlx_migrations table exists and has entries, skip
    let already_run = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = '_sqlx_migrations'",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    if already_run > 0 {
        return;
    }

    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .expect("failed to run migrations on test database");
}

/// Creates a connection pool pointed at the test database and runs migrations.
pub async fn create_test_pool() -> PgPool {
    load_test_env();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env.test");

    // Create database if it doesn't exist
    let db_name = url.split('/').last().unwrap_or("hastebin_test");
    let base_url = url.rsplitn(2, '/').nth(1).unwrap_or(&url);

    // Connect to postgres database to create our test database
    let postgres_url = format!("{}/postgres", base_url);
    if let Ok(pg_pool) = sqlx::PgPool::connect(&postgres_url).await {
        let _ = sqlx::query(&format!("CREATE DATABASE {}", db_name))
            .execute(&pg_pool)
            .await;
        pg_pool.close().await;
    }

    // Now connect to test database
    let pool = engine::db::create_pool(&url)
        .await
        .expect("failed to connect to test database");

    // Run migrations once
    run_migrations_once(&pool).await;

    pool
}

/// Truncates all application tables and resets their sequences.
///
/// `comment` is truncated first to satisfy the FK constraint from comment → paste.
pub async fn clear_tables(pool: &PgPool) {
    sqlx::query("TRUNCATE TABLE comment RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("failed to truncate comment table");

    sqlx::query("TRUNCATE TABLE paste RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("failed to truncate paste table");
}

/// Reasonable test defaults that mirror the values in `.env.test`.
pub const SOFT_LIMIT: usize = 50_000;
pub const DEFAULT_EXPIRY_DAYS: u32 = 7;
