use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::AppState;
use crate::types::{CreatePriceAlertRequest, PriceAlert};

/// POST /v1/price-alerts — create a new price alert
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreatePriceAlertRequest>,
) -> Json<Value> {
    if req.target_price <= 0.0 {
        return Json(json!({
            "error": { "code": "invalid_target_price", "message": "Target price must be positive" }
        }));
    }
    if req.direction != "above" && req.direction != "below" {
        return Json(json!({
            "error": { "code": "invalid_direction", "message": "Direction must be 'above' or 'below'" }
        }));
    }

    let id = format!("alert_{}", Uuid::new_v4().to_string().replace('-', ""));
    let now = chrono::Utc::now().timestamp();

    if let Err(e) = sqlx::query(
        "INSERT INTO price_alerts (id, address, target_price, direction, triggered, created_at) \
         VALUES (?1, ?2, ?3, ?4, 0, ?5)"
    )
    .bind(&id)
    .bind(&req.address)
    .bind(req.target_price)
    .bind(&req.direction)
    .bind(now)
    .execute(&state.db)
    .await
    {
        return Json(json!({
            "error": { "code": "internal_error", "message": format!("Failed to create alert: {e}") }
        }));
    }

    Json(json!(PriceAlert {
        id,
        address: req.address,
        target_price: req.target_price,
        direction: req.direction,
        triggered: false,
        created_at: now,
        triggered_at: None,
    }))
}

/// GET /v1/price-alerts?address=... — list user's price alerts
pub async fn list(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<Value> {
    let address = match params.get("address") {
        Some(a) => a,
        None => {
            return Json(json!({
                "error": { "code": "missing_address", "message": "address query param required" }
            }));
        }
    };

    let rows = match sqlx::query_as::<_, (String, String, f64, String, i32, i64, Option<i64>)>(
        "SELECT id, address, target_price, direction, triggered, created_at, triggered_at \
         FROM price_alerts WHERE address = ?1 ORDER BY created_at DESC"
    )
    .bind(address)
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

    let alerts: Vec<PriceAlert> = rows
        .into_iter()
        .map(|(id, addr, target_price, direction, triggered, created_at, triggered_at)| {
            PriceAlert {
                id,
                address: addr,
                target_price,
                direction,
                triggered: triggered != 0,
                created_at,
                triggered_at,
            }
        })
        .collect();

    Json(json!({ "alerts": alerts, "total": alerts.len() }))
}

/// DELETE /v1/price-alerts/:id — delete a price alert
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Value> {
    let result = sqlx::query("DELETE FROM price_alerts WHERE id = ?1")
        .bind(&id)
        .execute(&state.db)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(json!({ "status": "deleted", "alert_id": id }))
        }
        Ok(_) => {
            Json(json!({
                "error": { "code": "not_found", "message": format!("No alert found with id '{id}'") }
            }))
        }
        Err(e) => {
            Json(json!({
                "error": { "code": "internal_error", "message": format!("{e}") }
            }))
        }
    }
}

/// PATCH /v1/price-alerts/:id/trigger — mock/force-trigger an alert (admin)
pub async fn trigger(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Value> {
    let now = chrono::Utc::now().timestamp();
    let result = sqlx::query(
        "UPDATE price_alerts SET triggered = 1, triggered_at = ?1 WHERE id = ?2 AND triggered = 0"
    )
    .bind(now)
    .bind(&id)
    .execute(&state.db)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(json!({ "status": "triggered", "alert_id": id, "triggered_at": now }))
        }
        Ok(_) => {
            Json(json!({
                "error": { "code": "not_found", "message": format!("Alert '{id}' not found or already triggered") }
            }))
        }
        Err(e) => {
            Json(json!({
                "error": { "code": "internal_error", "message": format!("{e}") }
            }))
        }
    }
}
