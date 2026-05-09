use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use engine::models::Comment;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub paste_id: i64,
    pub content: String,
    pub author: Option<String>,
    pub from_row: i64,
    pub from_column: i64,
    pub to_row: i64,
    pub to_column: i64,
}

#[derive(Debug, Serialize)]
pub struct CreateCommentResponse {
    pub id: i64,
}

pub async fn fetch_comment(
    State(state): State<crate::AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Comment>, StatusCode> {
    match Comment::fetch(id, &state.db).await {
        Some(comment) => Ok(Json(comment)), // uses serde_json
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn fetch_comments_by_paste(
    State(state): State<crate::AppState>,
    Path(paste_id): Path<i64>,
) -> Json<Vec<Comment>> {
    let comments = Comment::fetch_from_paste(paste_id, &state.db).await;
    Json(comments)
}

pub async fn create_comment(
    State(state): State<crate::AppState>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<Json<CreateCommentResponse>, StatusCode> {
    let id = Comment::create(
        payload.paste_id,
        payload.content,
        payload.author,
        payload.from_row,
        payload.from_column,
        payload.to_row,
        payload.to_column,
        &state.db,
    )
    .await;

    Ok(Json(CreateCommentResponse { id }))
}
