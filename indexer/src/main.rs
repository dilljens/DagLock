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

    // Background task: flush anchor batches every N seconds
    {
        let anchor_service = anchor_bg;
        let interval_secs = args.anchor_interval_seconds;
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;
                anchor_service.flush_pending().await;
            }
        });
        info!(
            "Anchor flush task started (interval: {}s)",
            interval_secs
        );
    }

    if let Some(wrpc_url) = state.wrpc_url.clone() {
        listener::spawn(
            wrpc_url.clone(),
            state.db.clone(),
            state.network.clone(),
            state.daglock_kas_template.clone(),
            state.daglock_krc20_template.clone(),
        );
        if args.auto_sweep_vaults {
            let sweep_wrpc = match crate::listener::try_connect_wrpc(&wrpc_url, &args.network).await
            {
                Ok(c) => Some(c),
                Err(e) => {
                    warn!("Vault sweep wRPC connection failed: {}", e);
                    None
                }
            };
            listener::spawn_vault_sweeper(
                state.db.clone(),
                sweep_wrpc,
                state.treasury_pubkey.clone(),
            );
        }
    } else if !args.no_wrpc {
        // No explicit URL — try resolver auto-discovery for the listener too
        match crate::listener::try_connect_resolver(&args.network).await {
            Ok(client) => {
                info!("Listener connecting via Resolver (auto-discovery)");
                let db = state.db.clone();
                let network = state.network.clone();
                let kas_hash = state
                    .daglock_kas_template
                    .as_ref()
                    .and_then(|h| hex::decode(h).ok());
                let krc20_hash = state
                    .daglock_krc20_template
                    .as_ref()
                    .and_then(|h| hex::decode(h).ok());
                let resolved_url = "resolver://auto".to_string();
                tokio::spawn(async move {
                    crate::listener::run_online_loop_with_reconnect(
                        client,
                        db,
                        kas_hash,
                        krc20_hash,
                        &resolved_url,
                        &network,
                    )
                    .await;
                });
            }
            Err(e) => {
                warn!(
                    "Resolver connection failed for listener: {e} — running without block scanning"
                );
            }
        }
    }

    // Background task: reconcile expired offers every 5 minutes
    {
        let db = state.db.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                match crate::db::queries::reconcile_expired_offers(&db).await {
                    Ok(count) if count > 0 => {
                        info!("Expired {count} stale offers");
                    }
                    Ok(_) => {}
                    Err(e) => warn!("Failed to reconcile expired offers: {e}"),
                }
            }
        });
    }

    // Background task: auto-escalate expired mediation to jury every 60 seconds
    {
        let db = state.db.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                let now = chrono::Utc::now().timestamp();
                match crate::db::queries::find_expired_mediations(&db, now).await {
                    Ok(cases) => {
                        for (escrow_id, amount_sompi) in &cases {
                            // Get the escrow to check dispute mode
                            if let Ok(Some(escrow)) =
                                crate::db::queries::get_escrow(&db, escrow_id).await
                            {
                                // If dispute_mode was "jury", create a jury case
                                if escrow.dispute_mode.as_deref() == Some("jury") {
                                    let (juror_count, threshold) =
                                        crate::api::jury::juror_count_and_threshold(*amount_sompi);

                                    let eligible =
                                        match crate::db::queries::list_eligible_jurors_simple(&db)
                                            .await
                                        {
                                            Ok(e) => e,
                                            Err(e) => {
                                                warn!(
                                                    "Failed to list jurors for mediation escalation: {e}"
                                                );
                                                continue;
                                            }
                                        };

                                    if eligible.len() < juror_count as usize {
                                        warn!(
                                            "Need {juror_count} jurors for {} but only {} registered",
                                            escrow_id,
                                            eligible.len()
                                        );
                                        // Mark as escalated anyway
                                        let _ = crate::db::queries::mark_mediation_escalated(
                                            &db, escrow_id,
                                        )
                                        .await;
                                        continue;
                                    }

                                    let candidate_pool: Vec<_> = eligible
                                        .iter()
                                        .take(
                                            (juror_count as usize)
                                                .saturating_mul(2)
                                                .min(eligible.len()),
                                        )
                                        .collect();
                                    let pool_size = candidate_pool.len();
                                    let needed = (juror_count as usize).min(pool_size);
                                    let mut indices: Vec<usize> = (0..pool_size).collect();
                                    for i in (pool_size - needed..pool_size).rev() {
                                        let j = rand::random::<usize>() % (i + 1);
                                        indices.swap(i, j);
                                    }
                                    let selected: Vec<String> = indices[pool_size - needed..]
                                        .iter()
                                        .map(|&i| candidate_pool[i].address.clone())
                                        .collect();

                                    if let Err(e) = crate::db::queries::create_jury_case(
                                        &db,
                                        escrow_id,
                                        juror_count,
                                        threshold,
                                        &selected,
                                    )
                                    .await
                                    {
                                        warn!(
                                            "Failed to create jury case for mediation escalation: {e}"
                                        );
                                        continue;
                                    }
                                }

                                // Mark mediation as escalated
                                let _ =
                                    crate::db::queries::mark_mediation_escalated(&db, escrow_id)
                                        .await;
                                info!("Escalated expired mediation {} to jury", escrow_id);
                            }
                        }
                    }
                    Err(e) => warn!("Failed to find expired mediations: {e}"),
                }
            }
        });
    }

    // Background task: auto-escalate disputes every 60 seconds
    if args.auto_escalate_disputes {
        let db = state.db.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                let now = chrono::Utc::now().timestamp();
                match crate::db::queries::find_escalatable_cases(&db, now).await {
                    Ok(cases) => {
                        for (case_id, level, _status) in &cases {
                            let new_level = level + 1;
                            let deadline = match new_level {
                                1 => now + 432_000,
                                2 => now + 864_000,
                                _ => continue,
                            };
                            if let Err(e) = crate::db::queries::update_escalation_level(
                                &db, case_id, new_level, deadline,
                            ).await {
                                warn!("Failed to escalate case {}: {}", case_id, e);
                                continue;
                            }
                            if new_level == 2 {
                                let _ = crate::db::queries::auto_decide_case(
                                    &db, case_id, "seller_wins", now,
                                ).await;
                            }
                            info!("Escalated dispute case {} to level {} (deadline: {})", case_id, new_level, deadline);
                        }
                    }
                    Err(e) => warn!("Auto-escalate query failed: {}", e),
                }
            }
        });
    }

    // Background task: auto-wipe revealed chat evidence after resolution
    {
        let db = state.db.clone();
        let wipe_hours = args.evidence_auto_wipe_hours;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                match crate::db::queries::get_active_reveals(&db).await {
                    Ok(reveals) => {
                        let now = chrono::Utc::now().timestamp();
                        for (case_id, revealed_at) in &reveals {
                            let elapsed = now - revealed_at;
                            if elapsed > (wipe_hours as i64) * 3600 {
                                if let Err(e) =
                                    crate::db::queries::clear_evidence(&db, case_id).await
                                {
                                    warn!("Failed to auto-wipe evidence for case {}: {}", case_id, e);
                                } else {
                                    info!("Auto-wiped evidence for case {} ({}h old)", case_id, wipe_hours);
                                }
                            }
                        }
                    }
                    Err(e) => warn!("Failed to find active reveals for wipe: {e}"),
                }
            }
        });
    }

    // Background task: auto-settle eligible escrows every 60 seconds
    if args.auto_settle_escrows {
        let db = state.db.clone();
        let ws_tx = state.ws_tx.clone();
        let sig_verifier = state.sig_verifier.clone();
        let verifier = state.verifier.clone();
        let email_service = state.email_service.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                match crate::db::queries::escrows::find_auto_settleable_escrows(&db).await {
                    Ok(escrows) => {
                        for escrow in &escrows {
                            let svc = crate::services::escrow_service::EscrowService::new(
                                db.clone(),
                                &ws_tx,
                                sig_verifier.clone(),
                                verifier.clone(),
                                email_service.clone(),
                            );
                            if let Err(e) = svc.auto_settle(&escrow.id).await {
                                warn!("Auto-settle failed for {}: {e}", escrow.id);
                            } else {
                                info!("Auto-settled escrow {} via background sweeper", escrow.id);
                            }
                        }
                    }
                    Err(e) => warn!("Auto-settle query failed: {e}"),
                }
            }
        });
    }

    // Background task: compute and store daily stats every N seconds
    {
        let db = state.db.clone();
        let interval = args.stats_interval_seconds;
        tokio::spawn(async move {
            let mut timer = tokio::time::interval(std::time::Duration::from_secs(interval));
            // Run once on startup
            if let Err(e) = crate::db::queries::compute_and_store_daily_stats(&db).await {
                tracing::warn!("Failed to compute daily stats: {e}");
            }
            loop {
                timer.tick().await;
                if let Err(e) = crate::db::queries::compute_and_store_daily_stats(&db).await {
                    tracing::warn!("Failed to compute daily stats: {e}");
                } else {
                    tracing::info!("Daily stats snapshot stored");
                }
            }
        });
        tracing::info!("Daily stats background task started (interval: {interval}s)");
    }

    // Background task: price history tracking every 5 minutes
    crate::services::price_oracle::spawn(state.db.clone());
    tracing::info!("Price oracle background task started (5-minute interval)");

    // Background task: check price alerts every 5 minutes
    if args.price_alerts_enabled {
        let db = state.db.clone();
        let ws_tx = state.ws_tx.clone();
        let email_service = state.email_service.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                if let Some(price) = crate::types::fetch_kas_usd_price().await {
                    crate::services::price_alerts::check_alerts(
                        &db, price, &ws_tx, &email_service,
                    )
                    .await;
                }
            }
        });
        tracing::info!("Price alert checker background task started (5-minute interval)");
    }

    // Background task: sweep stale deposits every 60 seconds
    if args.auto_sweep_deposits {
        let db = state.db.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                match crate::db::queries::find_stale_deposits(&db).await {
                    Ok(deposits) => {
                        for dep in &deposits {
                            if let Err(e) = crate::db::queries::sweep_deposit(&db, &dep.id).await {
                                warn!("Failed to sweep deposit {}: {e}", dep.id);
                                continue;
                            }
                            info!("Swept stale deposit {} for escrow {}", dep.id, dep.escrow_id);
                        }
                        if !deposits.is_empty() {
                            info!("Swept {} stale deposit(s)", deposits.len());
                        }
                    }
                    Err(e) => warn!("Failed to find stale deposits: {e}"),
                }
            }
        });
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
