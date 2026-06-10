//! Evidence and dispute resolution API handlers.

use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    Json,
};
use blake2b_simd::Params;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;
use crate::types::*;

/// POST /v1/escrows/{id}/evidence
///
/// Submit evidence for a disputed escrow.
/// Requires authentication headers.
pub async fn submit_evidence(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateEvidenceRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Verify escrow exists and is disputed
    let escrow = queries::get_escrow(&state.db, &id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "An internal error occurred."
            ))),
        )
    })?;

    let escrow = escrow.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "escrow_not_found",
                format!("No escrow found with id '{id}'")
            ))),
        )
    })?;

    if escrow.status != EscrowStatus::Disputed {
        return Err((
            StatusCode::CONFLICT,
            Json(json!(ApiError::new(
                "not_disputed",
                "Escrow is not in disputed state"
            ))),
        ));
    }

    // Extract authenticated address
    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new(
                "unauthorized",
                "Missing or invalid auth headers"
            ))),
        )
    })?;

    // Verify signature — proves the caller owns the claimed address
    let expected_msg = format!("evidence:{}", id);
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &expected_msg)
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "invalid_signature",
                "Signature does not match the claimed address"
            ))),
        ));
    }

    // Verify the submitter is one of the escrow parties
    if auth.address != escrow.buyer_address
        && escrow.seller_address.as_deref() != Some(&auth.address)
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "forbidden",
                "Only escrow parties can submit evidence"
            ))),
        ));
    }

    // Validate content size (max 100KB)
    if body.content.len() > 102_400 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "content_too_large",
                "Evidence content must be under 100KB"
            ))),
        ));
    }

    // Compute content hash for integrity
    let content_hash = Params::new()
        .hash_length(16)
        .to_state()
        .update(body.content.as_bytes())
        .finalize();

    let evidence = DisputeEvidence {
        id: format!(
            "ev_{}",
            Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or_default()
        ),
        escrow_id: id,
        submitted_by: auth.address,
        content: body.content,
        content_hash: hex::encode(content_hash.as_bytes()),
        signed_message: body.signed_message,
        created_at: chrono::Utc::now().timestamp(),
    };

    queries::insert_evidence(&state.db, &evidence)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new(
                    "internal_error",
                    "An internal error occurred."
                ))),
            )
        })?;

    Ok(Json(json!(evidence)))
}

/// GET /v1/escrows/{id}/evidence
///
/// List all evidence for an escrow (public).
pub async fn list_evidence(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let evidence = queries::list_evidence(&state.db, &id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "An internal error occurred."
            ))),
        )
    })?;

    Ok(Json(json!({ "evidence": evidence, "escrow_id": id })))
}

/// POST /v1/escrows/{id}/resolve-dispute
///
/// Resolve a dispute with an outcome.
/// Requires authentication headers.
pub async fn resolve_dispute(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<ResolveDisputeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate outcome
    if body.outcome != "expunge" && body.outcome != "uphold" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_outcome",
                "Outcome must be 'expunge' or 'uphold'"
            ))),
        ));
    }

    // Verify escrow exists and is disputed
    let escrow = queries::get_escrow(&state.db, &id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "An internal error occurred."
            ))),
        )
    })?;

    let escrow = escrow.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "escrow_not_found",
                format!("No escrow found with id '{id}'")
            ))),
        )
    })?;

    if escrow.status != EscrowStatus::Disputed {
        return Err((
            StatusCode::CONFLICT,
            Json(json!(ApiError::new(
                "not_disputed",
                "Escrow is not in disputed state"
            ))),
        ));
    }

    // Extract authenticated address
    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new(
                "unauthorized",
                "Missing or invalid auth headers"
            ))),
        )
    })?;

    // Verify signature — proves the caller owns the claimed address
    let expected_msg = format!("resolve:{}", id);
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &expected_msg)
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "invalid_signature",
                "Signature does not match the claimed address"
            ))),
        ));
    }

    // Verify the resolver is one of the escrow parties or the mediator
    let is_party = auth.address == escrow.buyer_address
        || escrow.seller_address.as_deref() == Some(&auth.address);
    let is_mediator = escrow.mediator_key.as_deref() == Some(&auth.address);

    if !is_party && !is_mediator {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "forbidden",
                "Only escrow parties or the mediator can resolve disputes"
            ))),
        ));
    }

    // Resolve
    queries::resolve_dispute(&state.db, &id, &body.outcome, &body.resolved_by)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new(
                    "internal_error",
                    "An internal error occurred."
                ))),
            )
        })?;

    Ok(Json(json!({
        "status": "resolved",
        "escrow_id": id,
        "outcome": body.outcome,
    })))
}
