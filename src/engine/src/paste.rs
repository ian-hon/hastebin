use sqlx::{Pool, Postgres};

use crate::{
    models::{Iota, Paste},
    utils,
};

impl Paste {
    fn new(
        id: i64,
        content: String,
        title: Option<String>,
        author: Option<String>,
        views: i64,
        created_at: i64,
        expires_at: Option<i64>,
        forked_from: Option<i64>,
    ) -> Self {
        Self {
            id,
            content,
            title,
            author,
            views,
            created_at,
            expires_at,
            forked_from,
        }
    }

    pub async fn fetch(id: i64, pool: &Pool<Postgres>) -> Option<Paste> {
        // https://docs.rs/sqlx/latest/sqlx/fn.query_as.html#example-map-rows-using-tuples
        let r = sqlx::query_as::<_, Paste>("SELECT * FROM hastebin.paste WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await;

        let _ = sqlx::query("UPDATE hastebin.paste SET view = view + 1 WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await;

        r.ok().flatten()
    }

    pub async fn create(
        content: String,
        title: Option<String>,
        author: Option<String>,
        expires_at: Option<i64>,
        forked_from: Option<i64>,
        pool: &Pool<Postgres>,
    ) -> i64 {
        let id = Self::generate_id(pool).await;

        let _ = sqlx::query("INSERT INTO hastebin.paste(id, content, title, author, views, created_at, expires_at, forked_from) VALUES($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind(id)
            .bind(content)
            .bind(title)
            .bind(author)
            .bind(0)
            .bind(utils::get_time())
            .bind(expires_at)
            .bind(forked_from)
            .execute(pool);

        id
    }
}
impl Iota<i64> for Paste {
    async fn fetch_all_ids(pool: &Pool<Postgres>) -> Vec<i64> {
        // https://docs.rs/sqlx/latest/sqlx/fn.query_scalar.html
        // https://docs.rs/sqlx/latest/sqlx/query/struct.QueryAs.html#method.fetch_all
        let r = sqlx::query_scalar::<_, i64>("SELECT id FROM hastebin.paste")
            .fetch_all(pool)
            .await;

        r.unwrap_or_default()
    }
}
