use crate::utils;
use sqlx::{Pool, Postgres, Row};

// statistics
#[derive(Debug)]
pub struct CleanupStats {
    pub pastes_deleted: i64,
    pub total_bytes_freed: i64,
    pub total_bytes: i64,
    pub timestamp: i64,
}

impl CleanupStats {
    pub fn new(pastes_deleted: i64, total_bytes_freed: i64, total_bytes: i64) -> Self {
        Self {
            pastes_deleted,
            total_bytes_freed,
            total_bytes,
            timestamp: utils::get_time(),
        }
    }
}

// deletes all expired pastes from the db
// maybe some other cleanup tasks here?
// invalid utf8s? or extremely large pastes?
pub async fn delete_expired_pastes(pool: &Pool<Postgres>) -> anyhow::Result<(i64, i64, i64)> {
    let current_time = utils::get_time();

    // get statistics first
    // octet_length is for bytes https://www.postgresql.org/docs/12/functions-binarystring.html
    // regular char length https://stackoverflow.com/questions/75909283/how-exactly-is-the-length-of-a-postgres-varchar-determined-utf-8
    let stats_row = sqlx::query(
        "SELECT COUNT(*), COALESCE(SUM(octet_length(content)), 0) FROM paste WHERE expires_at IS NOT NULL AND expires_at <= $1"
    )
    .bind(current_time)
    .fetch_one(pool)
    .await?;

    let total_stats_row = sqlx::query("SELECT COALESCE(SUM(octet_length(content)), 0) FROM paste")
        .bind(current_time)
        .fetch_one(pool)
        .await?;

    let count: i64 = stats_row.get(0);
    let total_bytes_freed: i64 = stats_row.get(1);
    let total_bytes: i64 = total_stats_row.get(0);

    // after that delete
    sqlx::query("DELETE FROM paste WHERE expires_at IS NOT NULL AND expires_at <= $1")
        .bind(current_time)
        .execute(pool)
        .await?;

    Ok((count, total_bytes_freed, total_bytes))
}

// run the cleanup task + return stats
pub async fn run_cleanup(pool: &Pool<Postgres>) -> anyhow::Result<CleanupStats> {
    let (deleted, bytes_freed, total_bytes) = delete_expired_pastes(pool).await?;
    Ok(CleanupStats::new(deleted, bytes_freed, total_bytes))
}
