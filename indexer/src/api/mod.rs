//! REST API routes for the DagLock indexer.

pub mod admin;
pub mod apps;
pub mod blocks;
pub mod compile;
pub mod counteroffers;
pub mod deposits;
pub mod escrows;
pub mod evidence;
pub mod feedback;
pub mod flags;
pub mod identity;
pub mod invoices;
pub mod jury;
pub mod mediator;
pub mod messages;
pub mod milestones;
pub mod multi_escrows;
pub mod network;
pub mod notifications;
pub mod offers;
pub mod pay;
pub mod price_alerts;
pub mod receipts;
pub mod reports;
pub mod reputation;
pub mod reveal;
pub mod stats;
pub mod status;
pub mod subscriptions;
pub mod swap;
pub mod tokens;
pub mod vaults;
pub mod vouches;
pub mod webhooks;

use crate::auth::SignatureVerifier;
use crate::ratelimit::RateLimiter;
use crate::verification::EscrowVerifier;
use crate::websocket;
use axum::extract::Query;
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{Pool, Sqlite};
use std::collections::HashSet;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::warn;

/// Shared health tracker for background tasks.
/// Maps task name → last heartbeat timestamp.
/// Tasks call `heartbeat("task_name")` on each loop iteration.
/// The health endpoint checks if heartbeats are recent (< 2x interval).
pub type BackgroundHealth =
    std::sync::Arc<std::sync::Mutex<std::collections::HashMap<&'static str, std::time::Instant>>>;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Sqlite>,
    pub started_at: std::time::Instant,
    pub network: String,
    pub wrpc_url: Option<String>,
    pub daglock_kas_template: Option<String>,
    pub daglock_krc20_template: Option<String>,
    pub daglock_vault_softlock_template: Option<String>,
    pub daglock_vault_multisig_template: Option<String>,
    /// On-chain verification service.
    /// Uses MockVerifier for now; replace with WrpcVerifier when wRPC is ready.
    pub verifier: Arc<dyn EscrowVerifier>,
    /// Signature verifier — MockVerifier or SchnorrVerifier based on --mock-auth.
    pub sig_verifier: Arc<dyn SignatureVerifier>,
    /// WebSocket broadcast channel.
    pub ws_tx: tokio::sync::broadcast::Sender<crate::websocket::WsEvent>,
    /// Canonical treasury public key (64 hex).
    pub treasury_pubkey: Option<String>,
    /// Kaspa block explorer base URL.
    pub explorer_base_url: String,
    /// Email notification service (optional).
    pub email_service: Option<std::sync::Arc<crate::services::email::EmailService>>,
    /// AI mediator API key (from env or CLI arg).
    pub ai_mediator_api_key: Option<String>,
    /// AI mediator model name.
    pub ai_mediator_model: Option<String>,
    /// Skip Ed25519 chat signature verification (dev mode).
    pub mock_chat_sig: bool,
    /// On-chain hash anchoring service.
    pub anchor_service: std::sync::Arc<crate::services::anchor::AnchorService>,
    /// Shared rate limiter with tier cache.
    pub rate_limiter: Arc<RateLimiter>,
    /// Admin auth token for privileged endpoints.
    pub admin_token: Option<String>,
    /// Background task heartbeat tracker for health endpoint.
    pub background_health: BackgroundHealth,
}

