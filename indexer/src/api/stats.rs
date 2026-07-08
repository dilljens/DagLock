//! Analytics dashboard endpoints — daily stats and live summary.

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::api::AppState;
use crate::db::queries;

#[derive(Debug, Deserialize)]
pub struct DailyParams {
    days: Option<i64>,
}

/// GET /v1/stats/daily?days=30
pub async fn daily(
    State(state): State<AppState>,
    Query(params): Query<DailyParams>,
) -> Json<Value> {
    let days = params.days.unwrap_or(30).max(1).min(365);
    match queries::get_daily_stats(&state.db, days).await {
        Ok(stats) => Json(json!({ "stats": stats, "days": days })),
        Err(e) => {
            tracing::error!("Failed to fetch daily stats: {e}");
            Json(json!({ "error": "failed_to_fetch_stats", "stats": [], "days": days }))
        }
    }
}

/// POST /v1/stats/compute — trigger one-shot daily stats computation.
/// Useful for backfilling or forcing a refresh without restarting the indexer.
pub async fn compute(State(state): State<AppState>) -> Json<Value> {
    match queries::compute_and_store_daily_stats(&state.db).await {
        Ok(_) => {
            tracing::info!("Daily stats computed on demand");
            Json(json!({ "status": "ok", "message": "Daily stats computed successfully" }))
        }
        Err(e) => {
            tracing::error!("Failed to compute daily stats on demand: {e}");
            Json(json!({ "status": "error", "message": format!("{e}") }))
        }
    }
}

/// GET /v1/stats/summary
pub async fn summary(State(state): State<AppState>) -> Json<Value> {
    let uptime = state.started_at.elapsed().as_secs() as i64;
    match queries::get_live_summary(&state.db).await {
        Ok(mut s) => {
            s.uptime_seconds = uptime;
            Json(json!(s))
        }
        Err(e) => {
            tracing::error!("Failed to fetch live summary: {e}");
            Json(json!({
                "total_escrows": 0, "total_volume_sompi": 0, "total_fees_sompi": 0,
                "active_escrows": 0, "total_users": 0, "open_offers": 0,
                "uptime_seconds": uptime
            }))
        }
    }
}
