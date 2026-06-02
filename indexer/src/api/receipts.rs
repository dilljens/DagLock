//! Settlement receipt endpoints.

use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};

use crate::api::AppState;
use crate::db::queries;
use crate::types::ApiError;

/// GET /v1/receipts/{id}
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_receipt_escrow(&state.db, &id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new("internal_error", e.to_string()))),
            )
        })?;

    match escrow {
        Some(escrow) => Ok(Json(json!(queries::receipt_from_escrow(&escrow)))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "receipt_not_found",
                format!("No receipt found for escrow '{id}'")
            ))),
        )),
    }
}
