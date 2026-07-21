//! Block user API handlers.
//! Users can block other users to hide their escrows and offers.

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;

#[derive(Deserialize)]
pub struct CreateBlockRequest {
    pub blocked_address: String,
    pub reason: Option<String>,
}

#[derive(Deserialize)]
pub struct BlockQuery {
    pub address: Option<String>,
}

/// POST /v1/blocks
pub async fn create(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateBlockRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": format!("{}", e)})),
        )
    })?;

    if let Some(ref reason) = body.reason {
        if reason.len() > 500 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(
                    json!({"error": "reason_too_long", "message": "Block reason must be 500 characters or less"}),
                ),
            ));
        }
    }

    if auth.address == body.blocked_address {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "self_block", "message": "You cannot block yourself"})),
        ));
    }

    let id = format!("blk_{}", Uuid::new_v4().to_string().replace('-', ""));
    queries::blocks::create_block(
        &state.db,
        &id,
        &auth.address,
        &body.blocked_address,
        body.reason.as_deref(),
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "db_error", "message": format!("{e}")})),
        )
    })?;

    Ok(Json(json!({
        "status": "blocked",
        "id": id,
        "blocked_address": body.blocked_address,
    })))
}

/// DELETE /v1/blocks/:id
pub async fn delete(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": format!("{}", e)})),
        )
    })?;

    let removed = queries::blocks::delete_block(&state.db, &id, &auth.address)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error", "message": format!("{e}")})),
            )
        })?;

    if !removed {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "not_found", "message": "Block not found or not owned by you"})),
        ));
    }

    Ok(Json(json!({"status": "unblocked"})))
}

/// GET /v1/blocks?address=
pub async fn list(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(query): Query<BlockQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": format!("{}", e)})),
        )
    })?;

    let address = query.address.as_deref().unwrap_or(&auth.address);
    let blocks = queries::blocks::list_blocks(&state.db, address)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error", "message": format!("{e}")})),
            )
        })?;

    Ok(Json(json!({
        "blocks": blocks,
        "total": blocks.len(),
    })))
}
