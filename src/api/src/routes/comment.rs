use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router<crate::AppState> {
    Router::new()
        .route("/fetch/{id}", get(crate::handlers::comment::fetch_comment))
        .route(
            "/paste/{paste_id}",
            get(crate::handlers::comment::fetch_comments_by_paste),
        )
        .route("/create", post(crate::handlers::comment::create_comment))
}
