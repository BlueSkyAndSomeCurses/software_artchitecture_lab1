use std::{collections::HashMap, sync::Arc, time::SystemTime};

use axum::{extract::State, http::StatusCode, Json};
use reqwest::Client;
use uuid::Uuid;

use tokio;

use shared::models::{
    TransactionCommand, TransactionMessage, TransactionResponse, UserInfoResponse,
};

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
        Ok(resp) if resp.status().is_success() => {
            resp.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?
        }
        _ => return Err(StatusCode::BAD_GATEWAY),
    };

    Ok(Json(counter_resp))
}

pub async fn get_user_info(
    State(client): State<Arc<Client>>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<UserInfoResponse>, StatusCode> {
    if user_id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let (balance_resp, user_trasactions_resp) = tokio::join!(
        client
            .get(&format!("http://localhost:8081/user/{}", user_id))
            .send(),
        client
            .get(&format!("http://localhost:8082/transactions/{}", user_id))
            .send()
    );

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
    State(client): State<Arc<Client>>,
) -> Result<Json<HashMap<String, f64>>, StatusCode> {
    let user_balances_resp = client
        .get("http://localhost:8081/accounts")
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    
    if !user_balances_resp.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }

    let user_balances = user_balances_resp
        .json::<HashMap<String, f64>>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(user_balances))
}
