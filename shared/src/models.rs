use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceIpResponse {
    pub ip: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KafkaConfigResponse {
    pub brokers: String,
    pub transactions_topic: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub user_id: String,
    pub balance: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionAcceptedResponse {
    pub transaction_id: String,
    pub user_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct UserInfoResponse {
    pub balance: f64,
    pub transactions: Vec<f64>,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Metrics {
    pub counter_time: u128,
    pub logging_time: u128,
}

#[derive(Debug, Clone, Copy)]
pub enum ServiceKind {
    Counter,
    Logging,
}
