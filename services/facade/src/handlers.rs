use std::{sync::Arc, time::SystemTime};

use axum::{extract::State, http::StatusCode, Json};
use reqwest::Client;
use uuid::Uuid;

use tokio;

use shared::models::{TransactionCommand, TransactionMessage, TransactionResponse};

pub async fn process_transaction(
    State(client): State<Arc<Client>>,
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
        client
            .post("http://localhost:8081/counter")
            .json(&transaction_cmd)
            .send(),
        client
            .post("http://localhost:8082/log")
            .json(&transaction_cmd)
            .send()
    );
    
    let counter_resp = match counter_result {
        Ok(resp) if resp.status().is_success() => resp.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?,
        _ => return Err(StatusCode::BAD_GATEWAY),
    };

    Ok(Json(counter_resp))
}
