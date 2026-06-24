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

use crate::auth::{parse_message, verify_nonce, AuthContext};
use crate::db::queries;
use crate::services::webhooks::{self, WebhookEvent};
use crate::types::*;

use crate::websocket::WsEvent;
// Preimage verification uses SHA-256 via the covenant's trade_hash

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
    pub mode: Option<String>, // "standard" or "jury" — defaults to standard
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

/// GET /v1/escrows/:id/lock-status
/// Check if the escrow's UTXO is confirmed on-chain.
/// Returns { confirmed: bool, status: escrow_status }.
/// Uses the configured verifier (WrpcVerifier or MockVerifier).
pub async fn lock_status(
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
        Some(e) => {
            let confirmed = state.verifier.verify_utxo_exists(&e).await.unwrap_or(false);
            Ok(Json(json!({
                "confirmed": confirmed,
                "status": e.status,
                "escrow_id": id,
            })))
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

/// POST /v1/escrows/{id}/swap
/// Atomic swap: submit a preimage to settle the escrow.
pub async fn atomic_swap(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AtomicSwapRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let svc = crate::services::escrow_service::EscrowService::new(
        state.db.clone(),
        &state.ws_tx,
        state.sig_verifier.clone(),
        state.verifier.clone(),
    );
    svc.atomic_swap(&id, &body.preimage)
        .await
        .map(|_| Json(json!({"status": "settled", "escrow_id": id, "method": "atomic_swap"})))
        .map_err(service_error)
}

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
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateEscrowRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate request body
    validate_create_request(&headers, &body, &state).await?;

    let fee_sompi = body.amount_sompi / daglock_shared::FEE_DENOMINATOR; // 0.5%
    let buyer_address = &body.buyer_address;

    // Rate limit: max 50 escrows per address per day
    let recent_count = queries::count_escrows_by_buyer_recent(&state.db, buyer_address, 86400)
        .await
        .map_err(|_e| crate::types::internal_error())?;
    if recent_count >= 50 {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!(ApiError::new(
                "rate_limited",
                "Max 50 escrows per day per address"
            ))),
        ));
    }

    let escrow = Escrow {
        id: format!(
            "esc_{}",
            Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .expect("UUID should have a dash")
        ),
        lock_tx_id: body.lock_tx_id,
        lock_tx_output_index: body.lock_tx_output_index,
        status: EscrowStatus::PendingConfirmation,
        asset_type: body.asset_type.unwrap_or_else(|| "KAS".to_string()),
        buyer_address: buyer_address.clone(),
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
        mediator_key: body.mediator_key.clone(),
        dispute_mode: body.dispute_mode.or_else(|| {
            if body.mediator_key.is_some() {
                Some("mediator".to_string())
            } else {
                None
            }
        }),
        dispute_outcome: None,
        dispute_resolved_at: None,
        price_at_creation: if body.price_type.as_deref() == Some("market") {
            crate::types::fetch_kas_usd_price().await
        } else {
            body.price_at_creation
        },
        price_currency: body.price_currency.or_else(|| {
            if body.price_type.as_deref() == Some("market") {
                Some("USD".to_string())
            } else {
                None
            }
        }),
        trade_hash: body.trade_hash.clone(),
        price_lock_time: if body.price_at_creation.is_some()
            || body.price_type.as_deref() == Some("market")
        {
            Some(chrono::Utc::now().timestamp())
        } else {
            None
        },
        price_at_settlement: if body.price_type.as_deref() == Some("market") {
            None // Will be set at settlement time
        } else {
            body.price_at_creation
        },
        price_source: if body.price_type.as_deref() == Some("market") {
            Some("coingecko".to_string())
        } else {
            None
        },
        price_type: body.price_type.clone(),
    };

    queries::insert_escrow(&state.db, &escrow)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    let _ = state.ws_tx.send(WsEvent::escrow_created(&escrow.id));
    webhooks::dispatch(state.db.clone(), WebhookEvent::EscrowCreated(&escrow.id));

    Ok((StatusCode::CREATED, Json(json!(escrow))))
}

