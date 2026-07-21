//! Multi-party escrow API handlers.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::api::AppState;
use crate::auth::{parse_message, verify_nonce, AuthContext};
use crate::db::queries;
use crate::types::*;

/// Verify that the caller is a party to a multi-party escrow.
async fn verify_multi_escrow_auth(
    state: &AppState,
    headers: &axum::http::HeaderMap,
    escrow: &MultiEscrow,
    action: &str,
) -> Result<(), (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(headers).map_err(|e| {
        (StatusCode::UNAUTHORIZED, Json(json!(crate::types::ApiError::new("unauthorized", e.to_string()))))
    })?;

    // Must be a party to the escrow
    if !escrow.parties.contains(&auth.address) {
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
pub struct CreateMultiRequest {
    pub lock_tx_id: TxId,
    pub parties: Vec<String>,
    pub shares: Vec<i64>,
    pub total_amount: i64,
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub address: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct SignRequest {
    pub address: String,
}

/// POST /v1/multi-escrows
pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateMultiRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    if body.parties.len() < 2 || body.parties.len() > 4 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_parties", "message": "Must have 2-4 parties"})),
        ));
    }
    if body.parties.len() != body.shares.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_shares", "message": "Parties and shares must match"})),
        ));
    }
    if body.total_amount <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_amount", "message": "Total amount must be positive"})),
        ));
    }
    let total_shares: i64 = body.shares.iter().sum();
    if total_shares != 10000 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_shares", "message": "Shares must sum to 10000 (100%)"})),
        ));
    }

    let id = generate_id("multi_");
    let now = chrono::Utc::now().timestamp();

    let escrow = MultiEscrow {
        id,
        lock_tx_id: body.lock_tx_id,
        parties: body.parties,
        shares: body.shares,
        total_amount: body.total_amount,
        status: "active".to_string(),
        created_at: now,
        settled_at: None,
        refunded_at: None,
        signatures: Vec::new(),
    };

    queries::insert_multi_escrow(&state.db, &escrow)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": format!("{e}")})),
            )
        })?;

    Ok((StatusCode::CREATED, Json(json!(escrow))))
}

/// GET /v1/multi-escrows?address=...
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

    let (escrows, total) = queries::list_multi_by_address(&state.db, address, limit, offset)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "internal_error", "message": "An internal error occurred."})),
            )
        })?;

    Ok(Json(json!({
        "multi_escrows": escrows,
        "total": total,
        "limit": limit,
        "offset": offset,
    })))
}

/// GET /v1/multi-escrows/:id
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_multi_escrow(&state.db, &id)
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
                Json(json!({"error": "not_found", "message": format!("No multi-party escrow found with id '{}'", id)})),
            )
        })?;

    Ok(Json(json!(escrow)))
}

/// POST /v1/multi-escrows/:id/sign
pub async fn sign(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<SignRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_multi_escrow(&state.db, &id)
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
                Json(json!({"error": "not_found", "message": format!("No multi-party escrow found with id '{}'", id)})),
            )
        })?;

    if escrow.status != "active" {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "invalid_status", "message": format!("Escrow is '{}', not 'active'", escrow.status)})),
        ));
    }

    if !escrow.parties.contains(&body.address) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "not_a_party", "message": "Address is not a party to this escrow"})),
        ));
    }

    if escrow.signatures.contains(&body.address) {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "already_signed", "message": "Address has already signed"})),
        ));
    }

    queries::record_signature(&state.db, &id, &body.address)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": "Failed to record signature"})),
            )
        })?;

    let updated = queries::get_multi_escrow(&state.db, &id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "internal_error", "message": "An internal error occurred."})),
            )
        })?
        .unwrap();

    let all_signed = updated.signatures.len() == updated.parties.len();

    Ok(Json(json!({
        "status": "signed",
        "escrow_id": id,
        "signature_count": updated.signatures.len(),
        "parties_count": updated.parties.len(),
        "all_signed": all_signed,
    })))
}

/// POST /v1/multi-escrows/:id/refund
///
/// Requires authentication as a party to the escrow:
/// - X-Daglock-Address: Party's Kaspa address
/// - X-Daglock-Signature: Schnorr signature of "refund:{id}:{timestamp}:{nonce}"
/// - X-Daglock-Message: The signed message
pub async fn refund(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_multi_escrow(&state.db, &id)
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
                Json(json!({"error": "not_found", "message": format!("No multi-party escrow found with id '{}'", id)})),
            )
        })?;

    if escrow.status != "active" {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "invalid_status", "message": format!("Escrow is '{}', not 'active'", escrow.status)})),
        ));
    }

    // Verify caller is authorized
    verify_multi_escrow_auth(&state, &headers, &escrow, "refund").await?;

    queries::refund_multi_escrow(&state.db, &id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": "Failed to refund multi-party escrow"})),
            )
        })?;

    Ok(Json(json!({
        "status": "refunded",
        "escrow_id": id,
        "message": "Multi-party escrow refunded."
    })))
}

/// POST /v1/multi-escrows/:id/swap
///
/// Requires authentication as a party to the escrow:
/// - X-Daglock-Address: Party's Kaspa address
/// - X-Daglock-Signature: Schnorr signature of "swap:{id}:{timestamp}:{nonce}"
/// - X-Daglock-Message: The signed message
pub async fn swap(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_multi_escrow(&state.db, &id)
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
                Json(json!({"error": "not_found", "message": format!("No multi-party escrow found with id '{}'", id)})),
            )
        })?;

    if escrow.status != "active" {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "invalid_status", "message": format!("Escrow is '{}', not 'active'", escrow.status)})),
        ));
    }

    // Verify caller is authorized
    verify_multi_escrow_auth(&state, &headers, &escrow, "swap").await?;

    let all_signed = escrow.signatures.len() == escrow.parties.len();
    if !all_signed {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "not_all_signed", "message": "Not all parties have signed yet"})),
        ));
    }

    queries::settle_multi_escrow(&state.db, &id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": "Failed to settle multi-party escrow"})),
            )
        })?;

    Ok(Json(json!({
        "status": "settled",
        "escrow_id": id,
        "method": "swap",
        "message": "Multi-party escrow settled via atomic swap."
    })))
}