/// Build the Axum router with all API routes.
pub fn build_router(state: AppState, cors_origin: &str) -> Router {
    // Configure CORS for browser access
    let cors = if cors_origin == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        // Support multiple origins (comma-separated) and wildcard subdomain matching
        let allowed_origins: Vec<String> = cors_origin
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        CorsLayer::new()
            .allow_origin(AllowOrigin::predicate(
                move |origin: &axum::http::HeaderValue, _| {
                    let origin_str = origin.to_str().unwrap_or("");
                    allowed_origins.iter().any(|allowed| {
                        if allowed == origin_str {
                            return true;
                        }
                        // Wildcard subdomain: *.example.com matches anything.example.com
                        if let Some(suffix) = allowed.strip_prefix("*.") {
                            return origin_str.ends_with(suffix) || origin_str == &suffix[2..];
                        }
                        false
                    })
                },
            ))
            .allow_methods(Any)
            .allow_headers(Any)
    };

    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/status", get(status::get))
        .route("/v1/network", get(network::get))
        .route("/v1/network/price", get(network::price))
        .route("/v1/network/price/history", get(network::price_history))
        .route("/v1/network/explorer", get(network::explorer))
        .route("/v1/fees/estimate", get(network::fees_estimate))
        .route(
            "/v1/price-alerts",
            post(price_alerts::create).get(price_alerts::list),
        )
        .route("/v1/price-alerts/:id", delete(price_alerts::delete))
        .route("/v1/price-alerts/:id/trigger", patch(price_alerts::trigger))
        .route("/v1/compile", post(compile::compile))
        .route("/v1/escrows", get(escrows::list).post(escrows::create))
        .route("/v1/escrows/export", get(escrows::export_csv))
        .route("/v1/escrows/:id", get(escrows::get_by_id))
        .route("/v1/escrows/:id/lock-status", get(escrows::lock_status))
        .route("/v1/escrows/:id/settle", post(escrows::settle))
        .route("/v1/escrows/:id/refund", post(escrows::refund))
        .route("/v1/escrows/:id/dispute", post(escrows::dispute))
        .route("/v1/escrows/:id/cancel", post(escrows::cancel))
        .route("/v1/escrows/:id/swap", post(escrows::atomic_swap))
        .route("/v1/escrows/:id/auto-settle", post(escrows::auto_settle))
        .route(
            "/v1/escrows/:id/chat-pubkey",
            post(escrows::submit_chat_pubkey),
        )
        .route(
            "/v1/escrows/:id/evidence",
            post(evidence::submit_evidence).get(evidence::list_evidence),
        )
        .route(
            "/v1/escrows/:id/log-dispute-outcome",
            post(evidence::log_dispute_outcome),
        )
        .route(
            "/v1/escrows/:id/messages",
            post(messages::send).get(messages::list),
        )
        .route("/v1/escrows/:id/messages/anchors", get(messages::anchors))
        .route("/v1/escrows/:id/messages/reveal", post(reveal::reveal))
        .route("/v1/openapi.json", get(openapi_spec))
        .route(
            "/v1/subscriptions",
            get(subscriptions::list).post(subscriptions::create),
        )
        .route("/v1/subscriptions/:id", get(subscriptions::get_by_id))
        .route("/v1/subscriptions/:id/cancel", post(subscriptions::cancel))
        .route("/v1/subscriptions/:id/draw", post(subscriptions::draw))
        .route("/v1/stats", get(escrows::stats))
        .route("/v1/stats/daily", get(stats::daily))
        .route("/v1/stats/summary", get(stats::summary))
        .route("/v1/stats/compute", post(stats::compute))
        .route("/v1/identity", post(identity::create_identity))
        .route("/v1/offers", get(offers::list).post(offers::create))
        .route("/v1/offers/:id/accept", post(offers::accept))
        .route("/v1/offers/:id/cancel", post(offers::cancel))
        .route("/v1/flags/:address", get(flags::get))
        .route("/v1/flags", post(flags::set))
        .route("/v1/reputation/:address", get(reputation::get))
        .route("/v1/receipts/:id", get(receipts::get))
        .route("/v1/vouches", post(vouches::create).get(vouches::list))
        .route("/v1/vouches/:id", post(vouches::delete))
        .route("/v1/vaults", get(vaults::list).post(vaults::create))
        .route("/v1/vaults/:id", get(vaults::get_by_id))
        .route("/v1/vaults/:id/withdraw", post(vaults::withdraw))
        .route("/v1/vaults/:id/transfer", post(vaults::transfer))
        .route("/v1/vaults/:id/sweep", post(vaults::sweep_vault))
        .route("/v1/vaults/:id/relock", post(vaults::relock_vault))
        .route("/v1/vaults/:id/early-exit", post(vaults::early_exit))
        .route("/v1/vaults/:id/heir-withdraw", post(vaults::heir_withdraw))
        .route("/v1/swap/generate", post(swap::generate))
        // App management routes (API key required — use X-Daglock-Api-Key header)
        .route("/v1/apps/register", post(apps::register))
        .route("/v1/apps/:id", get(apps::get_by_id))
        .route(
            "/v1/apps/:id/keys",
            get(apps::list_keys).post(apps::create_key),
        )
        .route("/v1/apps/:id/keys/:key_id", post(apps::delete_key))
        .route("/v1/invoices", get(invoices::list).post(invoices::create))
        .route("/v1/invoices/:id", get(invoices::get))
        .route("/v1/jury/register", post(jury::register))
        .route("/v1/jury/unregister", post(jury::unregister))
        .route("/v1/jury/cases", get(jury::list_cases))
        .route("/v1/jury/cases/active/:address", get(jury::active_cases))
        .route("/v1/jury/cases/:id", get(jury::get_case))
        .route("/v1/jury/cases/:id/vote", post(jury::cast_vote))
        .route("/v1/jury/cases/:id/evidence", get(reveal::evidence))
        .route(
            "/v1/jury/cases/:id/evidence/clear",
            post(reveal::clear_evidence),
        )
        .route("/v1/jury/candidates", get(jury::list_candidates))
        // Metrics
        .route("/v1/metrics", get(crate::metrics::metrics_handler))
        .route("/v1/metrics/json", get(crate::metrics::metrics_json))
        // Blocklist & reports
        .route("/v1/blocks", get(blocks::list).post(blocks::create))
        .route("/v1/blocks/:id", post(blocks::delete))
        .route("/v1/reports", get(reports::list).post(reports::create))
        // Email notifications
        .route(
            "/v1/notifications",
            get(notifications::get).post(notifications::subscribe),
        )
        .route("/v1/notifications/verify", post(notifications::verify))
        .route(
            "/v1/notifications/preferences",
            post(notifications::update_preferences),
        )
        // Trade feedback
        .route(
            "/v1/escrows/:id/feedback",
            get(feedback::list).post(feedback::create),
        )
        // Security deposits
        .route(
            "/v1/escrows/:id/deposit",
            post(deposits::create).get(deposits::get_by_escrow),
        )
        .route("/v1/escrows/:id/deposit/release", post(deposits::release))
        .route("/v1/escrows/:id/deposit/forfeit", post(deposits::forfeit))
        .route("/v1/deposits/sweep", post(deposits::sweep))
        // Payment sessions (Escrow-as-a-Service)
        .route("/v1/pay", post(pay::create_session))
        .route("/v1/pay/:session_id", get(pay::get_session))
        .route("/v1/pay/:session_id/fund", post(pay::fund_session))
        // Counter-offers
        .route("/v1/offers/:id/counter", post(counteroffers::create))
        .route("/v1/offers/:id/counters", get(counteroffers::list))
        .route("/v1/counteroffers/:id/accept", post(counteroffers::accept))
        .route(
            "/v1/counteroffers/:id/decline",
            post(counteroffers::decline),
        )
        // Milestone escrows
        .route(
            "/v1/milestones",
            get(milestones::list).post(milestones::create),
        )
        .route("/v1/milestones/:id", get(milestones::get_by_id))
        .route(
            "/v1/milestones/:id/release",
            post(milestones::release_milestone),
        )
        .route(
            "/v1/milestones/:id/approve",
            post(milestones::approve_milestone),
        )
        .route("/v1/milestones/:id/dispute", post(milestones::dispute))
        .route("/v1/milestones/:id/refund", post(milestones::refund))
        .route("/v1/milestones/:id/complete", post(milestones::complete))
        // AI Mediation
        .route(
            "/v1/escrows/:id/mediate",
            post(mediator::mediate).get(mediator::status),
        )
        .route(
            "/v1/escrows/:id/mediate/:party/accept",
            post(mediator::accept),
        )
        // Multi-party escrows
        .route(
            "/v1/multi-escrows",
            get(multi_escrows::list).post(multi_escrows::create),
        )
        .route("/v1/multi-escrows/:id", get(multi_escrows::get_by_id))
        .route("/v1/multi-escrows/:id/sign", post(multi_escrows::sign))
        .route("/v1/multi-escrows/:id/refund", post(multi_escrows::refund))
        .route("/v1/multi-escrows/:id/swap", post(multi_escrows::swap))
        // KRC-20 token dashboard
        .route("/v1/tokens", get(tokens::list))
        .route("/v1/tokens/registered", get(tokens::registered_list))
        .route("/v1/tokens/deploy", post(tokens::deploy))
        .route("/v1/tokens/:ticker", get(tokens::get))
        .route("/v1/tokens/:ticker", patch(tokens::update))
        .route("/v1/tokens/:ticker/chart", get(tokens::chart))
        .route("/v1/ws", get(websocket_handler))
        // Admin moderation endpoints (require X-Daglock-Admin header)
        .route("/v1/admin/reports", get(admin::list_reports))
        .route("/v1/admin/blocks", get(admin::list_blocks))
        .route("/v1/admin/blocks/:id", delete(admin::delete_block))
        .route("/v1/admin/flags", post(admin::set_flags))
        // Tier management (admin only)
        .route(
            "/v1/apps/:id/keys/:key_id/tier",
            patch(apps::update_key_tier),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(request_id_middleware))
        .layer(RequestBodyLimitLayer::new(1024 * 1024)) // 1MB max body
        .route_layer(axum::middleware::from_fn_with_state(
            state.rate_limiter.clone(),
            crate::ratelimit::rate_limit_mw,
        ))
        .with_state(state.clone())
}

