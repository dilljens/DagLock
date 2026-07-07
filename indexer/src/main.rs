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

    let explorer_base_url = std::env::var("EXPLORER_BASE_URL")
        .unwrap_or_else(|_| "https://kas.fyi".to_string());

    let email_service = {
        let smtp_host = std::env::var("SMTP_HOST").ok();
        let smtp_port = std::env::var("SMTP_PORT").ok().and_then(|p| p.parse().ok());
        let smtp_user = std::env::var("SMTP_USER").ok();
        let smtp_pass = std::env::var("SMTP_PASS").ok();
        let from_addr = std::env::var("NOTIFICATION_FROM").ok();

        if smtp_host.is_some() && from_addr.is_some() {
            info!("Email notifications configured (SMTP: {:?}, From: {:?})", smtp_host, from_addr);
            Some(std::sync::Arc::new(crate::services::email::EmailService::new(
                smtp_host, smtp_port, smtp_user, smtp_pass, from_addr,
                "https://daglock.com".to_string(),
            )))
        } else {
            warn!("Email notifications disabled — set SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASS, NOTIFICATION_FROM to enable");
            None
        }
    };

    let anchor_service = Arc::new(crate::services::anchor::AnchorService::new(
        pool.clone(),
        args.wrpc_url.clone(),
        args.anchor_wallet_key.clone(),
    ));
    let anchor_bg = anchor_service.clone();

    let rate_limiter = Arc::new(crate::ratelimit::RateLimiter::new());

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
        explorer_base_url,
        email_service,
        ai_mediator_api_key: args.ai_mediator_api_key.clone(),
        ai_mediator_model: Some(args.ai_mediator_model.clone()),
        mock_chat_sig: args.mock_chat_sig,
        anchor_service,
        rate_limiter,
        admin_token: args.admin_token.clone(),
    };

    spawn_background_tasks(&args, &state, anchor_bg).await;

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

/// Spawn all background task loops. Delegates to focused sub-functions for each domain.
async fn spawn_background_tasks(args: &Args, state: &AppState, anchor_bg: Arc<crate::services::anchor::AnchorService>) {
    spawn_anchor_flush(anchor_bg, args.anchor_interval_seconds);
    spawn_wrpc_listener(args, state).await;
    spawn_offer_reconciler(state.db.clone());
    spawn_mediation_escalator(state.db.clone());
    spawn_dispute_escalator(state.db.clone(), args.auto_escalate_disputes);
    spawn_evidence_wiper(state.db.clone(), args.evidence_auto_wipe_hours);
    spawn_auto_settler(state, args.auto_settle_escrows);
    spawn_daily_stats(state.db.clone(), args.stats_interval_seconds);
    crate::services::price_oracle::spawn(state.db.clone());
    spawn_price_alert_checker(state, args.price_alerts_enabled);
    spawn_deposit_sweeper(state.db.clone(), args.auto_sweep_deposits);
}

fn spawn_anchor_flush(anchor_service: Arc<crate::services::anchor::AnchorService>, interval_secs: u64) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
        loop { interval.tick().await; anchor_service.flush_pending().await; }
    });
}

async fn spawn_wrpc_listener(args: &Args, state: &AppState) {
    if let Some(wrpc_url) = state.wrpc_url.clone() {
        crate::listener::spawn(
            wrpc_url.clone(), state.db.clone(), state.network.clone(),
            state.daglock_kas_template.clone(), state.daglock_krc20_template.clone(),
        );
        if args.auto_sweep_vaults {
            let sweep_wrpc = match crate::listener::try_connect_wrpc(&wrpc_url, &args.network).await {
                Ok(c) => Some(c),
                Err(e) => { warn!("Vault sweep wRPC connection failed: {}", e); None }
            };
            crate::listener::spawn_vault_sweeper(state.db.clone(), sweep_wrpc, state.treasury_pubkey.clone());
        }
    } else if !args.no_wrpc {
        match crate::listener::try_connect_resolver(&args.network).await {
            Ok(client) => {
                let db = state.db.clone();
                let kas = state.daglock_kas_template.as_ref().and_then(|h| hex::decode(h).ok());
                let krc20 = state.daglock_krc20_template.as_ref().and_then(|h| hex::decode(h).ok());
                let net = state.network.clone();
                tokio::spawn(async move {
                    crate::listener::run_online_loop_with_reconnect(client, db, kas, krc20, "resolver://auto", &net).await;
                });
            }
            Err(e) => warn!("Resolver connection failed for listener: {e} — running without block scanning"),
        }
    }
}

fn spawn_offer_reconciler(db: sqlx::Pool<sqlx::Sqlite>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            match crate::db::queries::reconcile_expired_offers(&db).await {
                Ok(count) if count > 0 => info!("Expired {count} stale offers"),
                Ok(_) => {}
                Err(e) => warn!("Failed to reconcile expired offers: {e}"),
            }
        }
    });
}

