use crate::handlers;
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use std::env;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub kafka_brokers: String,
    pub kafka_transactions_topic: String,
}

pub async fn create_state() -> AppState {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");
    let kafka_brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "kafka:9092".to_string());
    let kafka_transactions_topic =
        env::var("KAFKA_TRANSACTIONS_TOPIC").unwrap_or_else(|_| "transactions".to_string());

    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    AppState {
        db: pg_pool,
        kafka_brokers,
        kafka_transactions_topic,
    }
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/user/{user_id}", get(handlers::get_user_balance))
        .route("/accounts", get(handlers::get_user_balances))
        .with_state(state)
}
