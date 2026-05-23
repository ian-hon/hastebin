mod common;

use common::{DEFAULT_EXPIRY_DAYS, SOFT_LIMIT, clear_tables, create_test_pool};
use engine::{models::Paste, utils};

// ── helpers ──────────────────────────────────────────────────────────────────

async fn create_simple_paste(pool: &sqlx::PgPool) -> i64 {
    Paste::create(
        "hello, world".to_string(),
        None,
        None,
        None,
        true,
        None,
        None,
        pool,
        SOFT_LIMIT,
        DEFAULT_EXPIRY_DAYS,
    )
    .await
    .expect("create_simple_paste: Paste::create returned None")
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_and_fetch_returns_correct_content() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let id = create_simple_paste(&pool).await;
    let paste = Paste::fetch_internal(id, &pool)
        .await
        .expect("paste should exist after creation");

    assert_eq!(paste.id, id);
    assert_eq!(paste.content, "hello, world");
    assert_eq!(paste.views, 0);
    assert!(paste.comments_enabled);

    clear_tables(&pool).await;
}

#[tokio::test]
async fn fetch_increments_view_counter() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let id = create_simple_paste(&pool).await;

    Paste::fetch(id, &pool).await;
    Paste::fetch(id, &pool).await;

    let paste = Paste::fetch_internal(id, &pool)
        .await
        .expect("paste should exist");
    assert_eq!(paste.views, 2);

    clear_tables(&pool).await;
}

#[tokio::test]
async fn fetch_internal_does_not_increment_views() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let id = create_simple_paste(&pool).await;

    Paste::fetch_internal(id, &pool).await;
    Paste::fetch_internal(id, &pool).await;

    let paste = Paste::fetch_internal(id, &pool)
        .await
        .expect("paste should exist");
    assert_eq!(paste.views, 0);

    clear_tables(&pool).await;
}

#[tokio::test]
async fn fetch_nonexistent_paste_returns_none() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let result = Paste::fetch_internal(i64::MAX, &pool).await;
    assert!(result.is_none());

    clear_tables(&pool).await;
}

#[tokio::test]
async fn delete_paste_removes_it_from_db() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let id = create_simple_paste(&pool).await;
    assert!(Paste::fetch_internal(id, &pool).await.is_some());

    let deleted = Paste::delete(id, &pool).await;
    assert!(deleted, "delete should return true");
    assert!(Paste::fetch_internal(id, &pool).await.is_none());

    clear_tables(&pool).await;
}

#[tokio::test]
async fn delete_nonexistent_paste_returns_true_gracefully() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    // DELETE with no matching rows still succeeds at the SQL level
    let result = Paste::delete(i64::MAX, &pool).await;
    assert!(result);

    clear_tables(&pool).await;
}

#[tokio::test]
async fn create_with_optional_fields_roundtrips() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let id = Paste::create(
        "content with metadata".to_string(),
        Some("My Title".to_string()),
        Some("alice".to_string()),
        None,
        false,
        None,
        None,
        &pool,
        SOFT_LIMIT,
        DEFAULT_EXPIRY_DAYS,
    )
    .await
    .expect("create failed");

    let paste = Paste::fetch_internal(id, &pool)
        .await
        .expect("paste missing");
    assert_eq!(paste.title.as_deref(), Some("My Title"));
    assert_eq!(paste.author.as_deref(), Some("alice"));
    assert!(!paste.comments_enabled);

    clear_tables(&pool).await;
}

#[tokio::test]
async fn create_with_explicit_expiry_stores_value() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let future_expiry = utils::get_time() + 3600; // 1 hour from now
    let id = Paste::create(
        "temporary paste".to_string(),
        None,
        None,
        None,
        true,
        Some(future_expiry),
        None,
        &pool,
        SOFT_LIMIT,
        DEFAULT_EXPIRY_DAYS,
    )
    .await
    .expect("create failed");

    let paste = Paste::fetch_internal(id, &pool)
        .await
        .expect("paste missing");
    assert_eq!(paste.expires_at, Some(future_expiry));

    clear_tables(&pool).await;
}

#[tokio::test]
async fn oversized_content_receives_enforced_expiry() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    // content that exceeds the soft limit
    let large_content = "x".repeat(SOFT_LIMIT + 1);
    let id = Paste::create(
        large_content,
        None,
        None,
        None,
        true,
        None, // no explicit expiry — should be imposed by the soft limit logic
        None,
        &pool,
        SOFT_LIMIT,
        DEFAULT_EXPIRY_DAYS,
    )
    .await
    .expect("create failed");

    let paste = Paste::fetch_internal(id, &pool)
        .await
        .expect("paste missing");
    assert!(
        paste.expires_at.is_some(),
        "oversized paste should have an enforced expiry"
    );

    clear_tables(&pool).await;
}

#[tokio::test]
async fn checksum_passphrase_is_stored_as_hash_not_plaintext() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let plaintext = "supersecret";
    let id = Paste::create(
        "private paste".to_string(),
        None,
        None,
        Some(plaintext.to_string()),
        true,
        None,
        None,
        &pool,
        SOFT_LIMIT,
        DEFAULT_EXPIRY_DAYS,
    )
    .await
    .expect("create failed");

    let paste = Paste::fetch_internal(id, &pool)
        .await
        .expect("paste missing");
    let stored = paste.checksum_passphrase.as_deref().unwrap_or("");
    assert_ne!(
        stored, plaintext,
        "passphrase must be stored as a hash, not plaintext"
    );
    assert_eq!(stored.len(), 64, "SHA-256 hex digest should be 64 chars");

    clear_tables(&pool).await;
}

#[tokio::test]
async fn forked_paste_references_parent() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let parent_id = create_simple_paste(&pool).await;
    let fork_id = Paste::create(
        "forked content".to_string(),
        None,
        None,
        None,
        true,
        None,
        Some(parent_id),
        &pool,
        SOFT_LIMIT,
        DEFAULT_EXPIRY_DAYS,
    )
    .await
    .expect("fork create failed");

    let fork = Paste::fetch_internal(fork_id, &pool)
        .await
        .expect("fork missing");
    assert_eq!(fork.forked_from, Some(parent_id));

    clear_tables(&pool).await;
}

#[tokio::test]
async fn content_is_compressed_and_decompressed_transparently() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let original = "The quick brown fox jumps over the lazy dog".repeat(200);
    let id = Paste::create(
        original.clone(),
        None,
        None,
        None,
        true,
        None,
        None,
        &pool,
        SOFT_LIMIT,
        DEFAULT_EXPIRY_DAYS,
    )
    .await
    .expect("create failed");

    let paste = Paste::fetch_internal(id, &pool)
        .await
        .expect("paste missing");
    assert_eq!(paste.content, original);

    clear_tables(&pool).await;
}
