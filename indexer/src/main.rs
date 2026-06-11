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
mod ratelimit;
mod services;
mod types;
mod verification;
mod websocket;

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::broadcast;

use clap::Parser;
use tracing::{info, warn};
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

    // Centralized config validation (panics on invalid config)
    args.validate();

    // Warn if DAGLOCK_MESSAGE_KEY is not set (messages stored in plaintext)
    if std::env::var("DAGLOCK_MESSAGE_KEY").is_err() {
        warn!("DAGLOCK_MESSAGE_KEY not set — encrypted messages will use a deterministic dev key. Set DAGLOCK_MESSAGE_KEY=64_hex_chars for production.");
    }

    // Warn if --mock-auth is used on testnet (insecure, but allowed)
    if args.mock_auth {
        warn!(
            "--mock-auth enabled on {}. Any signature will be accepted. Never use on mainnet.",
            args.network
        );
    }

    let pool = init_pool(&args.database_url)
        .await
        .expect("Failed to initialize database");

    info!("Database ready: {}", args.database_url);

    // Initialize on-chain verifier
    // 1) If --wrpc-url is provided, connect to that specific node
    // 2) If not, try auto-discovery via Kaspa Public Node Network (Resolver)
    // 3) Use --no-wrpc to skip connection entirely (local dev)
    // 4) Fall back to MockVerifier (always succeeds) when offline
    let verifier: Arc<dyn crate::verification::EscrowVerifier> = {
        if args.no_wrpc {
            warn!("--no-wrpc set — using mock verifier (offline mode)");
            Arc::new(crate::verification::MockVerifier)
        } else if let Some(ref wrpc_url) = args.wrpc_url {
            match crate::listener::try_connect_wrpc(wrpc_url, &args.network).await {
                Ok(client) => {
                    info!("wRPC verifier connected to {wrpc_url}");
                    Arc::new(crate::verification::WrpcVerifier::new(Some(client)))
                }
                Err(e) => {
                    warn!("Failed to connect wRPC verifier: {e} — using mock verifier");
                    Arc::new(crate::verification::MockVerifier)
                }
            }
        } else {
            match crate::listener::try_connect_resolver(&args.network).await {
                Ok(client) => {
                    info!("wRPC verifier connected via Resolver (auto-discovery)");
                    Arc::new(crate::verification::WrpcVerifier::new(Some(client)))
                }
                Err(e) => {
                    warn!("Resolver connection failed: {e} — using mock verifier");
                    Arc::new(crate::verification::MockVerifier)
                }
            }
        }
    };

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
        daglock_vault_softlock_template: args.daglock_vault_softlock_template.clone(),
        daglock_vault_multisig_template: args.daglock_vault_multisig_template.clone(),
        verifier,
        sig_verifier,
        ws_tx,
        treasury_pubkey: args.treasury_pubkey.clone(),
    };

    if let Some(wrpc_url) = state.wrpc_url.clone() {
        listener::spawn(
            wrpc_url.clone(),
            state.db.clone(),
            state.network.clone(),
            state.daglock_kas_template.clone(),
            state.daglock_krc20_template.clone(),
        );
        if args.auto_sweep_vaults {
            let sweep_wrpc = match crate::listener::try_connect_wrpc(&wrpc_url, &args.network).await {
                Ok(c) => Some(c),
                Err(e) => { warn!("Vault sweep wRPC connection failed: {}", e); None }
            };
            listener::spawn_vault_sweeper(state.db.clone(), sweep_wrpc, state.treasury_pubkey.clone());
        }
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
