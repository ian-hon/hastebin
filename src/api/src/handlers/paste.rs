use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use engine::models::Paste;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct FetchPasteResponse {
    pub paste: Paste,
    pub checksum_pair: Option<(String, String)>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePasteRequest {
    pub content: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub comments_enabled: bool,
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
) -> Result<Json<FetchPasteResponse>, StatusCode> {
    // check cache
    if let Some(mut paste) = state.cache.get(id) {
        if let Some(expires_at) = paste.expires_at
            && engine::utils::get_time() > expires_at
        {
            // remove, then proceed with regular fetching
            println!("cached paste expired");
            state.cache.remove(id);
        } else {
            println!("returning from cache");
            return Ok(Json(FetchPasteResponse {
                checksum_pair: paste.construct_checksum_pair(),
                paste: {
                    paste.checksum_passphrase = None;
                    paste
                },
            }));
        }
    }

    // cache miss
    // this section runs if the cached paste is expired too
    println!("cache miss. fetching from db");
    match Paste::fetch(id, &state.db).await {
        Some(mut paste) => {
            if let Some(expires_at) = paste.expires_at {
                let current_time = engine::utils::get_time();
                // println!("{current_time}, {expires_at}");
                if current_time > expires_at {
                    // delete it
                    Paste::delete(id, &state.db).await;
                    return Err(StatusCode::NOT_FOUND);
                }
            }

            state.cache.insert(id, paste.clone());
            Ok(Json(FetchPasteResponse {
                checksum_pair: paste.construct_checksum_pair(),
                paste: {
                    paste.checksum_passphrase = None;
                    paste
                },
            }))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_paste(
    State(state): State<crate::AppState>,
    Json(payload): Json<CreatePasteRequest>,
) -> Result<Json<CreatePasteResponse>, StatusCode> {
    // we check if the parent exists first (use fetch_internal to avoid increasing views)
    if let Some(p) = payload.forked_from
        && let None = Paste::fetch_internal(p, &state.db).await
    {
        return Err(StatusCode::NOT_FOUND);
    }

    if let Some(id) = Paste::create(
        payload.content,
        payload.title,
        payload.author,
        payload.checksum_passphrase,
        payload.comments_enabled,
        payload.expires_at,
        payload.forked_from,
        &state.db,
        state.config.size_soft_limit,
        state.config.default_expiry_days,
    )
    .await
    {
        return Ok(Json(CreatePasteResponse { id }));
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
