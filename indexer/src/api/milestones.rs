//! Milestone escrow API handlers.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::api::AppState;
use crate::auth::{parse_message, verify_nonce, AuthContext};
use crate::db::queries;
use crate::types::*;

/// Verify that the caller is authorized to act on a milestone escrow.
/// The caller must be either the buyer or seller, and must provide a valid
/// Schnorr signature for the action.
async fn verify_milestone_auth(
    state: &AppState,
    headers: &axum::http::HeaderMap,
    escrow: &MilestoneEscrow,
    action: &str,
) -> Result<(), (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(headers).map_err(|e| {
        (StatusCode::UNAUTHORIZED, Json(json!(crate::types::ApiError::new("unauthorized", e.to_string()))))
    })?;

    // Check caller is buyer or seller
    let is_buyer = auth.address == escrow.buyer_address;
    let is_seller = auth.address == escrow.seller_address;
    if !is_buyer && !is_seller {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(crate::types::ApiError::new("forbidden", "Only escrow parties can perform this action"))),
        ));
    }

    let parsed = parse_message(&auth.message).map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(json!(crate::types::ApiError::new("invalid_message", e.to_string()))))
    })?;

    if parsed.action != action {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(crate::types::ApiError::new("forbidden", format!("Message must be '{action}:{{id}}:ts:nonce'")))),
        ));
    }

    if !state.sig_verifier.verify_signature(&auth.address, &auth.signature, &auth.message)
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(crate::types::ApiError::new("forbidden", "Invalid signature"))),
        ));
    }

    verify_nonce(&state.db, &parsed, &auth.address).await.map_err(|e| {
        (StatusCode::FORBIDDEN, Json(json!(crate::types::ApiError::new("forbidden", e.to_string()))))
    })?;

    Ok(())
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub address: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// POST /v1/milestones
pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateMilestoneRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    if body.buyer_address == body.seller_address {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "self_referential", "message": "Buyer and seller must be different"})),
        ));
    }
    if body.total_amount <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_amount", "message": "Total amount must be positive"})),
        ));
    }
    if body.milestone_amounts.is_empty() || body.milestone_amounts.len() > 5 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_milestones", "message": "Must have 1-5 milestones"})),
        ));
    }
    if body.milestone_amounts.len() != body.milestone_timeouts.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_milestones", "message": "Milestone amounts and timeouts must match"})),
        ));
    }
    let total: i64 = body.milestone_amounts.iter().sum();
    if total != body.total_amount {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_milestones", "message": "Milestone amounts must sum to total_amount"})),
        ));
    }

    let milestone_statuses: Vec<String> = body
        .milestone_amounts
        .iter()
        .map(|_| "pending".to_string())
        .collect();

    let id = generate_id("ms_");
    let now = chrono::Utc::now().timestamp();

    let escrow = MilestoneEscrow {
        id,
        lock_tx_id: body.lock_tx_id,
        buyer_address: body.buyer_address,
        seller_address: body.seller_address,
        total_amount: body.total_amount,
        milestone_amounts: body.milestone_amounts,
        milestone_timeouts: body.milestone_timeouts,
        current_milestone: 0,
        milestone_statuses,
        status: "active".to_string(),
        created_at: now,
        completed_at: None,
    };

    queries::insert_milestone_escrow(&state.db, &escrow)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": format!("{e}")})),
            )
        })?;

    Ok((StatusCode::CREATED, Json(json!(escrow))))
}

/// GET /v1/milestones?address=...
pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let address = params.address.as_deref().unwrap_or("");
    if address.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_address", "message": "address query parameter is required"})),
        ));
    }

    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let (milestones, total) = queries::list_milestones_by_address(&state.db, address, limit, offset)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "internal_error", "message": "An internal error occurred."})),
            )
        })?;

    Ok(Json(json!({
        "milestones": milestones,
        "total": total,
        "limit": limit,
        "offset": offset,
    })))
}

/// GET /v1/milestones/:id
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_milestone_escrow(&state.db, &id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "internal_error", "message": "An internal error occurred."})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "not_found", "message": format!("No milestone escrow found with id '{}'", id)})),
            )
        })?;

    Ok(Json(json!(escrow)))
}

/// POST /v1/milestones/:id/release
///
/// Requires authentication as the seller:
/// - X-Daglock-Address: Seller's Kaspa address
/// - X-Daglock-Signature: Schnorr signature of "release:{id}:{timestamp}:{nonce}"
/// - X-Daglock-Message: The signed message
pub async fn release_milestone(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut escrow = queries::get_milestone_escrow(&state.db, &id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "internal_error", "message": "An internal error occurred."})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "not_found", "message": format!("No milestone escrow found with id '{}'", id)})),
            )
        })?;

    if escrow.status != "active" {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "invalid_status", "message": format!("Milestone escrow is '{}', not 'active'", escrow.status)})),
        ));
    }

    // Verify caller is authorized (seller only for release)
    verify_milestone_auth(&state, &headers, &escrow, "release").await?;

    let idx = escrow.current_milestone as usize;
    if idx >= escrow.milestone_amounts.len() {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "all_released", "message": "All milestones have been released"})),
        ));
    }

    if escrow.milestone_statuses[idx] != "pending" {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "already_released", "message": "Current milestone is not pending"})),
        ));
    }

    escrow.milestone_statuses[idx] = "released".to_string();
    escrow.current_milestone += 1;

    // Check if all milestones released
    let all_done = escrow
        .milestone_statuses
        .iter()
        .all(|s| s == "released" || s == "approved");

    if all_done {
        escrow.status = "completed".to_string();
        escrow.completed_at = Some(chrono::Utc::now().timestamp());
        queries::complete_milestone_escrow(&state.db, &id)
            .await
            .map_err(|_e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "database_error", "message": "Failed to complete milestone escrow"})),
                )
            })?;
    } else {
        queries::update_milestone_status(
            &state.db,
            &id,
            escrow.current_milestone,
            &escrow.milestone_statuses,
        )
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": "Failed to update milestone status"})),
            )
        })?;
    }

    Ok(Json(json!(escrow)))
}

