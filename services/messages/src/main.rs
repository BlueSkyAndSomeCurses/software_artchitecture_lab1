mod consumer;
mod handlers;
mod router;

#[tokio::main]
async fn main() {
    let state = router::create_state().await;
    let app = router::create_router(state.clone());

    tokio::spawn(async move {
        consumer::run_consumer(state).await;
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
