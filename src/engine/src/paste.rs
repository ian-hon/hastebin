// pub async fn fetch_paste() {}

// pub async fn create_paste() {}

use crate::{models::Paste, utils};

impl Paste {
    fn new(
        id: usize,
        content: String,
        title: Option<String>,
        author: Option<String>,
        created_at: u64,
        expires_at: Option<u64>,
        forked_from: Option<usize>,
    ) -> Self {
        Self {
            id,
            content,
            title,
            author,
            created_at,
            expires_at,
            forked_from,
        }
    }

    pub async fn fetch(id: usize) {} // temp

    pub async fn create(
        content: String,
        title: Option<String>,
        author: Option<String>,
        expires_at: Option<u64>,
        forked_from: Option<usize>,
    ) -> Self {
        // temp
        Self::new(
            0,
            content,
            title,
            author,
            utils::get_time(),
            expires_at,
            forked_from,
        )
    }
}
