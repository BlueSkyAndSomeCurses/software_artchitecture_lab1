
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
        .entry(payload.user_id)
        .and_modify(|curr_bal| *curr_bal += payload.amount)
        .or_insert(payload.amount);

    return Ok(Json(TransactionResponse {
        balance: *balance_ref,
        transaction_id: payload.transaction_id,
    }));
}
