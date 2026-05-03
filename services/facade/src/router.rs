use crate::handlers;
use axum::{
    routing::{get, post},
    Router,
};
use k8s_openapi::api::core::v1::Endpoints;
use kube::Client as KubeClient;
use rand::seq::SliceRandom;
use rdkafka::{config::ClientConfig, producer::FutureProducer};
use reqwest::Client;
use shared::models::Metrics;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<Client>,
    pub kafka_producer: FutureProducer,
    pub kafka_transactions_topic: String,
    pub metrics: Arc<Mutex<Metrics>>,
    pub logging_base_url: String,
    pub counter_base_url: String,
}

async fn resolve_service_endpoint(
    kube_client: &KubeClient,
    kube_namespace: &str,
    service_name: &str,
) -> Result<(String, u16), String> {
    let api: kube::api::Api<Endpoints> =
        kube::api::Api::namespaced(kube_client.clone(), kube_namespace);
    let endpoints = api
        .get(service_name)
        .await
        .map_err(|error| format!("failed to read endpoints for {service_name}: {error}"))?;

    let mut candidates: Vec<(String, u16)> = Vec::new();

    if let Some(subsets) = endpoints.subsets {
        for subset in subsets {
            let ports = subset.ports.unwrap_or_default();
            let addresses = subset.addresses.unwrap_or_default();

            for address in addresses {
                if ports.is_empty() {
                    continue;
                }

                for port in &ports {
                    if port.name.as_deref() == Some("http") || ports.len() == 1 {
                        candidates.push((address.ip.clone(), port.port as u16));
                    }
                }
            }
        }
    }

    let chosen = candidates
        .choose(&mut rand::thread_rng())
        .ok_or_else(|| format!("no endpoints available for {service_name}"))?
        .clone();

    Ok(chosen)
}

pub async fn create_router() -> Router {
    let kafka_brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "kafka:9092".to_string());
    let kafka_transactions_topic =
        env::var("KAFKA_TRANSACTIONS_TOPIC").unwrap_or_else(|_| "transactions".to_string());
    let logging_service_name =
        env::var("LOGGING_SERVICE_NAME").unwrap_or_else(|_| "logging-service".to_string());
    let counter_service_name =
        env::var("COUNTER_SERVICE_NAME").unwrap_or_else(|_| "counter-service".to_string());
    let kube_namespace = env::var("KUBE_NAMESPACE").unwrap_or_else(|_| "default".to_string());

    let http_client = Client::new();
    let kube_client = KubeClient::try_default()
        .await
        .expect("failed to create kube client");

    let (logging_ip, logging_port) = resolve_service_endpoint(
        &kube_client,
        &kube_namespace,
        &logging_service_name,
    )
    .await
    .expect("failed to resolve logging service endpoint");
    let (counter_ip, counter_port) = resolve_service_endpoint(
        &kube_client,
        &kube_namespace,
        &counter_service_name,
    )
    .await
    .expect("failed to resolve counter service endpoint");

    let logging_base_url = format!("http://{}:{}", logging_ip, logging_port);
    let counter_base_url = format!("http://{}:{}", counter_ip, counter_port);

    let state = AppState {
        client: Arc::new(http_client),
        kafka_producer: ClientConfig::new()
            .set("bootstrap.servers", &kafka_brokers)
            .create()
            .expect("failed to create kafka producer"),
        kafka_transactions_topic,
        metrics: Arc::new(Mutex::new(Metrics::default())),
        logging_base_url,
        counter_base_url,
    };

    Router::new()
        .route("/transaction", post(handlers::process_transaction))
        .route("/user/{user_id}", get(handlers::get_user_info))
        .route("/accounts", get(handlers::get_accounts_balances))
        .route("/metrics", get(handlers::get_timings))
        .with_state(state)
}
