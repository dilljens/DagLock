//! DagLock Indexer — REST API for escrow tracking.
//!
//! v1: Works standalone with SQLite + REST API. wRPC listener will be added in v2.
//! For now, escrows are registered manually via the POST /v1/escrows endpoint.

mod api;
mod auth;
mod config;
mod crypto;
mod db;
mod listener;
mod types;
mod verification;
mod websocket;

use std::sync::Arc;
use tokio::sync::broadcast;
use std::time::Instant;

use clap::Parser;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::api::{build_router, AppState};
use crate::config::Args;
use crate::db::init_pool;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&args.log_level))
        .init();

    info!("Starting DagLock Indexer v{}", env!("CARGO_PKG_VERSION"));

    let pool = init_pool(&args.database_url)
        .await
        .expect("Failed to initialize database");

    info!("Database ready: {}", args.database_url);

    // Initialize verifier — use MockVerifier for now
    // TODO: Replace with WrpcVerifier when wRPC client is fully implemented
    let verifier: Arc<dyn crate::verification::EscrowVerifier> =
        Arc::new(crate::verification::MockVerifier);

    // Create WebSocket event channel
    let (ws_tx, _) = broadcast::channel(100);

    let state = AppState {
        db: pool,
        started_at: Instant::now(),
        network: args.network.clone(),
        wrpc_url: args.wrpc_url.clone(),
        daglock_kas_template: args.daglock_kas_template.clone(),
        daglock_krc20_template: args.daglock_krc20_template.clone(),
        verifier,
        ws_tx,
    };

    if let Some(wrpc_url) = state.wrpc_url.clone() {
        listener::spawn(
            wrpc_url,
            state.db.clone(),
            state.network.clone(),
            state.daglock_kas_template.clone(),
            state.daglock_krc20_template.clone(),
        );
    }

    let app = build_router(state);

    let addr = format!("{}:{}", args.host, args.port);
    info!("REST API listening on http://{addr}");

    let listener_tcp = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TCP listener");

    axum::serve(listener_tcp, app)
        .await
        .expect("Failed to start HTTP server");
}
