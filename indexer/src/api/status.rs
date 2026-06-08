//! Public status endpoint — uptime, volume, settlement count.

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use crate::api::AppState;
use crate::db::queries;

/// GET /v1/status
pub async fn get(State(state): State<AppState>) -> Json<Value> {
    let uptime = state.started_at.elapsed().as_secs();

    // Get aggregate stats from the DB
    let (total, settled, volume) = match queries::get_network_counts(&state.db).await {
        Ok((t, s, v)) => (t, s, v),
        Err(_) => (0, 0, 0.0),
    };

    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "network": state.network,
        "node_synced": state.wrpc_url.is_some(),
        "uptime_seconds": uptime,
        "total_escrows": total,
        "total_settled": settled,
        "total_volume_kas": volume,
        "last_24h": {
            "escrows_created": null,
            "escrows_settled": null,
            "volume_kas": null
        }
    }))
}