/// Serve the OpenAPI spec.
async fn openapi_spec() -> Json<serde_json::Value> {
    let spec = include_str!("../../static/openapi.json");
    Json(serde_json::from_str(spec).unwrap_or_default())
}

/// Query parameters for WebSocket auth.
#[derive(Deserialize)]
struct WsAuthParams {
    address: Option<String>,
    signature: Option<String>,
    message: Option<String>,
}

/// WebSocket upgrade handler.
///
/// Optional authentication via query parameters:
///   ?address=kaspa:...&signature=hex&message=action:id:ts:nonce
///
/// Without auth, the WebSocket connects but receives no events (privacy).
/// With auth, only events for escrows the caller is a participant in are delivered.
async fn websocket_handler(
    ws: axum::extract::WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<AppState>,
    Query(params): Query<WsAuthParams>,
) -> axum::response::Response {
    let rx = state.ws_tx.subscribe();

    // Attempt to authenticate from query params
    let auth_addresses = if let (Some(addr), Some(sig), Some(msg)) =
        (&params.address, &params.signature, &params.message)
    {
        if state
            .sig_verifier
            .verify_signature(addr, sig, msg)
            .unwrap_or(false)
        {
            let mut addrs = HashSet::new();
            addrs.insert(addr.clone());
            Some(addrs)
        } else {
            // Invalid signature — connect but no events
            warn!("WebSocket: invalid auth signature for {}", addr);
            None
        }
    } else {
        None
    };

    ws.on_upgrade(move |socket| websocket::handle_socket(socket, state.db, rx, auth_addresses))
}

