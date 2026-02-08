use std::{collections::HashMap};
use axum::{
    routing::{get, post},
    Router,
};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::handlers;

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<Client>,
    pub log_db: Arc<Mutex<HashMap<String, (String, f64)>>>,
}

pub fn create_router() -> Router {
    
    let state = AppState {
        client: Arc::new(Client::new()),
        log_db: Arc::new(Mutex::new(HashMap::new())),
    };

    Router::new()
        .route("/log", post(handlers::process_transaction))
        .route("/transactions/{user_id}", get(handlers::get_user_transactions))
        .with_state(state)
}