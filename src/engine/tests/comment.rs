mod common;

use common::{DEFAULT_EXPIRY_DAYS, SOFT_LIMIT, clear_tables, create_test_pool};
use engine::models::{Comment, Paste};

// ── helpers ──────────────────────────────────────────────────────────────────

async fn create_paste_with_comments(pool: &sqlx::PgPool, enabled: bool) -> i64 {
    Paste::create(
        "paste for commenting".to_string(),
        None,
        None,
        None,
        enabled,
        None,
        None,
        pool,
        SOFT_LIMIT,
        DEFAULT_EXPIRY_DAYS,
    )
    .await
    .expect("failed to create paste for comment tests")
}

async fn create_comment(paste_id: i64, pool: &sqlx::PgPool) -> i64 {
    Comment::create(
        paste_id,
        "this is a test comment".to_string(),
        Some("tester".to_string()),
        0,  // page_index
        1,  // from_row
        0,  // from_column
        1,  // to_row
        10, // to_column
        pool,
    )
    .await
    .expect("Comment::create returned None")
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_comment_returns_an_id() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let paste_id = create_paste_with_comments(&pool, true).await;
    let comment_id =
        Comment::create(paste_id, "hello".to_string(), None, 0, 0, 0, 0, 5, &pool).await;

    assert!(comment_id.is_some(), "create should return Some(id)");

    clear_tables(&pool).await;
}

#[tokio::test]
async fn fetch_comments_for_paste_returns_all_comments() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let paste_id = create_paste_with_comments(&pool, true).await;
    create_comment(paste_id, &pool).await;
    create_comment(paste_id, &pool).await;
    create_comment(paste_id, &pool).await;

    let comments = Comment::fetch_from_paste(paste_id, &pool).await;
    assert_eq!(comments.len(), 3);
    assert!(comments.iter().all(|c| c.paste_id == paste_id));

    clear_tables(&pool).await;
}

#[tokio::test]
async fn fetch_comments_for_paste_with_no_comments_returns_empty_vec() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let paste_id = create_paste_with_comments(&pool, true).await;
    let comments = Comment::fetch_from_paste(paste_id, &pool).await;
    assert!(comments.is_empty());

    clear_tables(&pool).await;
}

#[tokio::test]
async fn create_comment_on_paste_with_comments_disabled_returns_none() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let paste_id = create_paste_with_comments(&pool, false).await;
    let result = Comment::create(
        paste_id,
        "should be rejected".to_string(),
        None,
        0,
        0,
        0,
        0,
        0,
        &pool,
    )
    .await;

    assert!(
        result.is_none(),
        "commenting on a paste with comments_enabled=false should return None"
    );

    clear_tables(&pool).await;
}

#[tokio::test]
async fn create_comment_on_nonexistent_paste_returns_none() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let result = Comment::create(
        i64::MAX, // no such paste
        "orphan comment".to_string(),
        None,
        0,
        0,
        0,
        0,
        0,
        &pool,
    )
    .await;

    assert!(result.is_none());

    clear_tables(&pool).await;
}

#[tokio::test]
async fn comment_stores_author_and_position_correctly() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let paste_id = create_paste_with_comments(&pool, true).await;
    let comment_id = Comment::create(
        paste_id,
        "annotated text".to_string(),
        Some("bob".to_string()),
        2,  // page_index
        5,  // from_row
        3,  // from_column
        5,  // to_row
        15, // to_column
        &pool,
    )
    .await
    .expect("create returned None");

    let comments = Comment::fetch_from_paste(paste_id, &pool).await;
    let comment = comments
        .iter()
        .find(|c| c.id == comment_id)
        .expect("comment not found after creation");

    assert_eq!(comment.author.as_deref(), Some("bob"));
    assert_eq!(comment.page_index, 2);
    assert_eq!(comment.from_row, 5);
    assert_eq!(comment.from_column, 3);
    assert_eq!(comment.to_row, 5);
    assert_eq!(comment.to_column, 15);
    assert_eq!(comment.content, "annotated text");

    clear_tables(&pool).await;
}

#[tokio::test]
async fn deleting_paste_cascades_to_comments() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let paste_id = create_paste_with_comments(&pool, true).await;
    create_comment(paste_id, &pool).await;
    create_comment(paste_id, &pool).await;

    Paste::delete(paste_id, &pool).await;

    // comments should have been deleted by ON DELETE CASCADE
    let comments = Comment::fetch_from_paste(paste_id, &pool).await;
    assert!(
        comments.is_empty(),
        "comments should cascade-delete when the parent paste is deleted"
    );

    clear_tables(&pool).await;
}

#[tokio::test]
async fn comments_from_different_pastes_are_isolated() {
    let pool = create_test_pool().await;
    clear_tables(&pool).await;

    let paste_a = create_paste_with_comments(&pool, true).await;
    let paste_b = create_paste_with_comments(&pool, true).await;

    create_comment(paste_a, &pool).await;
    create_comment(paste_b, &pool).await;
    create_comment(paste_b, &pool).await;

    let comments_a = Comment::fetch_from_paste(paste_a, &pool).await;
    let comments_b = Comment::fetch_from_paste(paste_b, &pool).await;

    assert_eq!(comments_a.len(), 1);
    assert_eq!(comments_b.len(), 2);

    clear_tables(&pool).await;
}
