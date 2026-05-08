use crate::{models::Comment, utils};

impl Comment {
    fn new(
        paste_id: i64,
        id: i64,
        content: String,
        author: Option<String>,
        created_at: i64,
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
            from_row,
            from_column,
            to_row,
            to_column,
        }
    }

    // should this be here? or creation inside paste entirely?
    pub async fn create(
        paste_id: i64,
        content: String,
        author: Option<String>,
        from_row: i64,
        from_column: i64,
        to_row: i64,
        to_column: i64,
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
