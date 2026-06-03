//! Escrow CRUD API handlers.

use axum::http::StatusCode;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::AppState;

use crate::auth::{verify_refund_authorization, verify_settle_authorization, AuthContext};
use crate::db::queries;
use crate::types::*;
use crate::verification::{verify_escrow_refundable, verify_escrow_settleable};

/// List escrows query parameters.
#[derive(Deserialize)]
pub struct ListQuery {
    pub address: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct DisputeRequest {
    pub reason: String,
}

/// GET /v1/escrows
pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let address = params.address.as_deref().unwrap_or("");
    if address.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_address",
                "address query parameter is required"
            ))),
        ));
    }

    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let (escrows, total) = queries::list_escrows_by_address(
        &state.db,
        address,
        params.role.as_deref(),
        params.status.as_deref(),
        limit,
        offset,
    )
    .await
    .map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "An internal error occurred. Please try again later."
            ))),
        )
    })?;

    Ok(Json(json!({
        "escrows": escrows,
        "total": total,
        "limit": limit,
        "offset": offset,
    })))
}

/// GET /v1/escrows/{id}
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_escrow(&state.db, &id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "An internal error occurred. Please try again later."
            ))),
        )
    })?;

    match escrow {
        Some(e) => Ok(Json(json!(e))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "escrow_not_found",
                format!("No escrow found with id '{id}'")
            ))),
        )),
    }
}

/// Validate a Kaspa address format.
pub fn validate_kaspa_address(address: &str) -> bool {
    // Basic validation: must start with "kaspa:" and be non-empty after prefix
    if !address.starts_with("kaspa:") {
        return false;
    }
    let prefix_len = "kaspa:".len();
    if address.len() <= prefix_len {
        return false;
    }
    // The rest should be bech32 characters (qpzry9x8gf2tvdw0s3jn54khce6mua7l)
    let bech32_chars = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    address[prefix_len..]
        .chars()
        .all(|c| bech32_chars.contains(c))
}

/// POST /v1/escrows
pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateEscrowRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate required fields
    if body.amount_sompi <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_amount",
                "amount must be positive"
            ))),
        ));
    }

    // Validate buyer address
    if !validate_kaspa_address(&body.buyer_address) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_address",
                "Invalid buyer Kaspa address"
            ))),
        ));
    }

    // Validate buyer address length
    if body.buyer_address.len() > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_address",
                "Buyer address too long"
            ))),
        ));
    }

    // Validate seller address if provided
    if let Some(ref seller) = body.seller_address {
        if !validate_kaspa_address(seller) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!(ApiError::new(
                    "invalid_address",
                    "Invalid seller Kaspa address"
                ))),
            ));
        }
        if seller.len() > 100 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!(ApiError::new(
                    "invalid_address",
                    "Seller address too long"
                ))),
            ));
        }
    }

    // Validate amount range
    if body.amount_sompi > 100_000_000_000_000 {
        // 1M KAS max
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_amount",
                "Amount exceeds maximum (1M KAS)"
            ))),
        ));
    }

    let fee_sompi = body.amount_sompi / 200; // 0.5%

    let escrow = Escrow {
        id: format!(
            "esc_{}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        ),
        lock_tx_id: body.lock_tx_id,
        lock_tx_output_index: body.lock_tx_output_index,
        status: EscrowStatus::PendingConfirmation,
        asset_type: body.asset_type.unwrap_or_else(|| "KAS".to_string()),
        buyer_address: body.buyer_address,
        seller_address: body.seller_address,
        amount_sompi: body.amount_sompi,
        fee_sompi,
        template_hash: body.template_hash.unwrap_or_default(),
        expiration_daa_score: body.expiration_daa_score,
        disputed_at: None,
        dispute_reason: None,
        cancelled_at: None,
        expired_at: None,
        created_at: chrono::Utc::now().timestamp(),
        settled_at: None,
        refunded_at: None,
        mediator_key: body.mediator_key,
        dispute_outcome: None,
        dispute_resolved_at: None,
    };

    queries::insert_escrow(&state.db, &escrow)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new(
                    "internal_error",
                    "An internal error occurred. Please try again later."
                ))),
            )
        })?;

    Ok((StatusCode::CREATED, Json(json!(escrow))))
}

