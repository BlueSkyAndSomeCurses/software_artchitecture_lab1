use axum::{extract::State, http::StatusCode, Json};

use shared::models::{TransactionCommand};
use fred::prelude::{Error as RedisError, HashesInterface};

use crate::router::AppState;

pub async fn process_transaction(
    State(app_state): State<AppState>,
    Json(payload): Json<TransactionCommand>,
) -> StatusCode {
    if payload.user_id.is_empty() || payload.amount.is_nan() || payload.transaction_id.is_empty() {
        return StatusCode::BAD_REQUEST;
    }

    let log_db = &app_state.redis;
    
    let redis_key = format!("user:{}:transactions", payload.user_id);
    
    let result: Result<(), RedisError> = log_db.hset(redis_key, (payload.transaction_id.clone(), payload.amount)).await;
    
    match result {
        Ok(_) => {
            println!("Logged transaction: {} for user: {} with amount: {}", payload.transaction_id, payload.user_id, payload.amount);
            StatusCode::OK
        }
        Err(e) => {
            eprintln!("Failed to write to Redis: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
        
    }
}

pub async fn get_user_transactions(
    State(app_state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<Vec<f64>>, StatusCode> {
    if user_id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let log_db = &app_state.redis;
    let redis_key = format!("user:{}:transactions", user_id);

    let result: Result<Vec<String>, RedisError> = log_db.hvals(&redis_key).await;

    match result {
        Ok(amounts_str) => {
            let user_transactions: Vec<f64> = amounts_str
                .into_iter()
                .filter_map(|s| s.parse::<f64>().ok())
                .collect();

            Ok(Json(user_transactions))
        }
        Err(e) => {
            eprintln!("Failed to read from Redis: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
