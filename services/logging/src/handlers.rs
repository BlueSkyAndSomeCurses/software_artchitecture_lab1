use axum::{extract::State, http::StatusCode, Json};

use fred::prelude::{Error as RedisError, HashesInterface};
use shared::models::TransactionCommand;

use crate::router::AppState;

pub async fn process_transaction(
    State(app_state): State<AppState>,
    Json(payload): Json<TransactionCommand>,
) -> StatusCode {
    println!(
        "[logging] /log request: user_id={}, transaction_id={}, amount={}",
        payload.user_id, payload.transaction_id, payload.amount
    );

    if payload.user_id.is_empty() || payload.amount.is_nan() || payload.transaction_id.is_empty() {
        eprintln!(
            "[logging] /log rejected: invalid payload (user_id='{}', transaction_id='{}', amount={})",
            payload.user_id, payload.transaction_id, payload.amount
        );
        return StatusCode::BAD_REQUEST;
    }

    let redis_key = format!("user:{}:transactions", payload.user_id);

    if let Some(redis) = &app_state.redis {
        let result: Result<(), RedisError> = redis
            .hset(redis_key, (payload.transaction_id.clone(), payload.amount))
            .await;

        if let Err(e) = result {
            eprintln!("[logging] Redis write failed, skipping: {:?}", e);
        } else {
            println!(
                "[logging] Stored transaction: user_id={}, transaction_id={}",
                payload.user_id, payload.transaction_id
            );
        }
    } else {
        eprintln!("[logging] Redis not available, skipping write");
    }

    println!(
        "[logging] /log response: 200 OK for transaction_id={}",
        payload.transaction_id
    );
    StatusCode::OK
}

pub async fn get_user_transactions(
    State(app_state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<Vec<f64>>, StatusCode> {
    println!("[logging] /transactions request: user_id={}", user_id);

    if user_id.is_empty() {
        eprintln!("[logging] /transactions rejected: empty user_id");
        return Err(StatusCode::BAD_REQUEST);
    }

    let redis = match &app_state.redis {
        Some(r) => r,
        None => {
            eprintln!("[logging] Redis not available");
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };
    let redis_key = format!("user:{}:transactions", user_id);
    let result: Result<Vec<String>, RedisError> = redis.hvals(&redis_key).await;

    match result {
        Ok(amounts_str) => {
            let user_transactions: Vec<f64> = amounts_str
                .into_iter()
                .filter_map(|s| s.parse::<f64>().ok())
                .collect();

            println!(
                "[logging] /transactions response: user_id={}, count={}",
                user_id,
                user_transactions.len()
            );

            Ok(Json(user_transactions))
        }
        Err(e) => {
            eprintln!("[logging] Redis read failed: {:?}", e);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}