/// GET /v1/stats
pub async fn stats(State(state): State<AppState>) -> Json<Value> {
    match queries::get_stats(&state.db).await {
        Ok(s) => Json(json!(s)),
        Err(_e) => Json(json!(ApiError::new(
            "internal_error",
            "An internal error occurred. Please try again later."
        ))),
    }
}

/// POST /v1/escrows/{id}/settle
///
/// Requires authentication headers:
/// - X-Daglock-Address: Signer's Kaspa address
/// - X-Daglock-Signature: Hex-encoded signature
/// - X-Daglock-Message: Signed message (format: "settle:{escrow_id}")
pub async fn settle(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_escrow(&state.db, &id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "An internal error occurred. Please try again later."
            ))),
        )
    })?;

    match escrow {
        Some(current)
            if matches!(
                current.status,
                EscrowStatus::Settled
                    | EscrowStatus::Refunded
                    | EscrowStatus::Cancelled
                    | EscrowStatus::Expired
                    | EscrowStatus::PendingConfirmation
            ) =>
        {
            Err((
                StatusCode::CONFLICT,
                Json(json!(ApiError::new(
                    "escrow_already_finalized",
                    "Escrow is already finalized"
                ))),
            ))
        }
        Some(current) => {
            // Verify caller is authorized (buyer or seller with valid signature)
            let auth = AuthContext::from_headers(&headers).map_err(|e| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!(ApiError::new("unauthorized", e.to_string()))),
                )
            })?;
            // TODO: Replace MockSignatureVerifier with real verifier when key management is implemented
            let sig_verifier = crate::auth::Secp256k1Verifier::new();
            verify_settle_authorization(&current, &auth, &sig_verifier).map_err(|e| {
                (
                    StatusCode::FORBIDDEN,
                    Json(json!(ApiError::new("forbidden", e.to_string()))),
                )
            })?;

            // Verify escrow can be settled (UTXO exists on-chain)
            verify_escrow_settleable(&current, state.verifier.as_ref()).map_err(|e| {
                (
                    StatusCode::CONFLICT,
                    Json(json!(ApiError::new("verification_failed", e.to_string()))),
                )
            })?;

            // Atomic update: status + settled_at in one query, only if still active
            let settled = queries::settle_escrow_atomic(&state.db, &id)
                .await
                .map_err(|_e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new(
                            "internal_error",
                            "An internal error occurred. Please try again later."
                        ))),
                    )
                })?;

            if !settled {
                return Err((
                    StatusCode::CONFLICT,
                    Json(json!(ApiError::new(
                        "escrow_already_finalized",
                        "Escrow was already settled or is no longer active"
                    ))),
                ));
            }

            Ok(Json(json!({ "status": "settled", "escrow_id": id })))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "escrow_not_found",
                format!("No escrow found with id '{id}'")
            ))),
        )),
    }
}

