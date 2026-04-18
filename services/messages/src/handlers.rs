use std::collections::HashMap;

use axum::{extract::State, http::StatusCode, Json};

use shared::models::{TransactionCommand, TransactionResponse};

use sqlx::Row;

use crate::router::AppState;

pub async fn get_user_balance(
    State(app_state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<f64>, StatusCode> {
    if user_id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let query = "SELECT balance FROM user_balances WHERE user_id = $1";

    let balance = sqlx::query_scalar::<_, f64>(query)
        .bind(&user_id)
        .fetch_optional(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(balance.unwrap_or(0.0)))
}

pub async fn get_user_balances(State(app_state): State<AppState>) -> Json<HashMap<String, f64>> {
    let query = "SELECT user_id, balance FROM user_balances";

    let rows = sqlx::query(query)
        .fetch_all(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap();

    let mut map = HashMap::with_capacity(rows.len());

    for row in rows {
        let user_id: String = row.get("user_id");
        let balance: f64 = row.get("balance");
        map.insert(user_id, balance);
    }

    Json(map)
}
