use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize)]
pub struct TransactionMessage {
    pub user_id: String,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionCommand {
    pub transaction_id: String,
    pub user_id: String,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub transaction_id: String,
    pub balance: f64,
}