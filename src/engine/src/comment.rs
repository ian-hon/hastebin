use sqlx::{Pool, Postgres};

use crate::{
    models::{Comment, Iota},
    utils,
};

impl Comment {
    #[allow(unused)]
    fn new(
        paste_id: i64,
        id: i64,
        content: String,
        author: Option<String>,
        created_at: i64,
        page_index: i64,
        from_row: i64,
        from_column: i64,
        to_row: i64,
        to_column: i64,
    ) -> Self {
        Self {
            paste_id,
            id,
            content,
            author,
            created_at,
            page_index,
            from_row,
            from_column,
            to_row,
            to_column,
        }
    }

    // TODO: encrypt this as well
    pub async fn fetch(id: i64, pool: &Pool<Postgres>) -> Option<Comment> {
        let r = sqlx::query_as::<_, Comment>("SELECT * FROM comment WHERE id = $2")
            .bind(id)
            .fetch_optional(pool)
            .await;

        r.ok().flatten()
    }

    pub async fn fetch_from_paste(paste_id: i64, pool: &Pool<Postgres>) -> Vec<Comment> {
        let r = sqlx::query_as::<_, Comment>("SELECT * FROM comment WHERE paste_id = $1")
            .bind(paste_id)
            .fetch_all(pool)
            .await;

        r.unwrap_or_default()
    }

    pub async fn create(
        paste_id: i64,
        content: String,
        author: Option<String>,
        page_index: i64,
        from_row: i64,
        from_column: i64,
        to_row: i64,
        to_column: i64,
        pool: &Pool<Postgres>,
    ) -> Option<i64> {
        if !crate::models::Paste::fetch(paste_id, pool)
            .await?
            .comments_enabled
        {
            return None;
        }

        let id = Self::generate_id(pool).await;

        let _ = sqlx::query("INSERT INTO comment(paste_id, id, content, author, created_at, page_index, from_row, from_column, to_row, to_column) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
            .bind(paste_id)
            .bind(id)
            .bind(content)
            .bind(author)
            .bind(utils::get_time()) // see paste.rs on why not postgres' now()
            .bind(page_index)
            .bind(from_row)
            .bind(from_column)
            .bind(to_row)
            .bind(to_column)
            .execute(pool).await;

        Some(id)
    }
}

impl Iota<i64> for Comment {
    async fn fetch_all_ids(pool: &Pool<Postgres>) -> Vec<i64> {
        let r = sqlx::query_scalar::<_, i64>("SELECT id FROM comment")
            .fetch_all(pool)
            .await;

        r.unwrap_or_default()
    }

    async fn generate_id(pool: &Pool<Postgres>) -> i64 {
        // let ids = Self::fetch_all_ids(&pool).await;
        // // O(n), expensive?
        // *ids.iter().max().unwrap_or(&0)
        let candidate = sqlx::query_scalar::<_, i64>("SELECT MAX(id) FROM comment")
            .fetch_optional(pool)
            .await;

        candidate.ok().flatten().unwrap_or(0) + 1
    }
}