/// POST /v1/milestones/:id/approve
///
/// Requires authentication as the buyer:
/// - X-Daglock-Address: Buyer's Kaspa address
/// - X-Daglock-Signature: Schnorr signature of "approve:{id}:{timestamp}:{nonce}"
/// - X-Daglock-Message: The signed message
pub async fn approve_milestone(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut escrow = queries::get_milestone_escrow(&state.db, &id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "internal_error", "message": "An internal error occurred."})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "not_found", "message": format!("No milestone escrow found with id '{}'", id)})),
            )
        })?;

    if escrow.status != "active" {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "invalid_status", "message": format!("Milestone escrow is '{}', not 'active'", escrow.status)})),
        ));
    }

    // Verify caller is authorized (buyer for approval)
    verify_milestone_auth(&state, &headers, &escrow, "approve").await?;

    let idx = escrow.current_milestone as usize;
    if idx >= escrow.milestone_amounts.len() {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "all_released", "message": "All milestones have been released"})),
        ));
    }

    if escrow.milestone_statuses[idx] != "pending" {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "already_processed", "message": "Current milestone is not pending"})),
        ));
    }

    escrow.milestone_statuses[idx] = "approved".to_string();
    escrow.current_milestone += 1;

    let all_done = escrow
        .milestone_statuses
        .iter()
        .all(|s| s == "released" || s == "approved");

    if all_done {
        escrow.status = "completed".to_string();
        escrow.completed_at = Some(chrono::Utc::now().timestamp());
        queries::complete_milestone_escrow(&state.db, &id)
            .await
            .map_err(|_e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "database_error", "message": "Failed to complete milestone escrow"})),
                )
            })?;
    } else {
        queries::update_milestone_status(
            &state.db,
            &id,
            escrow.current_milestone,
            &escrow.milestone_statuses,
        )
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": "Failed to update milestone status"})),
            )
        })?;
    }

    Ok(Json(json!(escrow)))
}

/// POST /v1/milestones/:id/dispute
///
/// Requires authentication as buyer or seller:
/// - X-Daglock-Address: Party's Kaspa address
/// - X-Daglock-Signature: Schnorr signature of "dispute:{id}:{timestamp}:{nonce}"
/// - X-Daglock-Message: The signed message
pub async fn dispute(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_milestone_escrow(&state.db, &id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "internal_error", "message": "An internal error occurred."})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "not_found", "message": format!("No milestone escrow found with id '{}'", id)})),
            )
        })?;

    if escrow.status != "active" {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "invalid_status", "message": format!("Milestone escrow is '{}', not 'active'", escrow.status)})),
        ));
    }

    // Verify caller is authorized
    verify_milestone_auth(&state, &headers, &escrow, "dispute").await?;

    queries::update_milestone_status(&state.db, &id, escrow.current_milestone, &escrow.milestone_statuses)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": "Failed to update milestone escrow"})),
            )
        })?;

    Ok(Json(json!({
        "status": "disputed",
        "escrow_id": id,
        "message": "Milestone escrow disputed. All future releases halted."
    })))
}

/// POST /v1/milestones/:id/refund
///
/// Requires authentication as the buyer:
/// - X-Daglock-Address: Buyer's Kaspa address
/// - X-Daglock-Signature: Schnorr signature of "refund:{id}:{timestamp}:{nonce}"
/// - X-Daglock-Message: The signed message
pub async fn refund(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_milestone_escrow(&state.db, &id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "internal_error", "message": "An internal error occurred."})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "not_found", "message": format!("No milestone escrow found with id '{}'", id)})),
            )
        })?;

    if escrow.status != "active" && escrow.status != "disputed" {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "invalid_status", "message": format!("Milestone escrow is '{}', cannot refund", escrow.status)})),
        ));
    }

    // Verify caller is authorized (buyer for refund)
    verify_milestone_auth(&state, &headers, &escrow, "refund").await?;

    queries::refund_milestone_escrow(&state.db, &id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": "Failed to refund milestone escrow"})),
            )
        })?;

    Ok(Json(json!({
        "status": "refunded",
        "escrow_id": id,
        "message": "Remaining funds refunded to buyer."
    })))
}

/// POST /v1/milestones/:id/complete
///
/// Requires authentication as buyer or seller (mutual):
/// - X-Daglock-Address: Party's Kaspa address
/// - X-Daglock-Signature: Schnorr signature of "complete:{id}:{timestamp}:{nonce}"
/// - X-Daglock-Message: The signed message
pub async fn complete(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_milestone_escrow(&state.db, &id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "internal_error", "message": "An internal error occurred."})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "not_found", "message": format!("No milestone escrow found with id '{}'", id)})),
            )
        })?;

    if escrow.status != "active" {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "invalid_status", "message": format!("Milestone escrow is '{}', not 'active'", escrow.status)})),
        ));
    }

    // Verify caller is authorized
    verify_milestone_auth(&state, &headers, &escrow, "complete").await?;

    queries::complete_milestone_escrow(&state.db, &id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": "Failed to complete milestone escrow"})),
            )
        })?;

    Ok(Json(json!({
        "status": "completed",
        "escrow_id": id,
        "message": "Milestone escrow completed. All funds released."
    })))
}
