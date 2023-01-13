use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use std::{env, net::SocketAddr, str::FromStr};
use tracing::{debug, info, Level};
use tracing_subscriber::{prelude::*, EnvFilter};

#[tokio::main]
async fn main() {
    let env_filter =
        EnvFilter::builder().with_default_directive(Level::INFO.into()).from_env_lossy();

    if env_or_default("JSON_LOG", "false").expect("JSON_LOG should be true or false") {
        tracing::subscriber::set_global_default(
            tracing_subscriber::registry().with(env_filter).with(tracing_stackdriver::layer()),
        )
        .expect("failed to set global default subscriber");
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }

    let listen_addr = SocketAddr::new(
        env_or_default("HOST", "0.0.0.0").expect("LISTEN_HOST should be an IP address"),
        env_or_default("PORT", "8080").expect("LISTEN_PORT should be a number"),
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

async fn handler(Json(payload): Json<Value>) {
    debug!(payload = serde_json::to_string(&payload).unwrap(), "message received");
}
