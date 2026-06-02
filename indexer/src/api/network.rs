//! Network and fee endpoints.

use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::api::AppState;
use crate::db::queries;
use crate::types::{FeeEstimate, NetworkInfo};

/// GET /v1/network
pub async fn get(State(state): State<AppState>) -> Json<Value> {
    let (block_count, settled_count, _avg_fee_kas) = queries::get_network_counts(&state.db)
        .await
        .unwrap_or((0, 0, 0.0));

    Json(json!(NetworkInfo {
        network: state.network,
        daa_score: settled_count,
        block_count,
        difficulty: 0.0,
        bps: 10.0,
        daglock_kas_template_hash: state.daglock_kas_template,
        daglock_krc20_template_hash: state.daglock_krc20_template,
    }))
}

/// GET /v1/fees/estimate?amount_kas=...
pub async fn fees_estimate(
    State(_state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<Value> {
    let amount_kas = params
        .get("amount_kas")
        .cloned()
        .unwrap_or_else(|| "0".to_string());
    let amount_value = amount_kas.parse::<f64>().unwrap_or(0.0);
    let fee_kas = amount_value / 200.0;

    Json(json!(FeeEstimate {
        amount_kas,
        fee_kas: fee_kas.to_string(),
        fee_percentage: 0.5,
        network_fee_estimate: "0.00001".to_string(),
        miner_fee_budget: "0.00001".to_string(),
    }))
}
