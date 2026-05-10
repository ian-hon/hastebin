use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Paste {
    pub id: i64,
    pub content: String,

    pub title: Option<String>,
    pub author: Option<String>,
    pub checksum_passphrase: Option<String>,

    pub views: i64,
    pub comments_enabled: bool,

    // in epoch unix (ms)
    pub created_at: i64,
    pub expires_at: Option<i64>,

    // used for diffing and forking (duh)
    pub forked_from: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Comment {
    pub paste_id: i64,

    pub id: i64,
    pub content: String,

    // should we put email here too?
    pub author: Option<String>,

    pub created_at: i64,

    // row, column (y, x)
    // treat as unsigned ints
    pub from_row: i64,
    pub from_column: i64,

    pub to_row: i64,
    pub to_column: i64,
}

// similar to go's enum id system
// https://go.dev/wiki/Iota
pub trait Iota<I: Into<i64> + Copy> {
    // https://blog.rust-lang.org/2023/12/28/Rust-1.75.0/#async-fn-and-return-position-impl-trait-in-traits
    // https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/#where-the-gaps-lie
    fn fetch_all_ids(pool: &Pool<Postgres>) -> impl Future<Output = Vec<I>>;
    fn generate_id(pool: &Pool<Postgres>) -> impl Future<Output = I>;
}
