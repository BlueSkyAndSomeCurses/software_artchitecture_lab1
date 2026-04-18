use crate::handlers;
use axum::{
    routing::{get, post},
    Router,
};
use rdkafka::{config::ClientConfig, producer::FutureProducer};
use reqwest::Client;
use shared::models::{KafkaConfigResponse, Metrics};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<Client>,
    pub kafka_producer: FutureProducer,
    pub kafka_transactions_topic: String,
    pub metrics: Arc<Mutex<Metrics>>,
    pub config_server_base_url: String,
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

pub async fn create_router() -> Router {
    let config_server_base_url =
        env::var("CONFIG_SERVER_URL").unwrap_or_else(|_| "http://config-server:8083".to_string());
    let fallback_kafka_brokers =
        env::var("KAFKA_BROKERS").unwrap_or_else(|_| "kafka:9092".to_string());
    let fallback_kafka_transactions_topic =
        env::var("KAFKA_TRANSACTIONS_TOPIC").unwrap_or_else(|_| "transactions".to_string());

    let http_client = Client::new();
    let resolved_kafka_config = fetch_kafka_config(&http_client, &config_server_base_url).await;
    let kafka_brokers = resolved_kafka_config
        .as_ref()
        .map(|config| config.brokers.clone())
        .unwrap_or(fallback_kafka_brokers);
    let kafka_transactions_topic = resolved_kafka_config
        .map(|config| config.transactions_topic)
        .unwrap_or(fallback_kafka_transactions_topic);

    let state = AppState {
        client: Arc::new(http_client),
        kafka_producer: ClientConfig::new()
            .set("bootstrap.servers", &kafka_brokers)
            .create()
            .expect("failed to create kafka producer"),
        kafka_transactions_topic,
        metrics: Arc::new(Mutex::new(Metrics::default())),
        config_server_base_url,
    };

    Router::new()
        .route("/transaction", post(handlers::process_transaction))
        .route("/user/{user_id}", get(handlers::get_user_info))
        .route("/accounts", get(handlers::get_accounts_balances))
        .route("/metrics", get(handlers::get_timings))
        .with_state(state)
}
