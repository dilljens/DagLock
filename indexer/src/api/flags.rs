//! Account flags API — set and query per-address metadata (is_bot, label).
//!
//! `GET /v1/flags/:address` — read flags for an address
//! `POST /v1/flags` — set flags for an address (requires auth as the target address)

use axum::extract::{Path, State};
use axum::Json;
use serde_json::{json, Value};

use crate::api::AppState;
use crate::db::queries;
use axum::http::StatusCode;
use crate::types::{ApiError, SetAccountFlagsRequest};

/// GET /v1/flags/:address
pub async fn get(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<Value>)> {
    match queries::flags::get_account_flags(&state.db, &address).await {
        Ok(Some(flags)) => Ok(Json(json!(flags))),
        Ok(None) => Ok(Json(json!({
            "address": address,
            "is_bot": false,
            "label": null,
            "updated_at": 0
        }))),
        Err(e) => {
            tracing::error!("Failed to get flags for {address}: {e}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new("database_error", "Failed to look up flags"))),
            ))
        }
    }
}

/// POST /v1/flags
///
/// Set account flags. The caller must authenticate as the address they're setting
/// flags for (same auth as other state-changing operations).
pub async fn set(
    State(state): State<AppState>,
    Json(body): Json<SetAccountFlagsRequest>,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<Value>)> {
    if !body.address.starts_with("kaspa:") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new("invalid_address", "Address must start with 'kaspa:'"))),
        ));
    }

    match queries::flags::upsert_account_flags(&state.db, &body).await {
        Ok(_) => Ok(Json(json!({
            "status": "ok",
            "address": body.address,
            "is_bot": body.is_bot,
        }))),
        Err(e) => {
            tracing::error!("Failed to set flags for {}: {e}", body.address);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new("database_error", "Failed to set flags"))),
            ))
        }
    }
}
