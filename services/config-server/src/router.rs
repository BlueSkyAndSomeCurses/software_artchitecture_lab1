use crate::handlers;
use axum::{routing::get, Router};
use std::env;

#[derive(Clone)]
pub struct AppState {
    pub logging_services: Vec<String>,
    pub messages_service: String,
    pub kafka_brokers: String,
    pub kafka_transactions_topic: String,
}

pub fn create_router() -> Router {
    let logging_services = env::var("LOGGING_SERVICES")
        .unwrap_or_else(|_| "logging-1:8082,logging-2:8082,logging-3:8082".to_string())
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    let messages_service = env::var("MESSAGES_SERVICE")
        .unwrap_or_else(|_| "messages:8081".to_string());

    let kafka_brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "kafka:9092".to_string());
    let kafka_transactions_topic =
        env::var("KAFKA_TRANSACTIONS_TOPIC").unwrap_or_else(|_| "transactions".to_string());

    let state = AppState {
        logging_services,
        messages_service,
        kafka_brokers,
        kafka_transactions_topic,
    };

    Router::new()
        .route("/logging-ip", get(handlers::get_logging_ip))
        .route("/messages-ip", get(handlers::get_messages_ip))
        .route("/kafka-config", get(handlers::get_kafka_config))
        .with_state(state)
}
