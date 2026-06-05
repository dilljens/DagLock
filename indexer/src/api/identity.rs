//! Identity verification API handler.
//!
//! Allows users to optionally link social handles (Telegram, etc.) to
//! their Kaspa address by signing a verification message with their wallet.

use axum::http::StatusCode;
use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;
use crate::types::*;

/// POST /v1/identity
///
/// Link a social handle to a wallet address. The user must prove ownership
/// of the wallet by including auth headers with a signed message.
///
/// Signed message format: "daglock.io:verify:{platform}:{handle}"
/// This prevents replay attacks.
///
/// Request body:
/// {
///   "platform": "telegram",
///   "handle": "@dillon",
///   "signed_message": "daglock.io:verify:telegram:@dillon",
///   "signature_hex": "hex_signature_from_wallet"
/// }
pub async fn create_identity(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateIdentityRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate platform
    if body.platform != "telegram" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_platform",
                "Only 'telegram' is supported at this time"
            ))),
        ));
    }

    // Validate handle
    if body.handle.is_empty() || !body.handle.starts_with('@') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_handle",
                "Handle must be non-empty and start with @"
            ))),
        ));
    }

    // Extract authenticated address from headers
    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new(
                "unauthorized",
                "X-Daglock-* headers required"
            ))),
        )
    })?;

    // Verify the signed message matches the expected format
    let expected_message = format!("daglock.io:verify:{}:{}", body.platform, body.handle);
    if body.signed_message != expected_message {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_message",
                format!("Signed message must be '{expected_message}'")
            ))),
        ));
    }

    // Cryptographically verify the signature — the wallet signed
    // "daglock.io:verify:telegram:@handle" proving ownership of this address.
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &expected_message)
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

    queries::upsert_identity(
        &state.db,
        &auth.address,
        &body.platform,
        &body.handle,
        &body.signed_message,
        &body.signature_hex,
    )
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
        "status": "verified",
        "address": auth.address,
        "platform": body.platform,
        "handle": body.handle,
    })))
}
