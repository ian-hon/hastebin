use sqlx::{Pool, Postgres};

pub mod routes;

#[derive(Clone)]
pub struct AppState<C: Clone> {
    // we let the user define any config they want, thus the C generic
    //
    // in a perfect world, we would do the same for the pool,
    // but differing providers have differing apis
    pub db: Pool<Postgres>,
    pub config: C,
    // consider caching pastes and comments
}

impl<C: Clone> AppState<C> {
    pub fn new(db: Pool<Postgres>, config: C) -> Self {
        Self { db, config }
    }
}
