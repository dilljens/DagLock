//! Reputation API handler.

use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};

use crate::api::AppState;
use crate::db::queries;
use crate::types::*;

/// GET /v1/reputation/{address}
pub async fn get(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rep = queries::get_reputation(&state.db, &address)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new("internal_error", "An internal error occurred."))),
            )
        })?;

    Ok(Json(json!(rep)))
}