fn spawn_mediation_escalator(db: sqlx::Pool<sqlx::Sqlite>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let now = chrono::Utc::now().timestamp();
            let cases = match crate::db::queries::find_expired_mediations(&db, now).await {
                Ok(c) => c,
                Err(e) => { warn!("Failed to find expired mediations: {e}"); continue; }
            };
            for (escrow_id, amount_sompi) in &cases {
                let escrow = match crate::db::queries::get_escrow(&db, escrow_id).await {
                    Ok(Some(e)) => e, _ => continue,
                };
                if escrow.dispute_mode.as_deref() == Some("jury") {
                    let (juror_count, threshold) = crate::api::jury::juror_count_and_threshold(*amount_sompi);
                    let eligible = match crate::db::queries::list_eligible_jurors_simple(&db).await {
                        Ok(e) => e, Err(e) => { warn!("Failed to list jurors: {e}"); continue; }
                    };
                    if eligible.len() < juror_count as usize {
                        warn!("Need {juror_count} jurors for {}", escrow_id);
                        let _ = crate::db::queries::mark_mediation_escalated(&db, escrow_id).await;
                        continue;
                    }
                    let pool = eligible.iter().take((juror_count as usize).saturating_mul(2).min(eligible.len())).collect::<Vec<_>>();
                    let needed = (juror_count as usize).min(pool.len());
                    let mut indices: Vec<usize> = (0..pool.len()).collect();
                    for i in (pool.len() - needed..pool.len()).rev() {
                        let j = rand::random::<usize>() % (i + 1);
                        indices.swap(i, j);
                    }
                    let selected: Vec<String> = indices[pool.len() - needed..].iter().map(|&i| pool[i].address.clone()).collect();
                    if let Err(e) = crate::db::queries::create_jury_case(&db, escrow_id, juror_count, threshold, &selected).await {
                        warn!("Failed to create jury case: {e}"); continue;
                    }
                }
                let _ = crate::db::queries::mark_mediation_escalated(&db, escrow_id).await;
            }
        }
    });
}

fn spawn_dispute_escalator(db: sqlx::Pool<sqlx::Sqlite>, enabled: bool) {
    if !enabled { return; }
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let now = chrono::Utc::now().timestamp();
            let cases = match crate::db::queries::find_escalatable_cases(&db, now).await {
                Ok(c) => c, Err(e) => { warn!("Auto-escalate query failed: {}", e); continue; }
            };
            for (case_id, level, _status) in &cases {
                let new_level = level + 1;
                let deadline = match new_level { 1 => now + 432_000, 2 => now + 864_000, _ => continue };
                if let Err(e) = crate::db::queries::update_escalation_level(&db, case_id, new_level, deadline).await {
                    warn!("Failed to escalate case {}: {}", case_id, e); continue;
                }
                if new_level == 2 {
                    let _ = crate::db::queries::auto_decide_case(&db, case_id, "seller_wins", now).await;
                }
            }
        }
    });
}

fn spawn_evidence_wiper(db: sqlx::Pool<sqlx::Sqlite>, wipe_hours: u64) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            let now = chrono::Utc::now().timestamp();
            let reveals = match crate::db::queries::get_active_reveals(&db).await {
                Ok(r) => r, Err(e) => { warn!("Failed to get reveals: {e}"); continue; }
            };
            for (case_id, revealed_at) in &reveals {
                if now - revealed_at > (wipe_hours as i64 * 3600) {
                    if let Err(e) = crate::db::queries::clear_evidence(&db, case_id).await {
                        warn!("Failed to auto-wipe evidence for case {}: {}", case_id, e);
                    }
                }
            }
        }
    });
}

fn spawn_auto_settler(state: &AppState, enabled: bool) {
    if !enabled { return; }
    let db = state.db.clone();
    let ws_tx = state.ws_tx.clone();
    let sig_verifier = state.sig_verifier.clone();
    let verifier = state.verifier.clone();
    let email_service = state.email_service.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let escrows = match crate::db::queries::escrows::find_auto_settleable_escrows(&db).await {
                Ok(e) => e, Err(e) => { warn!("Auto-settle query failed: {e}"); continue; }
            };
            for escrow in &escrows {
                let svc = crate::services::escrow_service::EscrowService::new(db.clone(), &ws_tx, sig_verifier.clone(), verifier.clone(), email_service.clone());
                if let Err(e) = svc.auto_settle(&escrow.id).await {
                    warn!("Auto-settle failed for {}: {e}", escrow.id);
                }
            }
        }
    });
}

fn spawn_daily_stats(db: sqlx::Pool<sqlx::Sqlite>, interval: u64) {
    tokio::spawn(async move {
        if let Err(e) = crate::db::queries::compute_and_store_daily_stats(&db).await {
            tracing::warn!("Failed to compute daily stats: {e}");
        }
        let mut timer = tokio::time::interval(std::time::Duration::from_secs(interval));
        loop {
            timer.tick().await;
            if let Err(e) = crate::db::queries::compute_and_store_daily_stats(&db).await {
                tracing::warn!("Failed to compute daily stats: {e}");
            }
        }
    });
}

fn spawn_price_alert_checker(state: &AppState, enabled: bool) {
    if !enabled { return; }
    let db = state.db.clone();
    let ws_tx = state.ws_tx.clone();
    let email_service = state.email_service.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            if let Some(price) = crate::types::fetch_kas_usd_price().await {
                crate::services::price_alerts::check_alerts(&db, price, &ws_tx, &email_service).await;
            }
        }
    });
}

fn spawn_deposit_sweeper(db: sqlx::Pool<sqlx::Sqlite>, enabled: bool) {
    if !enabled { return; }
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let deposits = match crate::db::queries::find_stale_deposits(&db).await {
                Ok(d) => d, Err(e) => { warn!("Failed to find stale deposits: {e}"); continue; }
            };
            for dep in &deposits {
                if let Err(e) = crate::db::queries::sweep_deposit(&db, &dep.id).await {
                    warn!("Failed to sweep deposit {}: {e}", dep.id);
                }
            }
        }
    });
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