/// POST /v1/escrows/{id}/refund
///
/// Requires authentication headers:
/// - X-Daglock-Address: Buyer's Kaspa address
/// - X-Daglock-Signature: Hex-encoded signature
/// - X-Daglock-Message: Signed message (format: "refund:{escrow_id}")
pub async fn refund(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_escrow(&state.db, &id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "An internal error occurred. Please try again later."
            ))),
        )
    })?;

    match escrow {
        Some(current)
            if matches!(
                current.status,
                EscrowStatus::Settled
                    | EscrowStatus::Refunded
                    | EscrowStatus::Cancelled
                    | EscrowStatus::Expired
                    | EscrowStatus::PendingConfirmation
            ) =>
        {
            Err((
                StatusCode::CONFLICT,
                Json(json!(ApiError::new(
                    "escrow_already_finalized",
                    "Escrow is already finalized"
                ))),
            ))
        }
        Some(current) => {
            // Verify caller is authorized (buyer only with valid signature)
            let auth = AuthContext::from_headers(&headers).map_err(|e| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!(ApiError::new("unauthorized", e.to_string()))),
                )
            })?;
            // TODO: Replace MockSignatureVerifier with real verifier when key management is implemented
            let sig_verifier = crate::auth::Secp256k1Verifier::new();
            verify_refund_authorization(&current, &auth, &sig_verifier).map_err(|e| {
                (
                    StatusCode::FORBIDDEN,
                    Json(json!(ApiError::new("forbidden", e.to_string()))),
                )
            })?;

            // Verify escrow can be refunded (UTXO exists on-chain)
            verify_escrow_refundable(&current, state.verifier.as_ref()).map_err(|e| {
                (
                    StatusCode::CONFLICT,
                    Json(json!(ApiError::new("verification_failed", e.to_string()))),
                )
            })?;

            // Atomic update: status + refunded_at in one query, only if still active
            let refunded = queries::refund_escrow_atomic(&state.db, &id)
                .await
                .map_err(|_e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new(
                            "internal_error",
                            "An internal error occurred. Please try again later."
                        ))),
                    )
                })?;

            if !refunded {
                return Err((
                    StatusCode::CONFLICT,
                    Json(json!(ApiError::new(
                        "escrow_already_finalized",
                        "Escrow was already refunded or is no longer active"
                    ))),
                ));
            }

            Ok(Json(json!({ "status": "refunded", "escrow_id": id })))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "escrow_not_found",
                format!("No escrow found with id '{id}'")
            ))),
        )),
    }
}

/// POST /v1/escrows/{id}/dispute
pub async fn dispute(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<DisputeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_escrow(&state.db, &id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "An internal error occurred. Please try again later."
            ))),
        )
    })?;

    match escrow {
        Some(current)
            if matches!(
                current.status,
                EscrowStatus::Settled
                    | EscrowStatus::Refunded
                    | EscrowStatus::Cancelled
                    | EscrowStatus::Expired
                    | EscrowStatus::Disputed
            ) =>
        {
            Err((
                StatusCode::CONFLICT,
                Json(json!(ApiError::new(
                    "escrow_already_finalized",
                    "Escrow cannot be disputed"
                ))),
            ))
        }
        Some(_) => {
            queries::mark_escrow_disputed(&state.db, &id, body.reason.as_str())
                .await
                .map_err(|_e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new(
                            "internal_error",
                            "An internal error occurred. Please try again later."
                        ))),
                    )
                })?;
            Ok(Json(json!({ "status": "disputed", "escrow_id": id })))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "escrow_not_found",
                format!("No escrow found with id '{id}'")
            ))),
        )),
    }
}

/// POST /v1/escrows/{id}/cancel
pub async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_escrow(&state.db, &id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "An internal error occurred. Please try again later."
            ))),
        )
    })?;

    match escrow {
        Some(current)
            if matches!(
                current.status,
                EscrowStatus::Settled
                    | EscrowStatus::Refunded
                    | EscrowStatus::Cancelled
                    | EscrowStatus::Expired
            ) =>
        {
            Err((
                StatusCode::CONFLICT,
                Json(json!(ApiError::new(
                    "escrow_already_finalized",
                    "Escrow cannot be cancelled"
                ))),
            ))
        }
        Some(_) => {
            queries::mark_escrow_cancelled(&state.db, &id)
                .await
                .map_err(|_e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new(
                            "internal_error",
                            "An internal error occurred. Please try again later."
                        ))),
                    )
                })?;
            Ok(Json(json!({ "status": "cancelled", "escrow_id": id })))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "escrow_not_found",
                format!("No escrow found with id '{id}'")
            ))),
        )),
    }
}
