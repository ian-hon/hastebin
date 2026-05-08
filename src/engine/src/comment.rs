use crate::{models::Comment, utils};

impl Comment {
    fn new(
        paste_id: usize,
        id: usize,
        content: String,
        author: Option<String>,
        created_at: u64,
        from_row: usize,
        from_column: usize,
        to_row: usize,
        to_column: usize,
    ) -> Self {
        Self {
            paste_id,
            id,
            content,
            author,
            created_at,
            from_row,
            from_column,
            to_row,
            to_column,
        }
    }

    // should this be here? or creation inside paste entirely?
    pub async fn create(
        paste_id: usize,
        content: String,
        author: Option<String>,
        from_row: usize,
        from_column: usize,
        to_row: usize,
        to_column: usize,
    ) -> Self {
        Self::new(
            paste_id,
            0,
            content,
            author,
            utils::get_time(),
            from_row,
            from_column,
            to_row,
            to_column,
        )
    }
}
