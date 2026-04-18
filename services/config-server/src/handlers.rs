use axum::{extract::State, http::StatusCode, Json};
use rand::seq::SliceRandom;
use shared::models::{KafkaConfigResponse, ServiceIpResponse};
use tokio::net::lookup_host;

use crate::router::AppState;

async fn resolve_service_ip(service: &str) -> Result<String, StatusCode> {
    let mut resolved = lookup_host(service)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    resolved
        .next()
        .map(|addr| addr.ip().to_string())
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)
}

pub async fn get_logging_ip(
    State(state): State<AppState>,
) -> Result<Json<ServiceIpResponse>, StatusCode> {
    let service = state
        .logging_services
        .choose(&mut rand::thread_rng())
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let ip = resolve_service_ip(service).await?;

    Ok(Json(ServiceIpResponse { ip }))
}

pub async fn get_messages_ip(
    State(state): State<AppState>,
) -> Result<Json<ServiceIpResponse>, StatusCode> {
    let ip = resolve_service_ip(&state.messages_service).await?;

    Ok(Json(ServiceIpResponse { ip }))
}

pub async fn get_kafka_config(State(state): State<AppState>) -> Json<KafkaConfigResponse> {
    Json(KafkaConfigResponse {
        brokers: state.kafka_brokers.clone(),
        transactions_topic: state.kafka_transactions_topic.clone(),
    })
}
