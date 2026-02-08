use std::vec;

use axum::{extract::State, http::StatusCode, Json};

use shared::models::{TransactionCommand};

use crate::router::AppState;

pub async fn process_transaction(
    State(app_state): State<AppState>,
    Json(payload): Json<TransactionCommand>,
) -> StatusCode {
    if payload.user_id.is_empty() || payload.amount.is_nan() || payload.transaction_id.is_empty() {
        return StatusCode::BAD_REQUEST;
    }

    let mut log_db = app_state.log_db.lock().await;

    log_db.insert(payload.transaction_id.clone(), (payload.user_id.clone(), payload.amount));
    println!("Logged transaction: {} for user: {} with amount: {}", payload.transaction_id, payload.user_id, payload.amount);

    return StatusCode::OK;
}

pub async fn get_user_transactions(
    State(app_state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<Vec<f64>>, StatusCode> {
    if user_id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let log_db = app_state.log_db.lock().await;
    
    let user_transactions: Vec<f64> = log_db.iter()
        .filter(|(_, (uid, _))| uid == &user_id)
        .map(|(_, (_, amount))| *amount)
        .collect();

    return Ok(Json(user_transactions));
}
