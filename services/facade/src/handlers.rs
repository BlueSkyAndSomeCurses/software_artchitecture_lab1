use std::{
    collections::HashMap,
    time::{Instant, SystemTime},
};

use axum::{extract::State, http::StatusCode, Json};
use uuid::Uuid;

use tokio;
use tokio::sync::Mutex;

use crate::router::AppState;
use shared::models::{
    Metrics, ServiceKind, TransactionCommand, TransactionMessage, TransactionResponse,
    UserInfoResponse,
};

async fn update_timing(
    metrics: &Mutex<Metrics>,
    service: ServiceKind,
    duration: std::time::Duration,
) {
    let mut guard = metrics.lock().await;
    let timing = match service {
        ServiceKind::Counter => &mut guard.counter_time,
        ServiceKind::Logging => &mut guard.logging_time,
    };

    let last_ms = duration.as_nanos();
    *timing += last_ms;
}

pub async fn process_transaction(
    State(state): State<AppState>,
    Json(payload): Json<TransactionMessage>,
) -> Result<Json<TransactionResponse>, StatusCode> {
    if payload.user_id.is_empty() || payload.amount.is_nan() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let id = Uuid::new_v4().to_string();
    let transaction_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let transaction_id = format!("{}-{}", id, transaction_time);

    let transaction_cmd = TransactionCommand {
        transaction_id: transaction_id,
        user_id: payload.user_id,
        amount: payload.amount,
    };

    let (counter_result, log_result) = tokio::join!(
        async {
            let start = Instant::now();
            let resp = state
                .client
                .post("http://messages:8081/counter")
                .json(&transaction_cmd)
                .send()
                .await;
            (resp, start.elapsed())
        },
        async {
            let start = Instant::now();
            let resp = state
                .client
                .post("http://logging:8082/log")
                .json(&transaction_cmd)
                .send()
                .await;
            (resp, start.elapsed())
        }
    );

    let (counter_resp, counter_elapsed) = counter_result;
    let (_, log_elapsed) = log_result;
    update_timing(&state.metrics, ServiceKind::Counter, counter_elapsed).await;
    update_timing(&state.metrics, ServiceKind::Logging, log_elapsed).await;

    let counter_resp = match counter_resp {
        Ok(resp) if resp.status().is_success() => {
            resp.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?
        }
        _ => return Err(StatusCode::BAD_GATEWAY),
    };

    Ok(Json(counter_resp))
}

pub async fn get_user_info(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<UserInfoResponse>, StatusCode> {
    if user_id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let (balance_resp_time, user_resp_time) = tokio::join!(
        async {
            let start = Instant::now();
            let resp = state
                .client
                .get(&format!("http://messages:8081/user/{}", user_id))
                .send()
                .await;

            (resp, start.elapsed())
        },
        async {
            let start = Instant::now();
            let resp = state
                .client
                .get(&format!("http://logging:8082/transactions/{}", user_id))
                .send()
                .await;

            (resp, start.elapsed())
        },
    );
    let (balance_resp, counter_elapsed) = balance_resp_time;
    let (user_trasactions_resp, logging_elapsed) = user_resp_time;

    update_timing(
        &state.metrics,
        ServiceKind::Counter,
        counter_elapsed
    ).await;
    update_timing(
        &state.metrics,
        ServiceKind::Logging,
        logging_elapsed
    ).await;

    let balance = match balance_resp {
        Ok(resp) if resp.status().is_success() => resp.json::<f64>().await.ok(),
        _ => None,
    };

    let user_trasactions = match user_trasactions_resp {
        Ok(resp) if resp.status().is_success() => resp.json::<Vec<f64>>().await.ok(),
        _ => None,
    };

    return Ok(Json(UserInfoResponse {
        balance: balance.unwrap_or_default(),
        transactions: user_trasactions.unwrap_or_default(),
    }));
}

pub async fn get_accounts_balances(
    State(state): State<AppState>,
) -> Result<Json<HashMap<String, f64>>, StatusCode> {
    let counter_start = Instant::now();
    let user_balances_resp = state
        .client
        .get("http://messages:8081/accounts")
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    update_timing(
        &state.metrics,
        ServiceKind::Counter,
        counter_start.elapsed(),
    ).await;

    if !user_balances_resp.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }

    let user_balances = user_balances_resp
        .json::<HashMap<String, f64>>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(user_balances))
}

pub async fn get_timings(
    State(state): State<AppState>,
) -> Result<Json<Metrics>, StatusCode> {
    let guard = state
        .metrics
        .lock()
        .await;

    Ok(Json(guard.clone()))
}