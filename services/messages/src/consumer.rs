use std::env;

use futures_util::StreamExt;
use rdkafka::{
    config::ClientConfig,
    consumer::{CommitMode, Consumer, StreamConsumer},
    message::Message,
};
use shared::models::TransactionCommand;

use crate::router::AppState;

async fn execute_transaction(state: &AppState, payload: &TransactionCommand) -> Result<(), sqlx::Error> {
    let query = r#"
        INSERT INTO user_balances (user_id, balance)
        VALUES ($1, $2)
        ON CONFLICT (user_id)
        DO UPDATE SET balance = user_balances.balance + EXCLUDED.balance
    "#;

    sqlx::query(query)
        .bind(&payload.user_id)
        .bind(payload.amount)
        .execute(&state.db)
        .await?;

    Ok(())
}

pub async fn run_consumer(state: AppState) {
    let kafka_brokers = state.kafka_brokers.clone();
    let kafka_topic = state.kafka_transactions_topic.clone();
    let consumer_group =
        env::var("KAFKA_CONSUMER_GROUP").unwrap_or_else(|_| "messages-service-group".to_string());

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &kafka_brokers)
        .set("group.id", &consumer_group)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("failed to create kafka consumer");

    consumer
        .subscribe(&[&kafka_topic])
        .expect("failed to subscribe to kafka topic");

    let mut stream = consumer.stream();

    while let Some(message_result) = stream.next().await {
        let message = match message_result {
            Ok(message) => message,
            Err(error) => {
                eprintln!("failed to receive kafka message: {error}");
                continue;
            }
        };

        let payload = match message.payload_view::<str>() {
            Some(Ok(payload)) => payload,
            Some(Err(error)) => {
                eprintln!("failed to decode kafka payload as utf-8: {error}");
                let _ = consumer.commit_message(&message, CommitMode::Async);
                continue;
            }
            None => {
                eprintln!("received kafka message with empty payload");
                let _ = consumer.commit_message(&message, CommitMode::Async);
                continue;
            }
        };

        let transaction = match serde_json::from_str::<TransactionCommand>(payload) {
            Ok(transaction) => transaction,
            Err(error) => {
                eprintln!("failed to parse kafka payload: {error}");
                let _ = consumer.commit_message(&message, CommitMode::Async);
                continue;
            }
        };

        if transaction.user_id.is_empty()
            || transaction.amount.is_nan()
            || transaction.transaction_id.is_empty()
        {
            eprintln!("received invalid transaction payload");
            let _ = consumer.commit_message(&message, CommitMode::Async);
            continue;
        }

        match execute_transaction(&state, &transaction).await {
            Ok(_) => {
                if let Err(error) = consumer.commit_message(&message, CommitMode::Async) {
                    eprintln!("failed to commit kafka offset: {error}");
                }
            }
            Err(error) => {
                eprintln!("failed to execute transaction from kafka: {error}");
            }
        }
    }
}
