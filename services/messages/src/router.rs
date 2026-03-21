use axum::{
    routing::{get, post},
    Router,
};
use reqwest::Client;
use std::sync::Arc;
use crate::handlers;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use std::env;

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<Client>,
    pub db: PgPool
}

pub async fn create_router() -> Router {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");
    
    let pg_pool = PgPoolOptions::new().max_connections(5).connect(&db_url).await.unwrap();
    
    let state = AppState {
        client: Arc::new(Client::new()),
        db: pg_pool
    };

    Router::new()
        .route("/counter", post(handlers::process_transaction))
        .route("/user/{user_id}", get(handlers::get_user_balance))
        .route("/accounts", get(handlers::get_user_balances))
        .with_state(state)
}