/// Validate an escrow creation request.
/// Returns an error tuple on any validation failure.
async fn validate_create_request(
    headers: &axum::http::HeaderMap,
    body: &CreateEscrowRequest,
    state: &AppState,
) -> Result<(), (StatusCode, Json<Value>)> {
    if body.amount_sompi <= 0 {
        return Err(bad_request("invalid_amount", "amount must be positive"));
    }

    // Optional auth: verify buyer address if auth headers present
    if headers.contains_key("x-daglock-address") {
        let auth = AuthContext::from_headers(headers).map_err(|e| {
            (StatusCode::UNAUTHORIZED, Json(json!(ApiError::new("unauthorized", e.to_string()))))
        })?;
        if auth.address != body.buyer_address {
            return Err(forbidden("forbidden", "Signed address must match buyer_address"));
        }
    }

    if !validate_kaspa_address(&body.buyer_address) {
        return Err(bad_request("invalid_address", "Invalid buyer Kaspa address"));
    }
    if body.buyer_address.len() > 100 {
        return Err(bad_request("invalid_address", "Buyer address too long"));
    }

    if let Some(ref seller) = body.seller_address {
        if seller == &body.buyer_address {
            return Err(bad_request("self_referential", "Buyer and seller cannot be the same address"));
        }
        if !validate_kaspa_address(seller) {
            return Err(bad_request("invalid_address", "Invalid seller Kaspa address"));
        }
        if seller.len() > 100 {
            return Err(bad_request("invalid_address", "Seller address too long"));
        }
    }

    if let Some(ref med) = body.mediator_key {
        if !med.is_empty() && !validate_kaspa_address(med) {
            return Err(bad_request("invalid_address", "Invalid mediator Kaspa address"));
        }
    }

    if let Some(ref template_hash) = body.template_hash {
        if !template_hash.is_empty() {
            let known_hashes = [
                state.daglock_kas_template.as_deref(),
                state.daglock_krc20_template.as_deref(),
            ];
            let is_known = known_hashes.iter().any(|h| match h {
                Some(expected) => {
                    let expected_bytes = hex::decode(expected).unwrap_or_default();
                    expected_bytes.as_slice() == template_hash.as_slice()
                }
                None => false,
            });
            if !is_known {
                return Err(bad_request("unknown_template",
                    "Template hash does not match any known DagLock covenant."));
            }
        }
    }

    if let Some(ref trade_hash) = body.trade_hash {
        if !trade_hash.is_empty() {
            daglock_shared::validate_trade_hash(trade_hash)
                .map_err(|e| bad_request("invalid_trade_hash", format!("Invalid trade hash: {e}")))?;
        }
    }

    if body.amount_sompi > 100_000_000_000_000 {
        return Err(bad_request("invalid_amount", "Amount exceeds maximum (1M KAS)"));
    }

    if let Ok((existing, _)) =
        queries::list_escrows_by_address(&state.db, &body.buyer_address, None, None, 100, 0).await
    {
        if existing.iter().any(|e| {
            e.lock_tx_id == body.lock_tx_id && e.lock_tx_output_index == body.lock_tx_output_index
        }) {
            return Err(conflict("duplicate_lock", "An escrow already exists for this UTXO"));
        }
    }

    Ok(())
}

// ── Error helpers for validation ────────────────────────────────────

fn bad_request(code: &'static str, msg: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    (StatusCode::BAD_REQUEST, Json(json!(ApiError::new(code, msg.to_string()))))
}

fn forbidden(code: &'static str, msg: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    (StatusCode::FORBIDDEN, Json(json!(ApiError::new(code, msg.to_string()))))
}

fn conflict(code: &'static str, msg: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    (StatusCode::CONFLICT, Json(json!(ApiError::new(code, msg.to_string()))))
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
    let svc = crate::services::escrow_service::EscrowService::new(
        state.db.clone(),
        &state.ws_tx,
        state.sig_verifier.clone(),
        state.verifier.clone(),
    );
    svc.settle(&id, &headers)
        .await
        .map(|_| Json(json!({ "status": "settled", "escrow_id": id })))
        .map_err(service_error)
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
    let svc = crate::services::escrow_service::EscrowService::new(
        state.db.clone(),
        &state.ws_tx,
        state.sig_verifier.clone(),
        state.verifier.clone(),
    );
    svc.refund(&id, &headers)
        .await
        .map(|_| Json(json!({ "status": "refunded", "escrow_id": id })))
        .map_err(service_error)
}

