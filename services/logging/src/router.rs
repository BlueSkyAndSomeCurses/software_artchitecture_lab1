use crate::handlers;
use axum::{
    routing::{get, post},
    Router,
};
use reqwest::Client as HttpClient;
use std::sync::Arc;

use std::env;

use fred::prelude::{
    Builder, Client as RedisClient, ClientLike, Config, Error as FredError, Server, ServerConfig,
};

#[derive(Clone)]
pub struct AppState {
    pub http_client: Arc<HttpClient>,
    pub redis: Option<RedisClient>,
}

pub async fn create_router() -> Result<Router, FredError> {
    let redis_password = env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set");
    let master_name = env::var("REDIS_MASTER_NAME").unwrap_or_else(|_| "mymaster".to_string());
    let sentinels_env = env::var("REDIS_SENTINELS").unwrap_or_else(|_| {
        "redis-sentinel-1:26379,redis-sentinel-2:26379,redis-sentinel-3:26379".to_string()
    });

    println!("{}", master_name);
    println!("{}", sentinels_env);

    let mut sentinel_hosts = Vec::new();
    for sentinel_host in sentinels_env.split(',') {
        if let Some((host, port_str)) = sentinel_host.split_once(':') {
            let port = port_str.parse::<u16>().unwrap_or(26379);
            sentinel_hosts.push(Server::new(host, port));
        }
    }

    let config = Config {
        server: ServerConfig::Sentinel {
            service_name: master_name.clone().into(),
            hosts: sentinel_hosts.clone(),
        },
        password: Some(redis_password.clone().into()),
        ..Default::default()
    };

    let client = Builder::from_config(config).build().unwrap();

    let redis_client = match client.init().await {
        Ok(_) => {
            println!("✅ Successfully connected to Redis Sentinel master!"); // I like those emojies
            Some(client)
        }
        Err(e) => {
            eprintln!("Redis is unavailable, starting without it: {:?}", e);
            None
        }
    };

    let state = AppState {
        http_client: Arc::new(HttpClient::new()),
        redis: redis_client,
    };

    Ok(Router::new()
        .route("/log", post(handlers::process_transaction))
        .route(
            "/transactions/{user_id}",
            get(handlers::get_user_transactions),
        )
        .with_state(state))
}
