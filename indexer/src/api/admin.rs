//! Admin API — moderation endpoints.
//!
//! All endpoints require `X-Daglock-Admin` header matching the server's
//! `--admin-token` CLI argument. Configured via `--admin-token <token>`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::api::AppState;
use crate::db::queries;
use crate::types::{ApiError, SetAccountFlagsRequest};

/// Verify X-Daglock-Admin header matches the configured admin token.
fn verify_admin(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<(), (StatusCode, Json<Value>)> {
    let admin_token = state.admin_token.as_deref().ok_or_else(|| {
        (
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "admin_disabled",
                "Admin endpoints are not configured on this server."
            ))),
        )
    })?;

    let header_token = headers
        .get("x-daglock-admin")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!(ApiError::new(
                    "unauthorized",
                    "X-Daglock-Admin header required."
                ))),
            )
        })?;

    if header_token != admin_token {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new("forbidden", "Invalid admin token."))),
        ));
    }

    Ok(())
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

/// GET /v1/admin/reports — list all reports
pub async fn list_reports(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    verify_admin(&state, &headers)?;

    let limit = query.limit.unwrap_or(50).min(200);
    let offset = query.offset.unwrap_or(0);

    let reports = queries::reports::list_all_reports(&state.db, limit, offset)
        .await
        .map_err(|e| {
            tracing::error!("Admin list_reports failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new("db_error", "Failed to list reports"))),
            )
        })?;

    Ok(Json(json!({
        "reports": reports,
        "total": reports.len(),
        "limit": limit,
        "offset": offset,
    })))
}

/// GET /v1/admin/blocks — list all blocks
pub async fn list_blocks(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    verify_admin(&state, &headers)?;

    let limit = query.limit.unwrap_or(50).min(200);
    let offset = query.offset.unwrap_or(0);

    let blocks = queries::blocks::list_all_blocks(&state.db, limit, offset)
        .await
        .map_err(|e| {
            tracing::error!("Admin list_blocks failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new("db_error", "Failed to list blocks"))),
            )
        })?;

    Ok(Json(json!({
        "blocks": blocks,
        "total": blocks.len(),
        "limit": limit,
        "offset": offset,
    })))
}

/// DELETE /v1/admin/blocks/:id — remove a block by ID
pub async fn delete_block(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    verify_admin(&state, &headers)?;

    let removed = queries::blocks::delete_block_by_id(&state.db, &id)
        .await
        .map_err(|e| {
            tracing::error!("Admin delete_block failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new("db_error", "Failed to delete block"))),
            )
        })?;

    if !removed {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new("not_found", "Block not found"))),
        ));
    }

    Ok(Json(json!({"status": "unblocked"})))
}

/// POST /v1/admin/flags — set account flags (bypasses owner check)
pub async fn set_flags(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<SetAccountFlagsRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    verify_admin(&state, &headers)?;

    if !body.address.starts_with("kaspa:") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_address",
                "Address must start with 'kaspa:'"
            ))),
        ));
    }

    queries::flags::upsert_account_flags(&state.db, &body)
        .await
        .map_err(|e| {
            tracing::error!("Admin set_flags failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new("db_error", "Failed to set flags"))),
            )
        })?;

    Ok(Json(json!({
        "status": "ok",
        "address": body.address,
        "is_bot": body.is_bot,
    })))
}