/// POST /v1/escrows/{id}/dispute
pub async fn dispute(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
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
        Some(current) => {
            // Verify caller is authorized (buyer or seller with valid signature)
            let auth = AuthContext::from_headers(&headers).map_err(|e| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!(ApiError::new("unauthorized", e.to_string()))),
                )
            })?;
            let is_buyer = auth.address == current.buyer_address;
            let is_seller = current.seller_address.as_deref() == Some(&auth.address);
            if !is_buyer && !is_seller {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!(ApiError::new(
                        "forbidden",
                        "Only escrow parties can dispute"
                    ))),
                ));
            }
            let parsed = parse_message(&auth.message).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!(ApiError::new("invalid_message", e.to_string()))),
                )
            })?;
            if parsed.action != "dispute" || parsed.escrow_id != id {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!(ApiError::new(
                        "invalid_message",
                        "Message must be 'dispute:{id}:ts:nonce'"
                    ))),
                ));
            }
            if !state
                .sig_verifier
                .verify_signature(&auth.address, &auth.signature, &auth.message)
                .unwrap_or(false)
            {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!(ApiError::new(
                        "forbidden",
                        "Invalid signature for dispute"
                    ))),
                ));
            }
            verify_nonce(&state.db, &parsed, &auth.address)
                .await
                .map_err(|e| {
                    (
                        StatusCode::FORBIDDEN,
                        Json(json!(ApiError::new("forbidden", e.to_string()))),
                    )
                })?;

            let is_jury = body.mode.as_deref() == Some("jury");
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

            if is_jury {
                // Create a jury case with juror selection
                let (juror_count, threshold) =
                    crate::api::jury::juror_count_and_threshold(current.amount_sompi);

                // Fetch eligible jurors
                let eligible = queries::list_eligible_jurors_simple(&state.db)
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

                if eligible.len() < juror_count as usize {
                    return Err((
                        StatusCode::CONFLICT,
                        Json(json!(ApiError::new(
                            "insufficient_jurors",
                            format!(
                                "Need {} jurors but only {} registered",
                                juror_count,
                                eligible.len()
                            )
                        ))),
                    ));
                }

                // Random selection: take top N*2 by reliability_score → randomly pick N
                let candidate_pool: Vec<_> = eligible
                    .iter()
                    .take((juror_count as usize).saturating_mul(2).min(eligible.len()))
                    .collect();
                let pool_size = candidate_pool.len();
                let needed = (juror_count as usize).min(pool_size);
                let mut indices: Vec<usize> = (0..pool_size).collect();
                for i in (pool_size - needed..pool_size).rev() {
                    let j = rand::random::<usize>() % (i + 1);
                    indices.swap(i, j);
                }
                let selected: Vec<String> = indices[pool_size - needed..]
                    .iter()
                    .map(|&i| candidate_pool[i].address.clone())
                    .collect();

                let case_id =
                    queries::create_jury_case(&state.db, &id, juror_count, threshold, &selected)
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
                    "status": "disputed",
                    "escrow_id": id,
                    "jury_case_id": case_id,
                    "juror_count": juror_count,
                    "threshold": threshold,
                    "mode": "jury"
                })))
            } else {
                let _ = state
                    .ws_tx
                    .send(WsEvent::escrow_disputed(&id, &body.reason));
                Ok(Json(json!({
                    "status": "disputed",
                    "escrow_id": id
                })))
            }
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
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let svc = crate::services::escrow_service::EscrowService::new(
        state.db.clone(),
        &state.ws_tx,
        state.sig_verifier.clone(),
        state.verifier.clone(),
    );
    svc.cancel(&id, &headers)
        .await
        .map(|_| Json(json!({ "status": "cancelled", "escrow_id": id })))
        .map_err(service_error)
}
#[derive(Deserialize)]
pub struct AtomicSwapRequest {
    pub preimage: String,
}

/// Convert a service error to an HTTP error response.
fn service_error(e: crate::services::escrow_service::ServiceError) -> (StatusCode, Json<Value>) {
    use crate::services::escrow_service::ServiceError;
    let (status, message) = match &e {
        ServiceError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
        ServiceError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        ServiceError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
        ServiceError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
        ServiceError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
        ServiceError::VerificationFailed(msg) => (StatusCode::CONFLICT, msg.clone()),
        ServiceError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
    };
    (status, Json(json!(ApiError::new(e.error_code(), message))))
}