async fn health(axum::extract::State(state): axum::extract::State<AppState>) -> Json<Value> {
    let uptime = state.started_at.elapsed().as_secs();

    // Check database connectivity
    let db_ok = sqlx::query("SELECT 1").fetch_one(&state.db).await.is_ok();

    // Check background task liveness from heartbeats
    const TASK_TIMEOUT_SECS: u64 = 600; // 10 min without heartbeat = stale
    let now = std::time::Instant::now();
    let mut tasks = serde_json::Map::new();
    if let Ok(health) = state.background_health.lock() {
        for (&name, &last_beat) in health.iter() {
            let elapsed = now.duration_since(last_beat).as_secs();
            let alive = elapsed < TASK_TIMEOUT_SECS;
            tasks.insert(
                name.to_string(),
                json!({
                    "alive": alive,
                    "last_heartbeat_secs_ago": elapsed,
                }),
            );
        }
    }

    Json(json!({
        "status": if db_ok { "ok" } else { "degraded" },
        "version": "0.1.0",
        "db_connected": db_ok,
        "node_synced": state.wrpc_url.is_some(),
        "node_daa_score": serde_json::Value::Null,
        "background_tasks": tasks,
        "uptime_seconds": uptime,
    }))
}

/// Middleware that generates a request ID and adds it to the tracing span and response headers.
async fn request_id_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    let short_id = request_id[..8].to_string();

    // Add request_id to tracing span
    let span = tracing::info_span!("request", request_id = %short_id);
    let _guard = span.enter();

    let mut response = next.run(req).await;

    // Add X-Request-Id header to response
    if let Ok(value) = axum::http::HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", value);
    }

    response
}
