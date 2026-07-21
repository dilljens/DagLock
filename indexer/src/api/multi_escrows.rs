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
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(crate::types::ApiError::new(
                "unauthorized",
                e.to_string()
            ))),
        )
    })?;

    // Must be a party to the escrow
    if !escrow.parties.contains(&auth.address) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(crate::types::ApiError::new(
                "forbidden",
                "Only escrow parties can perform this action"
            ))),
        ));
    }

    let parsed = parse_message(&auth.message).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!(crate::types::ApiError::new(
                "invalid_message",
                e.to_string()
            ))),
        )
    })?;

    if parsed.action != action {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(crate::types::ApiError::new(
                "forbidden",
                format!("Message must be '{action}:{{id}}:ts:nonce'")
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
            Json(json!(crate::types::ApiError::new(
                "forbidden",
                "Invalid signature"
            ))),
        ));
    }

    verify_nonce(&state.db, &parsed, &auth.address)
        .await
        .map_err(|e| {
            (
                StatusCode::FORBIDDEN,
                Json(json!(crate::types::ApiError::new(
                    "forbidden",
                    e.to_string()
                ))),
            )
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
        return Err(bad_request("invalid_parties", "Must have 2-4 parties"));
    }
    if body.parties.len() != body.shares.len() {
        return Err(bad_request(
            "invalid_shares",
            "Parties and shares must match",
        ));
    }
    if body.total_amount <= 0 {
        return Err(bad_request(
            "invalid_amount",
            "Total amount must be positive",
        ));
    }
    let total_shares: i64 = body.shares.iter().sum();
    if total_shares != 10000 {
        return Err(bad_request(
            "invalid_shares",
            "Shares must sum to 10000 (100%)",
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
        .map_err(|_e| internal_error())?;

    Ok((StatusCode::CREATED, Json(json!(escrow))))
}

/// GET /v1/multi-escrows?address=...
pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let address = params.address.as_deref().unwrap_or("");
    if address.is_empty() {
        return Err(bad_request(
            "invalid_address",
            "address query parameter is required",
        ));
    }

    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let (escrows, total) = queries::list_multi_by_address(&state.db, address, limit, offset)
        .await
        .map_err(|_e| internal_error())?;

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
        .map_err(|_e| internal_error())?
        .ok_or_else(|| not_found("multi-party escrow", &id))?;

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
        .map_err(|_e| internal_error())?
        .ok_or_else(|| not_found("multi-party escrow", &id))?;

    if escrow.status != "active" {
        return Err(conflict(
            "invalid_status",
            format!("Escrow is '{}', not 'active'", escrow.status),
        ));
    }

    if !escrow.parties.contains(&body.address) {
        return Err(forbidden(
            "not_a_party",
            "Address is not a party to this escrow",
        ));
    }

    if escrow.signatures.contains(&body.address) {
        return Err(conflict("already_signed", "Address has already signed"));
    }

    queries::record_signature(&state.db, &id, &body.address)
        .await
        .map_err(|_e| internal_error())?;

    let updated = queries::get_multi_escrow(&state.db, &id)
        .await
        .map_err(|_e| internal_error())?
        .ok_or_else(|| not_found("multi-party escrow", &id))?;

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
        .map_err(|_e| internal_error())?
        .ok_or_else(|| not_found("multi-party escrow", &id))?;

    if escrow.status != "active" {
        return Err(conflict(
            "invalid_status",
            format!("Escrow is '{}', not 'active'", escrow.status),
        ));
    }

    // Verify caller is authorized
    verify_multi_escrow_auth(&state, &headers, &escrow, "refund").await?;

    queries::refund_multi_escrow(&state.db, &id)
        .await
        .map_err(|_e| internal_error())?;

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
        .map_err(|_e| internal_error())?
        .ok_or_else(|| not_found("multi-party escrow", &id))?;

    if escrow.status != "active" {
        return Err(conflict(
            "invalid_status",
            format!("Escrow is '{}', not 'active'", escrow.status),
        ));
    }

    // Verify caller is authorized
    verify_multi_escrow_auth(&state, &headers, &escrow, "swap").await?;

    let all_signed = escrow.signatures.len() == escrow.parties.len();
    if !all_signed {
        return Err(conflict(
            "not_all_signed",
            "Not all parties have signed yet",
        ));
    }

    queries::settle_multi_escrow(&state.db, &id)
        .await
        .map_err(|_e| internal_error())?;

    Ok(Json(json!({
        "status": "settled",
        "escrow_id": id,
        "method": "swap",
        "message": "Multi-party escrow settled via atomic swap."
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::MockVerifier;
    use axum::http::HeaderMap;

    fn test_multi_escrow() -> MultiEscrow {
        MultiEscrow {
            id: "multi_001".to_string(),
            lock_tx_id: "tx123".to_string(),
            parties: vec![
                "kaspatest:qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpk58a75"
                    .to_string(),
                "kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya"
                    .to_string(),
            ],
            shares: vec![5000, 5000],
            total_amount: 1_000_000_000,
            status: "active".to_string(),
            created_at: 1_700_000_000,
            settled_at: None,
            refunded_at: None,
            signatures: Vec::new(),
        }
    }

    async fn test_state() -> AppState {
        use crate::db::init_pool;
        let pool = init_pool("sqlite::memory:").await.unwrap();
        let (ws_tx, _) = tokio::sync::broadcast::channel(4096);
        let pool_clone = pool.clone();
        AppState {
            db: pool_clone,
            started_at: std::time::Instant::now(),
            network: "testnet-11".to_string(),
            wrpc_url: None,
            daglock_kas_template: None,
            daglock_krc20_template: None,
            daglock_vault_softlock_template: None,
            daglock_vault_multisig_template: None,
            verifier: std::sync::Arc::new(crate::verification::MockVerifier),
            sig_verifier: std::sync::Arc::new(MockVerifier::new()),
            ws_tx,
            treasury_pubkey: None,
            explorer_base_url: "https://kas.fyi".to_string(),
            email_service: None,
            ai_mediator_api_key: None,
            ai_mediator_model: None,
            mock_chat_sig: false,
            anchor_service: std::sync::Arc::new(crate::services::anchor::AnchorService::new(
                pool, None, None,
            )),
            rate_limiter: std::sync::Arc::new(crate::ratelimit::RateLimiter::new()),
            admin_token: None,
            background_health: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
        }
    }

    fn auth_headers(address: &str, action: &str, escrow_id: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("x-daglock-address", address.parse().unwrap());
        headers.insert("x-daglock-signature", "ff".repeat(32).parse().unwrap());
        headers.insert(
            "x-daglock-message",
            format!(
                "{action}:{escrow_id}:{}:{}",
                chrono::Utc::now().timestamp(),
                crate::auth::generate_nonce()
            )
            .parse()
            .unwrap(),
        );
        headers
    }

    #[tokio::test]
    async fn verify_multi_auth_party_allowed() {
        let state = test_state().await;
        let escrow = test_multi_escrow();
        let headers = auth_headers(&escrow.parties[0], "refund", &escrow.id);
        let result = verify_multi_escrow_auth(&state, &headers, &escrow, "refund").await;
        assert!(result.is_ok(), "Party should be authorized");
    }

    #[tokio::test]
    async fn verify_multi_auth_other_party_allowed() {
        let state = test_state().await;
        let escrow = test_multi_escrow();
        let headers = auth_headers(&escrow.parties[1], "refund", &escrow.id);
        let result = verify_multi_escrow_auth(&state, &headers, &escrow, "refund").await;
        assert!(result.is_ok(), "Other party should be authorized");
    }

    #[tokio::test]
    async fn verify_multi_auth_outsider_rejected() {
        let state = test_state().await;
        let escrow = test_multi_escrow();
        let headers = auth_headers("kaspa:outsider", "refund", &escrow.id);
        let result = verify_multi_escrow_auth(&state, &headers, &escrow, "refund").await;
        assert!(result.is_err(), "Outsider should be rejected");
    }

    #[tokio::test]
    async fn verify_multi_auth_wrong_action_rejected() {
        let state = test_state().await;
        let escrow = test_multi_escrow();
        let headers = auth_headers(&escrow.parties[0], "refund", &escrow.id);
        let result = verify_multi_escrow_auth(&state, &headers, &escrow, "swap").await;
        assert!(result.is_err(), "Wrong action mismatch should be rejected");
    }

    #[tokio::test]
    async fn verify_multi_auth_missing_headers_rejected() {
        let state = test_state().await;
        let escrow = test_multi_escrow();
        let headers = HeaderMap::new();
        let result = verify_multi_escrow_auth(&state, &headers, &escrow, "refund").await;
        assert!(result.is_err(), "Missing headers should be rejected");
    }
}
