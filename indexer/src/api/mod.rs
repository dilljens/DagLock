//! REST API routes for the DagLock indexer.

pub mod escrows;
pub mod network;
pub mod offers;
pub mod receipts;
pub mod reputation;

use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use crate::verification::EscrowVerifier;

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
}

/// Build the Axum router with all API routes.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/network", get(network::get))
        .route("/v1/fees/estimate", get(network::fees_estimate))
        .route("/v1/escrows", get(escrows::list).post(escrows::create))
        .route("/v1/escrows/{id}", get(escrows::get_by_id))
        .route("/v1/escrows/{id}/settle", post(escrows::settle))
        .route("/v1/escrows/{id}/refund", post(escrows::refund))
        .route("/v1/escrows/{id}/dispute", post(escrows::dispute))
        .route("/v1/escrows/{id}/cancel", post(escrows::cancel))
        .route("/v1/stats", get(escrows::stats))
        .route("/v1/offers", get(offers::list).post(offers::create))
        .route("/v1/offers/{id}/accept", post(offers::accept))
        .route("/v1/offers/{id}/cancel", post(offers::cancel))
        .route("/v1/reputation/{address}", get(reputation::get))
        .route("/v1/receipts/{id}", get(receipts::get))
        .with_state(state)
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
