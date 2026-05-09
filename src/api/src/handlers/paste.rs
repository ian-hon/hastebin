use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use engine::models::Paste;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreatePasteRequest {
    pub content: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub checksum_passphrase: Option<String>,
    pub expires_at: Option<i64>,
    pub forked_from: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CreatePasteResponse {
    pub id: i64,
}

pub async fn fetch_paste(
    State(state): State<crate::AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Paste>, StatusCode> {
    match Paste::fetch(id, &state.db).await {
        Some(paste) => Ok(Json(paste)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_paste(
    State(state): State<crate::AppState>,
    Json(payload): Json<CreatePasteRequest>,
) -> Result<Json<CreatePasteResponse>, StatusCode> {
    // we check if the parent exists first
    if let Some(p) = payload.forked_from
        && let None = Paste::fetch(p, &state.db).await
    {
        return Err(StatusCode::NOT_FOUND);
    }

    let id = Paste::create(
        payload.content,
        payload.title,
        payload.author,
        payload.checksum_passphrase,
        payload.expires_at,
        payload.forked_from,
        &state.db,
        state.config.size_soft_limit,
        state.config.default_expiry_days,
    )
    .await;

    Ok(Json(CreatePasteResponse { id }))
}
