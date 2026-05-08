use axum::{extract::{Path, State}, response::IntoResponse, Json};
use axum_extra::extract::WithRejection;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, Pool, Postgres};

use crate::{extractor_error::ExtractorError, utils::{self, async_rng_range_int_big, ValueInt}, AppState};

const KEY_LENGTH: u32 = 4;

#[derive(Serialize, Deserialize)]
pub struct Paste {
    pub id: i64,
    pub content: Vec<(String, String)>,
    pub signature: String, 
    pub views: i64,
    pub timestamp: i64
}
impl Paste {
    pub async fn create(db: &Pool<Postgres>, content: Vec<(String, String)>, signature: String) -> i64 {
        // returns id on creation
        let id = Self::generate_id(db).await;

        sqlx::query("insert into hastebin.paste(id, content, signature, views, timestamp) values($1, $2, $3, $4, $5);")
            .bind(id)
            .bind(serde_json::to_string(&content).unwrap())
            .bind(signature)
            .bind(0)
            .bind(utils::get_time())
            .execute(db)
            .await.unwrap();

        id
    }

    pub async fn generate_id(db: &Pool<Postgres>) -> i64 {
        let ids = sqlx::query_as::<_, ValueInt>("select id from hastebin.paste;")
            .fetch_all(db)
            .await
            .unwrap()
            .iter()
            .map(|x| x.0.clone())
            .collect::<Vec<i64>>();

        for _ in 0..(16i64.pow(KEY_LENGTH)) {
            let candidate = async_rng_range_int_big(0, 16i64.pow(KEY_LENGTH));
            if ids.contains(&candidate) {
                continue;
            }
    
            return candidate;
        }

        loop {
            let candidate = async_rng_range_int_big(0, 16i64.pow(KEY_LENGTH * 2));
            if ids.contains(&candidate) {
                continue;
            }

            return candidate;
        }
    }

    pub async fn fetch(db: &Pool<Postgres>, id: i64) -> Option<Paste> {
        // sqlx::query_as::<_, RawPaste>("select * from hastebin.paste where id = $1;")
        //     .bind(id)
        //     .fetch_optional(db)
        //     .await.unwrap().map_or(None, |p| Some(p.into()))

        match sqlx::query_as::<_, RawPaste>("select * from hastebin.paste where id = $1;")
            .bind(id)
            .fetch_optional(db)
            .await.unwrap() {
            Some(p) => {
                sqlx::query("update hastebin.paste set views = views + 1 where id = $1")
                    .bind(id)
                    .execute(db)
                    .await.unwrap();

                let mut p = p;
                p.views += 1;

                Some(p.into())
            },
            None => None
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PayloadPaste { // used by json
    pub content: Vec<(String, String)>,
    pub signature: String
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct RawPaste { // used by db
    pub id: i64,
    pub content: String,
    pub signature: String,
    pub views: i64,
    pub timestamp: i64
}
impl Into<Paste> for RawPaste {
    fn into(self) -> Paste {
        Paste {
            id: self.id,
            content: serde_json::from_str(self.content.as_str()).unwrap(),
            signature: self.signature,
            views: self.views,
            timestamp: self.timestamp
        }
    }
}

#[axum::debug_handler]
pub async fn create(
    State(app_state): State<AppState>,
    WithRejection(Json(payload), _): WithRejection<Json<PayloadPaste>, ExtractorError>
) -> impl IntoResponse {
    Paste::create(&app_state.db, payload.content, payload.signature).await.to_string().into_response()
}

#[axum::debug_handler]
pub async fn fetch(
    State(app_state): State<AppState>,
    Path(id): Path<ValueInt>
) -> impl IntoResponse {
    serde_json::to_string(&Paste::fetch(&app_state.db, id.0).await).unwrap().into_response()
}
