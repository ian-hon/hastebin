use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Paste {
    pub id: usize,
    pub content: String,

    pub title: Option<String>,
    pub author: Option<String>,

    // in epoch unix (ms)
    pub created_at: u64,
    pub expires_at: Option<u64>,

    // used for diffing and forking (duh)
    pub forked_from: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Comment {
    pub paste_id: usize,

    pub id: usize,
    pub content: String,

    // should we put email here too?
    pub author: Option<String>,

    pub created_at: u64,

    // row, column (y, x)
    pub from_row: usize,
    pub from_column: usize,

    pub to_row: usize,
    pub to_column: usize,
    // pub from: (usize, usize),
    // pub to: (usize, usize),
    // pub from: (usize, Option<usize>),
    // pub to: (usize, Option<usize>),
}
