use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router<crate::AppState> {
    Router::new()
        .route("/fetch/{id}", get(crate::handlers::paste::fetch_paste))
        .route("/create", post(crate::handlers::paste::create_paste))
}
