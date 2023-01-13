use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use std::{env, net::SocketAddr, str::FromStr};
use tracing::{info, Level};
use tracing_subscriber::{prelude::*, EnvFilter};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::builder().with_default_directive(Level::INFO.into()).from_env_lossy())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let listen_addr = SocketAddr::new(
        env_or_default("LISTEN_HOST", "127.0.0.1").expect("LISTEN_HOST should be an IP address"),
        env_or_default("LISTEN_PORT", "8080").expect("LISTEN_PORT should be a number"),
    );
    info!(listen_addr = listen_addr.to_string(), "starting server");

    axum::Server::bind(&listen_addr)
        .serve(
            Router::new()
                .route("/", post(handler))
                .route("/health", get(|| async { "UP" }))
                .into_make_service(),
        )
        .await
        .unwrap()
}

fn env_or_default<F: FromStr>(key: &str, default: &str) -> Result<F, F::Err> {
    env::var(key).unwrap_or_else(|_| default.to_string()).parse()
}

async fn handler(payload: Json<Value>) -> Json<Value> {
    payload
}
