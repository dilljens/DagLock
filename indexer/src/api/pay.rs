//! Payment session API — merchant checkout flow for Escrow-as-a-Service.
//!
//! Merchants create sessions via `POST /v1/pay` (API key auth).
//! Buyers interact with sessions via `GET /v1/pay/:session_id` (no auth).
//! Sessions expire after 24 hours.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};

use crate::api::AppState;
use crate::db::queries;
use crate::types::*;

/// Verify X-Daglock-Api-Key header and return app_id on success.
async fn verify_api_key(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<String, (StatusCode, Json<Value>)> {
    let api_key = headers
        .get("x-daglock-api-key")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!(ApiError::new(
                    "unauthorized",
                    "X-Daglock-Api-Key header required."
                ))),
            )
        })?;

    let key_hash = blake2b_simd::Params::new()
        .hash_length(32)
        .hash(api_key.as_bytes())
        .as_bytes()
        .to_vec();

    let key_info: Option<(String, String, bool)> = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT a.id, k.tier, k.webhooks_enabled FROM apps a
         INNER JOIN api_keys k ON k.app_id = a.id
         WHERE k.key_hash = ?1 AND k.is_active = 1 AND a.is_active = 1",
    )
    .bind(&key_hash)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "Failed to verify API key"
            ))),
        )
    })?
    .map(|(id, tier, wh)| (id, tier, wh != 0));

    match key_info {
        Some((id, _tier, _webhooks_enabled)) => {
            let _ = sqlx::query("UPDATE api_keys SET last_used_at = ?1 WHERE key_hash = ?2")
                .bind(chrono::Utc::now().timestamp())
                .bind(&key_hash)
                .execute(&state.db)
                .await;
            Ok(id)
        }
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new(
                "unauthorized",
                "Invalid or revoked API key"
            ))),
        )),
    }
}

/// Validate a Kaspa address format (matches existing pattern).
fn validate_kaspa_address(addr: &str) -> bool {
    addr.starts_with("kaspa:") && addr.len() >= 40 && addr.len() <= 80
}

/// POST /v1/pay — create a checkout session.
///
/// Body: { amount, asset?, seller_address, memo?, redirect_url?, webhook_url? }
/// Auth: X-Daglock-Api-Key header
pub async fn create_session(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let app_id = verify_api_key(&state, &headers).await?;

    let amount_sompi = body
        .get("amount")
        .and_then(|v| v.as_i64())
        .filter(|v| *v > 0)
        .ok_or_else(|| {
            bad_request(
                "invalid_amount",
                "Amount must be a positive integer (sompi)",
            )
        })?;

    let seller_address = body
        .get("seller_address")
        .and_then(|v| v.as_str())
        .filter(|s| validate_kaspa_address(s))
        .ok_or_else(|| bad_request("invalid_address", "Valid seller Kaspa address required"))?;

    let asset_type = body
        .get("asset")
        .and_then(|v| v.as_str())
        .unwrap_or("KAS")
        .to_uppercase();

    let memo = body.get("memo").and_then(|v| v.as_str()).map(String::from);
    let webhook_url = body
        .get("webhook_url")
        .and_then(|v| v.as_str())
        .map(String::from);
    let redirect_url = body
        .get("redirect_url")
        .and_then(|v| v.as_str())
        .map(String::from);

    let now = chrono::Utc::now().timestamp();
    let session_id = generate_id("pay");
    let expires_at = now + 86400; // 24 hours

    let session = PaymentSession {
        id: session_id.clone(),
        app_id,
        escrow_id: None,
        amount_sompi,
        asset_type,
        seller_address: seller_address.to_string(),
        memo,
        status: "pending".to_string(),
        buyer_address: None,
        created_at: now,
        expires_at,
        webhook_url,
        redirect_url,
    };

    queries::insert_session(&state.db, &session)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new(
                    "internal_error",
                    format!("Failed to create session: {e}")
                ))),
            )
        })?;

    let checkout_url = format!("https://daglock.com/pay/{}", session_id);

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "session_id": session_id,
            "status": "pending",
            "checkout_url": checkout_url,
            "expires_at": expires_at,
        })),
    ))
}

/// GET /v1/pay/:session_id — get session status (public, no auth required).
pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let session = queries::get_session(&state.db, &session_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new("internal_error", format!("{e}")))),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!(ApiError::new(
                    "session_not_found",
                    format!("Payment session '{session_id}' not found")
                ))),
            )
        })?;

    Ok(Json(json!({
        "id": session.id,
        "status": session.status,
        "amount_sompi": session.amount_sompi,
        "asset_type": session.asset_type,
        "seller_address": session.seller_address,
        "memo": session.memo,
        "buyer_address": session.buyer_address,
        "escrow_id": session.escrow_id,
        "created_at": session.created_at,
        "expires_at": session.expires_at,
    })))
}

/// POST /v1/pay/:session_id/fund — link an escrow to a session.
///
/// Called by the web component after the buyer creates the escrow.
/// Body: { escrow_id, buyer_address }
/// No auth required (escrow_id serves as proof of creation).
pub async fn fund_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow_id = body
        .get("escrow_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| bad_request("missing_escrow_id", "escrow_id is required"))?;

    let buyer_address = body
        .get("buyer_address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| bad_request("missing_buyer_address", "buyer_address is required"))?;

    // Verify session exists and is still pending
    let session = queries::get_session(&state.db, &session_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new("internal_error", format!("{e}")))),
            )
        })?
        .ok_or_else(|| not_found("payment_session", &session_id))?;

    if session.status != "pending" {
        return Err(bad_request(
            "session_not_pending",
            &format!(
                "Session is in '{}' state, expected 'pending'",
                session.status
            ),
        ));
    }

    // Verify the escrow exists and matches
    let escrow = queries::get_escrow(&state.db, escrow_id)
        .await
        .map_err(|_| internal_error())?
        .ok_or_else(|| not_found("escrow", escrow_id))?;

    if escrow.amount_sompi != session.amount_sompi {
        return Err(bad_request(
            "amount_mismatch",
            "Escrow amount does not match session amount",
        ));
    }

    queries::update_session_escrow(&state.db, &session_id, escrow_id, buyer_address)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new(
                    "internal_error",
                    format!("Failed to update session: {e}")
                ))),
            )
        })?;

    Ok(Json(json!({
        "status": "funded",
        "session_id": session_id,
        "escrow_id": escrow_id,
    })))
}
