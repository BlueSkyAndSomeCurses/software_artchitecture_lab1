
use std::collections::HashMap;

use axum::{extract::State, http::StatusCode, Json};

use shared::models::{TransactionCommand, TransactionResponse};

use crate::router::AppState;

pub async fn process_transaction(
    State(app_state): State<AppState>,
    Json(payload): Json<TransactionCommand>,
) -> Result<Json<TransactionResponse>, StatusCode> {
    if payload.user_id.is_empty() || payload.amount.is_nan() || payload.transaction_id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut db = app_state.db.lock().await;
    
    let balance_ref = db
        .entry(payload.user_id.clone())
        .and_modify(|curr_bal| *curr_bal += payload.amount)
        .or_insert(payload.amount);

    return Ok(Json(TransactionResponse {
        balance: *balance_ref,
        user_id: payload.user_id,
    }));
}

pub async fn get_user_balance(
    State(app_state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<f64>, StatusCode> {
    if user_id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut db = app_state.db.lock().await;
    
    return Ok(Json(*db.entry(user_id).or_insert(0.0)));
}


pub async fn get_user_balances(
    State(app_state): State<AppState>,
) -> Json<HashMap<String, f64>> {
    return Json(app_state.db.lock().await.clone());
}
