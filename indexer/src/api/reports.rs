//! User report API handlers.
//! Users can report bad actors. Reports are visible to jury admins.

use axum::extract::Query;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;
use crate::types::ApiError;

#[derive(Deserialize)]
pub struct CreateReportRequest {
    pub reported_address: String,
    pub escrow_id: Option<String>,
    pub reason: String,
}

#[derive(Deserialize)]
pub struct ReportQuery {
    pub address: Option<String>,
}

/// POST /v1/reports
pub async fn create(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateReportRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if body.reason.len() > 2000 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "reason_too_long",
                "Report reason must be 2000 characters or less"
            ))),
        ));
    }

    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": format!("{}", e)})),
        )
    })?;

    let id = format!("rpt_{}", Uuid::new_v4().to_string().replace('-', ""));
    queries::reports::create_report(
        &state.db,
        &id,
        &auth.address,
        &body.reported_address,
        body.escrow_id.as_deref(),
        &body.reason,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "db_error", "message": format!("{e}")})),
        )
    })?;

    Ok(Json(json!({
        "status": "reported",
        "id": id,
        "reported_address": body.reported_address,
    })))
}

/// GET /v1/reports?address=
pub async fn list(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": format!("{}", e)})),
        )
    })?;

    let address = query.address.as_deref().unwrap_or(&auth.address);
    let reports = queries::reports::list_reports(&state.db, address)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error", "message": format!("{e}")})),
            )
        })?;

    Ok(Json(json!({
        "reports": reports,
        "total": reports.len(),
    })))
}
