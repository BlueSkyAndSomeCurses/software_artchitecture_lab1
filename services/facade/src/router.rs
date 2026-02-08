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
        .route("/transaction", post(handlers::process_transaction))
        .route("/user/{user_id}", get(handlers::get_user_info))
        .route("/accounts", get(handlers::get_accounts_balances))
        .with_state(client)
}