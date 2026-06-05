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
use std::time::Instant;
use tokio::sync::broadcast;

use clap::Parser;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::api::{build_router, AppState};
use crate::config::Args;
use crate::db::init_pool;

#[tokio::main]
async fn main() {
    let mut args = Args::parse();

    // Railway injects PORT env var — use it as the source of truth
    // (standard Heroku/Railway pattern: PORT env overrides hardcoded defaults)
    if let Ok(port_str) = std::env::var("PORT") {
        if let Ok(port) = port_str.parse::<u16>() {
            args.port = port;
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&args.log_level))
        .init();

    info!("Starting DagLock Indexer v{}", env!("CARGO_PKG_VERSION"));

    // Production safety: refuse mainnet without explicit --allow-mainnet flag
    if args.network == "mainnet" && !args.allow_mainnet {
        panic!("DagLock refuses to start on mainnet without --allow-mainnet flag. Set --allow-mainnet to acknowledge production risk.");
    }

    let pool = init_pool(&args.database_url)
        .await
        .expect("Failed to initialize database");

    info!("Database ready: {}", args.database_url);

    // Initialize on-chain verifier — use MockVerifier for now
    // wRPC verifier — wired when the listener is connected to a Kaspa node
    let verifier: Arc<dyn crate::verification::EscrowVerifier> =
        Arc::new(crate::verification::MockVerifier);

    // Initialize signature verifier — chooses mock or real based on --mock-auth
    // Panics if --mock-auth is combined with --network mainnet
    let sig_verifier: Arc<dyn crate::auth::SignatureVerifier> =
        Arc::from(crate::auth::create_verifier(&args.network, args.mock_auth));

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
        sig_verifier,
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

    let app = build_router(state, &args.cors_origin);

    let addr = format!("{}:{}", args.host, args.port);
    info!("REST API listening on http://{addr}");

    let listener_tcp = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TCP listener");

    axum::serve(listener_tcp, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Failed to start HTTP server");
}

/// Handle graceful shutdown on SIGTERM/SIGINT.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutting down gracefully...");
}
