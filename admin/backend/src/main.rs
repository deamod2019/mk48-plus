use std::{net::SocketAddr, path::PathBuf};

use axum::{routing::{get}, Json, Router};
use serde_json::{json, Value};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    init_tracing();

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/entities", get(get_entities));

    // 默认监听 0.0.0.0:9000，便于与现有服务并行。
    let addr: SocketAddr = "0.0.0.0:9000".parse().expect("valid socket addr");
    info!("admin backend listening", %addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn init_tracing() {
    // RUST_LOG=info ./target/debug/mk48-admin-backend
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}

async fn health() -> &'static str {
    "ok"
}

async fn get_entities() -> Json<Value> {
    let path = data_file_path();
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => match serde_json::from_str::<Value>(&content) {
            Ok(v) => Json(v),
            Err(err) => {
                error!(?err, ?path, "failed to parse entities.json, returning empty structure");
                Json(empty_entities())
            }
        },
        Err(err) => {
            error!(?err, ?path, "failed to read entities.json, returning empty structure");
            Json(empty_entities())
        }
    }
}

fn empty_entities() -> Value {
    json!({
        "ships": [],
        "weapons": [],
        "sprites": []
    })
}

fn data_file_path() -> PathBuf {
    // admin/backend -> ../../data/entities.json
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/entities.json")
}
