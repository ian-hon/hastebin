mod common;

use common::{DEFAULT_EXPIRY_DAYS, SOFT_LIMIT, clear_tables, create_test_pool};
use engine::{cleanup, models::Paste, utils};

// ── helpers ──────────────────────────────────────────────────────────────────

async fn create_paste_expiring_at(expires_at: Option<i64>, pool: &sqlx::PgPool) -> i64 {
    Paste::create(
        "cleanup test paste".to_string(),
        None,
        None,
        None,
        true,
        expires_at,
        None,
        pool,
        SOFT_LIMIT,
        DEFAULT_EXPIRY_DAYS,
    )
    .await
    .expect("failed to create paste for cleanup tests")
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn expired_pastes_are_deleted() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let past = utils::get_time() - 3600; // 1 hour ago
    let id = create_paste_expiring_at(Some(past), &pool).await;

    cleanup::run_cleanup(&pool).await.expect("cleanup failed");

    assert!(
        Paste::fetch_internal(id, &pool).await.is_none(),
        "paste with a past expires_at should have been deleted"
    );

    clear_tables(&pool).await;
}

#[tokio::test]
async fn unexpired_pastes_are_kept() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let future = utils::get_time() + 3600; // 1 hour from now
    let id = create_paste_expiring_at(Some(future), &pool).await;

    cleanup::run_cleanup(&pool).await.expect("cleanup failed");

    assert!(
        Paste::fetch_internal(id, &pool).await.is_some(),
        "paste with a future expires_at should NOT have been deleted"
    );

    clear_tables(&pool).await;
}

#[tokio::test]
async fn pastes_with_no_expiry_are_kept() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let id = create_paste_expiring_at(None, &pool).await;

    cleanup::run_cleanup(&pool).await.expect("cleanup failed");

    assert!(
        Paste::fetch_internal(id, &pool).await.is_some(),
        "paste with no expiry should never be deleted by cleanup"
    );

    clear_tables(&pool).await;
}

#[tokio::test]
async fn cleanup_stats_count_only_expired_pastes() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let past = utils::get_time() - 1;
    let future = utils::get_time() + 86400;

    create_paste_expiring_at(Some(past), &pool).await; // should be deleted
    create_paste_expiring_at(Some(past), &pool).await; // should be deleted
    create_paste_expiring_at(Some(future), &pool).await; // should survive
    create_paste_expiring_at(None, &pool).await; // should survive

    let stats = cleanup::run_cleanup(&pool).await.expect("cleanup failed");
    assert_eq!(
        stats.pastes_deleted, 2,
        "only the 2 expired pastes should be counted as deleted"
    );

    clear_tables(&pool).await;
}

#[tokio::test]
async fn cleanup_stats_bytes_freed_is_positive_when_pastes_deleted() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let past = utils::get_time() - 1;
    create_paste_expiring_at(Some(past), &pool).await;

    let stats = cleanup::run_cleanup(&pool).await.expect("cleanup failed");
    assert!(
        stats.total_bytes_freed > 0,
        "bytes_freed should be > 0 when at least one paste is deleted"
    );

    clear_tables(&pool).await;
}

#[tokio::test]
async fn cleanup_with_no_expired_pastes_deletes_nothing() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let future = utils::get_time() + 86400;
    create_paste_expiring_at(Some(future), &pool).await;
    create_paste_expiring_at(None, &pool).await;

    let stats = cleanup::run_cleanup(&pool).await.expect("cleanup failed");
    assert_eq!(
        stats.pastes_deleted, 0,
        "no pastes should be deleted when none are expired"
    );
    assert_eq!(stats.total_bytes_freed, 0);

    clear_tables(&pool).await;
}

#[tokio::test]
async fn cleanup_stats_timestamp_is_recent() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let before = utils::get_time();
    let stats = cleanup::run_cleanup(&pool).await.expect("cleanup failed");
    let after = utils::get_time();

    assert!(
        stats.timestamp >= before && stats.timestamp <= after,
        "CleanupStats::timestamp should be set at cleanup time"
    );

    clear_tables(&pool).await;
}
