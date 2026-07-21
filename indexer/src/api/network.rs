//! Network and fee endpoints.

use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::api::AppState;
use crate::db::queries;
use crate::types::{FeeEstimate, NetworkInfo, PriceHistoryPoint};

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

/// GET /v1/network/price
/// Returns KAS/USD price from CoinGecko (5s timeout).
pub async fn price(State(_state): State<AppState>) -> Json<Value> {
    let kas_usd = crate::types::fetch_kas_usd_price().await.unwrap_or(0.0);
    let updated_at = chrono::Utc::now().timestamp();
    Json(json!({
        "kas_usd": kas_usd,
        "updated_at": updated_at,
    }))
}

/// GET /v1/network/explorer
/// Returns the Kaspa block explorer base URL.
pub async fn explorer(State(state): State<AppState>) -> Json<Value> {
    Json(json!({ "base_url": state.explorer_base_url }))
}

/// GET /v1/network/price/history?days=30
/// Returns KAS/USD price history for charting.
pub async fn price_history(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<Value> {
    let days: i64 = params
        .get("days")
        .and_then(|d| d.parse().ok())
        .unwrap_or(30)
        .min(90)
        .max(1);
    let cutoff = chrono::Utc::now().timestamp() - days * 86400;

    let rows = match sqlx::query_as::<_, (i64, f64)>(
        "SELECT timestamp, price_usd FROM price_history \
         WHERE timestamp >= ?1 ORDER BY timestamp ASC",
    )
    .bind(cutoff)
    .fetch_all(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            return Json(json!({
                "error": { "code": "internal_error", "message": format!("{e}") }
            }));
        }
    };

    let points: Vec<PriceHistoryPoint> = rows
        .into_iter()
        .map(|(ts, price)| PriceHistoryPoint {
            timestamp: ts,
            price_usd: price,
        })
        .collect();

    Json(json!({ "points": points, "days": days }))
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
    let fee_kas = amount_value / daglock_shared::FEE_DENOMINATOR as f64;

    Json(json!(FeeEstimate {
        amount_kas,
        fee_kas: fee_kas.to_string(),
        fee_percentage: 0.5,
        network_fee_estimate: "0.00001".to_string(),
        miner_fee_budget: "0.00001".to_string(),
    }))
}
