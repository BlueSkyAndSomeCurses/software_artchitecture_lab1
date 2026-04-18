use crate::handlers;
use axum::{
    routing::{get, post},
    Router,
};
use reqwest::Client;
use shared::models::KafkaConfigResponse;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use std::env;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub kafka_brokers: String,
    pub kafka_transactions_topic: String,
}

async fn fetch_kafka_config(
    client: &Client,
    config_server_base_url: &str,
) -> Option<KafkaConfigResponse> {
    let response = client
        .get(format!("{}/kafka-config", config_server_base_url))
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    response.json::<KafkaConfigResponse>().await.ok()
}

pub async fn create_state() -> AppState {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");
    let config_server_base_url =
        env::var("CONFIG_SERVER_URL").unwrap_or_else(|_| "http://config-server:8083".to_string());
    let fallback_kafka_brokers =
        env::var("KAFKA_BROKERS").unwrap_or_else(|_| "kafka:9092".to_string());
    let fallback_kafka_transactions_topic =
        env::var("KAFKA_TRANSACTIONS_TOPIC").unwrap_or_else(|_| "transactions".to_string());

    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    let http_client = Client::new();
    let resolved_kafka_config = fetch_kafka_config(&http_client, &config_server_base_url).await;
    let kafka_brokers = resolved_kafka_config
        .as_ref()
        .map(|config| config.brokers.clone())
        .unwrap_or(fallback_kafka_brokers);
    let kafka_transactions_topic = resolved_kafka_config
        .map(|config| config.transactions_topic)
        .unwrap_or(fallback_kafka_transactions_topic);

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
