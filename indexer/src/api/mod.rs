//! REST API routes for the DagLock indexer.

pub mod compile;
pub mod escrows;
pub mod evidence;
pub mod identity;
pub mod jury;
pub mod messages;
pub mod network;
pub mod offers;
pub mod receipts;
pub mod reputation;
pub mod vaults;
pub mod vouches;

use crate::verification::EscrowVerifier;
use crate::websocket;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Sqlite>,
    pub started_at: std::time::Instant,
    pub network: String,
    pub wrpc_url: Option<String>,
    pub daglock_kas_template: Option<String>,
    pub daglock_krc20_template: Option<String>,
    /// On-chain verification service.
    /// Uses MockVerifier for now; replace with WrpcVerifier when wRPC is ready.
    pub verifier: Arc<dyn EscrowVerifier>,
    /// WebSocket broadcast channel.
    pub ws_tx: tokio::sync::broadcast::Sender<crate::websocket::WsEvent>,
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
        let origin = cors_origin
            .parse::<axum::http::HeaderValue>()
            .expect("Invalid CORS origin");
        CorsLayer::new()
            .allow_origin(AllowOrigin::predicate(move |o, _| o == origin))
            .allow_methods(Any)
            .allow_headers(Any)
    };

    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/network", get(network::get))
        .route("/v1/network/price", get(network::price))
        .route("/v1/fees/estimate", get(network::fees_estimate))
        .route("/v1/compile", post(compile::compile))
        .route("/v1/escrows", get(escrows::list).post(escrows::create))
        .route("/v1/escrows/:id", get(escrows::get_by_id))
        .route("/v1/escrows/:id/settle", post(escrows::settle))
        .route("/v1/escrows/:id/refund", post(escrows::refund))
        .route("/v1/escrows/:id/dispute", post(escrows::dispute))
        .route("/v1/escrows/:id/cancel", post(escrows::cancel))
        .route("/v1/escrows/:id/swap", post(escrows::atomic_swap))
        .route(
            "/v1/escrows/:id/evidence",
            post(evidence::submit_evidence).get(evidence::list_evidence),
        )
        .route(
            "/v1/escrows/:id/resolve-dispute",
            post(evidence::resolve_dispute),
        )
        .route(
            "/v1/escrows/:id/messages",
            post(messages::send).get(messages::list),
        )
        .route("/v1/stats", get(escrows::stats))
        .route("/v1/identity", post(identity::create_identity))
        .route("/v1/offers", get(offers::list).post(offers::create))
        .route("/v1/offers/:id/accept", post(offers::accept))
        .route("/v1/offers/:id/cancel", post(offers::cancel))
        .route("/v1/reputation/:address", get(reputation::get))
        .route("/v1/receipts/:id", get(receipts::get))
        .route("/v1/vouches", post(vouches::create).get(vouches::list))
        .route("/v1/vouches/:id", post(vouches::delete))
        .route("/v1/vaults", get(vaults::list).post(vaults::create))
        .route("/v1/vaults/:id", get(vaults::get_by_id))
        .route("/v1/vaults/:id/withdraw", post(vaults::withdraw))
        .route("/v1/jury/register", post(jury::register))
        .route("/v1/jury/unregister", post(jury::unregister))
        .route("/v1/jury/cases", get(jury::list_cases))
        .route("/v1/jury/cases/:id", get(jury::get_case))
        .route("/v1/jury/cases/:id/vote", post(jury::cast_vote))
        .route("/v1/jury/candidates", get(jury::list_candidates))
        .route("/v1/ws", get(websocket_handler))
        .layer(cors)
        .with_state(state)
}

/// WebSocket upgrade handler.
async fn websocket_handler(
    ws: axum::extract::WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> axum::response::Response {
    let rx = state.ws_tx.subscribe();
    ws.on_upgrade(move |socket| websocket::handle_socket(socket, state.db, rx))
}

async fn health(axum::extract::State(state): axum::extract::State<AppState>) -> Json<Value> {
    let uptime = state.started_at.elapsed().as_secs();
    Json(json!({
        "status": "ok",
        "version": "0.1.0",
        "node_synced": state.wrpc_url.is_some(),
        "node_daa_score": 0,
        "uptime_seconds": uptime,
    }))
}
