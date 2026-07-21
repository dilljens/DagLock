#![allow(dead_code)]
//! Webhook subscription management

use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::api::AppState;
use crate::types::*;

#[derive(Deserialize)]
pub struct CreateWebhookRequest {
    pub event: String,
    pub url: String,
}

/// POST /v1/apps/:id/webhooks
pub async fn create(
    State(state): State<AppState>,
    Path(app_id): Path<String>,
    Json(body): Json<CreateWebhookRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate event name
    let valid_events = [
        "escrow.created",
        "escrow.settled",
        "escrow.refunded",
        "escrow.disputed",
        "escrow.cancelled",
        "escrow.expired",
        "offer.created",
        "offer.accepted",
    ];
    if !valid_events.contains(&body.event.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_event",
                format!(
                    "Unknown event: {}. Valid: {}",
                    body.event,
                    valid_events.join(", ")
                )
            ))),
        ));
    }

    let id = format!("wh_{}", Uuid::new_v4().to_string().replace('-', ""));
    let now = chrono::Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO webhooks (id, app_id, event, url, is_active, created_at)
         VALUES (?1, ?2, ?3, ?4, 1, ?5)",
    )
    .bind(&id)
    .bind(&app_id)
    .bind(&body.event)
    .bind(&body.url)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "Failed to create webhook"
            ))),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": id,
            "app_id": app_id,
            "event": body.event,
            "url": body.url,
            "created_at": now,
        })),
    ))
}

/// GET /v1/apps/:id/webhooks
pub async fn list(
    State(state): State<AppState>,
    Path(app_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows = sqlx::query("SELECT id, app_id, event, url, is_active, created_at FROM webhooks WHERE app_id = ?1 ORDER BY created_at DESC")
        .bind(&app_id)
        .fetch_all(&state.db)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new("internal_error", "An internal error occurred."))),
            )
        })?;

    let hooks: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.try_get::<String, _>("id").unwrap_or_default(),
                "app_id": r.try_get::<String, _>("app_id").unwrap_or_default(),
                "event": r.try_get::<String, _>("event").unwrap_or_default(),
                "url": r.try_get::<String, _>("url").unwrap_or_default(),
                "is_active": r.try_get::<bool, _>("is_active").unwrap_or(true),
                "created_at": r.try_get::<i64, _>("created_at").unwrap_or(0),
            })
        })
        .collect();

    Ok(Json(json!({ "webhooks": hooks, "total": hooks.len() })))
}

/// DELETE /v1/apps/:id/webhooks/:hook_id
pub async fn delete(
    State(state): State<AppState>,
    Path((app_id, hook_id)): Path<(String, String)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let result = sqlx::query("DELETE FROM webhooks WHERE id = ?1 AND app_id = ?2")
        .bind(&hook_id)
        .bind(&app_id)
        .execute(&state.db)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new(
                    "internal_error",
                    "Failed to delete webhook"
                ))),
            )
        })?;

    if result.rows_affected() > 0 {
        Ok(Json(json!({ "status": "deleted" })))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "webhook_not_found",
                "Webhook not found"
            ))),
        ))
    }
}
