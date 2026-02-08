use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use reqwest::Client;
use tokio::sync::Mutex;
use crate::handlers;
use shared::models::{Metrics};

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<Client>,
    pub metrics: Arc<Mutex<Metrics>>,
}

pub fn create_router() -> Router {
    let state = AppState {
        client: Arc::new(Client::new()),
        metrics: Arc::new(Mutex::new(Metrics::default())),
    };

    Router::new()
        .route("/transaction", post(handlers::process_transaction))
        .route("/user/{user_id}", get(handlers::get_user_info))
        .route("/accounts", get(handlers::get_accounts_balances))
        .route("/metrics", get(handlers::get_timings))
        .with_state(state)
}