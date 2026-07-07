//! Escrow-threaded messaging API — client-side encrypted.
//!
//! The server never sees plaintext content. All encryption happens on the
//! client using the counterparty's Ed25519 chat public key stored on the
//! escrow record. The server stores only ciphertext + nonce.
//!
//! # Chat signature verification (future)
//!
//! Each message carries an Ed25519 `chat_sig` over:
//!   `sha256(content_enc || nonce || escrow_id || seq)`
//! where `seq` is the 1-indexed message count for this escrow.
//! Ed25519 verification is not yet implemented in Rust (tracked as future
//! work). Use `--mock-chat-sig` in dev mode to skip verification.

use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;
use crate::types::*;

/// POST /v1/escrows/:id/messages — send an encrypted message
pub async fn send(
    State(state): State<AppState>,
    Path(escrow_id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<SendMessageRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // ── Validate ciphertext ──────────────────────────────────────
    if body.content_enc.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!(ApiError::new("invalid_content", "content_enc must be non-empty hex")))));
    }
    if hex::decode(&body.content_enc).is_err() {
        return Err((StatusCode::BAD_REQUEST, Json(json!(ApiError::new("invalid_content", "content_enc must be valid hex")))));
    }

    // ── Validate nonce (12 bytes = 24 hex chars) ─────────────────
    if body.nonce.len() != 24 || hex::decode(&body.nonce).is_err() {
        return Err((StatusCode::BAD_REQUEST, Json(json!(ApiError::new("invalid_nonce", "nonce must be 24 hex chars (12 bytes)")))));
    }

    // ── Validate chat_sig (64 bytes = 128 hex chars) ─────────────
    if body.chat_sig.len() != 128 || hex::decode(&body.chat_sig).is_err() {
        return Err((StatusCode::BAD_REQUEST, Json(json!(ApiError::new("invalid_chat_sig", "chat_sig must be 128 hex chars (64 bytes)")))));
    }

    // ── Auth ─────────────────────────────────────────────────────
    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (StatusCode::UNAUTHORIZED, Json(json!(ApiError::new("unauthorized", "X-Daglock-* headers required"))))
    })?;

    let expected_message = format!("message:{}", escrow_id);
    if !state.sig_verifier.verify_signature(&auth.address, &auth.signature, &expected_message)
        .map_err(|e| (StatusCode::FORBIDDEN, Json(json!(ApiError::new("forbidden", format!("Signature verification failed: {e}"))))))?
    {
        return Err((StatusCode::FORBIDDEN, Json(json!(ApiError::new("forbidden", "Invalid signature")))));
    }

    // ── Verify escrow exists and sender is a party ───────────────
    let escrow = queries::get_escrow(&state.db, &escrow_id)
        .await
        .map_err(|_e| crate::types::internal_error())?;
    let escrow = escrow.ok_or_else(|| {
        (StatusCode::NOT_FOUND, Json(json!(ApiError::new("escrow_not_found", format!("No escrow found with id '{escrow_id}'")))))
    })?;

    if auth.address != escrow.buyer_address
        && escrow.seller_address.as_deref() != Some(&auth.address)
    {
        return Err((StatusCode::FORBIDDEN, Json(json!(ApiError::new("forbidden", "Only escrow parties can send messages")))));
    }

    // ── Verify chat_sig (Ed25519) ────────────────────────────────
    // seq = 1-indexed message number in this escrow
    let seq = queries::count_messages(&state.db, &escrow_id)
        .await
        .unwrap_or(0)
        + 1;

    // Build signed message: sha256(content_enc || nonce || escrow_id || seq)
    let mut hasher = Sha256::new();
    hasher.update(body.content_enc.as_bytes());
    hasher.update(body.nonce.as_bytes());
    hasher.update(escrow_id.as_bytes());
    hasher.update(seq.to_string().as_bytes());
    let _signed_hash = hasher.finalize();

    if !state.mock_chat_sig {
        // TODO: Verify Ed25519 signature.
        //   1. Fetch sender's chat_pubkey via queries::get_chat_pubkey(&state.db, &escrow_id, &auth.address)
        //   2. Parse hex public key (32 bytes)
        //   3. Verify body.chat_sig (64 bytes) over _signed_hash using Ed25519
        //   4. Return FORBIDDEN if invalid
        //
        // Requires an Ed25519 verification library (e.g. ed25519-dalek).
        // Tracked as future work — use --mock-chat-sig to skip for now.
        tracing::warn!("chat_sig verification not yet implemented — accepting with --mock-chat-sig={}", state.mock_chat_sig);
    }

    // ── Store ────────────────────────────────────────────────────
    let now = chrono::Utc::now().timestamp();
    let msg = EscrowMessage {
        id: format!(
            "msg_{}",
            Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or_default()
        ),
        escrow_id: escrow_id.clone(),
        sender_address: auth.address.clone(),
        content: String::new(),
        created_at: now,
    };

    queries::insert_message(&state.db, &msg, &body.content_enc, &body.nonce)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    Ok(Json(json!({"status": "sent", "message_id": msg.id})))
}

/// GET /v1/escrows/:id/messages — read encrypted message thread
///
/// Returns raw ciphertext + nonce. The client is responsible for decryption
/// using the sender's chat public key stored on the escrow record.
pub async fn list(
    State(state): State<AppState>,
    Path(escrow_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Verify escrow exists
    let escrow = queries::get_escrow(&state.db, &escrow_id)
        .await
        .map_err(|_e| crate::types::internal_error())?;
    let escrow = escrow.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "escrow_not_found",
                format!("No escrow found with id '{escrow_id}'")
            ))),
        )
    })?;

    // Auth: parties can always read. Jurors can read during a dispute.
    let auth_res = AuthContext::from_headers(&headers);
    let allow = match auth_res {
        Ok(auth_ref) => {
            if !state
                .sig_verifier
                .verify_signature(
                    &auth_ref.address,
                    &auth_ref.signature,
                    &format!("messages:{}", escrow_id),
                )
                .unwrap_or(false)
            {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!(ApiError::new(
                        "forbidden",
                        "Invalid signature for messages"
                    ))),
                ));
            }
            if auth_ref.address == escrow.buyer_address
                || escrow.seller_address.as_deref() == Some(&auth_ref.address)
            {
                true
            } else if escrow.status == crate::types::EscrowStatus::Disputed {
                let jury_case = queries::get_jury_case_by_escrow(&state.db, &escrow_id)
                    .await
                    .unwrap_or(None);
                match jury_case {
                    Some(c) => c.jurors.contains(&auth_ref.address),
                    None => false,
                }
            } else {
                false
            }
        }
        Err(_) => false,
    };

    if !allow {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "forbidden",
                "Only escrow parties or assigned jurors can read messages"
            ))),
        ));
    }

    // Fetch raw ciphertext (no server-side decryption)
    let raw = queries::list_messages_raw(&state.db, &escrow_id)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    let messages: Vec<Value> = raw
        .iter()
        .map(|(sender, content_enc, nonce, created_at)| {
            json!({
                "sender_address": sender,
                "content_enc": content_enc,
                "nonce": nonce,
                "created_at": created_at,
            })
        })
        .collect();

    Ok(Json(json!({
        "messages": messages,
        "total": messages.len() as i64,
        "escrow_id": escrow_id,
    })))
}
