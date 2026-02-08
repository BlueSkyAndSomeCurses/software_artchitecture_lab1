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
    pub db: Arc<Mutex<HashMap<String, f64>>>,
}

pub fn create_router() -> Router {
    
    let state = AppState {
        client: Arc::new(Client::new()),
        db: Arc::new(Mutex::new(HashMap::new())),
    };

    Router::new()
        .route("/counter", post(handlers::process_transaction)).with_state(state)
}