use axum::{
    routing::{get, post},
    Router,
};
use reqwest::Client;
use std::sync::Arc;
use crate::handlers;

pub fn create_router() -> Router {
    let client = Arc::new(Client::new());

    Router::new()
        .route("/transaction", post(handlers::process_transaction)).with_state(client)
}