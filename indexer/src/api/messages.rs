use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    Json,
};
use ed25519_dalek::VerifyingKey;
use serde_json::{json, Value};
use sha2::{Digest, Sha256, Sha512};
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
    if body.content_enc.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_content",
                "content_enc must be non-empty hex"
            ))),
        ));
    }
    if body.content_enc.len() > 8192 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "content_too_large",
                "Encrypted message must be 8192 hex chars or less (4KB plaintext)"
            ))),
        ));
    }
    if hex::decode(&body.content_enc).is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_content",
                "content_enc must be valid hex"
            ))),
        ));
    }

    if body.nonce.len() != 24 || hex::decode(&body.nonce).is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_nonce",
                "nonce must be 24 hex chars (12 bytes)"
            ))),
        ));
    }

    // chat_sig is base64-encoded Ed25519 signature (88 chars for 64 bytes)
    let chat_sig_bytes: [u8; 64] = if body.chat_sig.len() == 88 {
        use base64::Engine as _;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&body.chat_sig)
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!(ApiError::new(
                        "invalid_chat_sig",
                        "chat_sig must be valid base64"
                    ))),
                )
            })?;
        <[u8; 64]>::try_from(decoded.as_slice()).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!(ApiError::new(
                    "invalid_chat_sig",
                    "chat_sig must decode to exactly 64 bytes"
                ))),
            )
        })?
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_chat_sig",
                "chat_sig must be 88 base64 chars (64 bytes)"
            ))),
        ));
    };

    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new(
                "unauthorized",
                "X-Daglock-* headers required"
            ))),
        )
    })?;

    let expected_message = format!("message:{}", escrow_id);
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &expected_message)
        .map_err(|e| {
            (
                StatusCode::FORBIDDEN,
                Json(json!(ApiError::new(
                    "forbidden",
                    format!("Signature verification failed: {e}")
                ))),
            )
        })?
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new("forbidden", "Invalid signature"))),
        ));
    }

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

    if auth.address != escrow.buyer_address
        && escrow.seller_address.as_deref() != Some(&auth.address)
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "forbidden",
                "Only escrow parties can send messages"
            ))),
        ));
    }

    let seq = queries::count_messages(&state.db, &escrow_id)
        .await
        .unwrap_or(0)
        + 1;

    // Verify Ed25519 chat signature: the client signs SHA-512(contentEnc:nonce:escrowId:seq)
    // using their chat private key. We verify against their registered chat pubkey.
    if !state.mock_chat_sig {
        // Get the sender's chat pubkey from the escrow record
        let sender_pubkey_b64 = if auth.address == escrow.buyer_address {
            escrow.chat_pubkey_buyer.clone()
        } else {
            escrow.chat_pubkey_seller.clone()
        };

        match sender_pubkey_b64 {
            Some(pubkey_b64) => {
                let pubkey_bytes = {
                    use base64::Engine as _;
                    base64::engine::general_purpose::STANDARD
                        .decode(&pubkey_b64)
                        .map_err(|_| {
                            (
                                StatusCode::FORBIDDEN,
                                Json(json!(ApiError::new(
                                    "invalid_chat_key",
                                    "Sender's chat pubkey is not valid base64"
                                ))),
                            )
                        })?
                };
                let pubkey = VerifyingKey::from_bytes(&pubkey_bytes.try_into().map_err(|_| {
                    (
                        StatusCode::FORBIDDEN,
                        Json(json!(ApiError::new(
                            "invalid_chat_key",
                            "Sender's chat pubkey is not 32 bytes"
                        ))),
                    )
                })?)
                .map_err(|_| {
                    (
                        StatusCode::FORBIDDEN,
                        Json(json!(ApiError::new(
                            "invalid_chat_key",
                            "Sender's chat pubkey is not a valid Ed25519 key"
                        ))),
                    )
                })?;

                // Hash the signed message with SHA-512 (matching nacl.hash in the client)
                let mut hasher = Sha512::new();
                hasher.update(body.content_enc.as_bytes());
                hasher.update(b":");
                hasher.update(body.nonce.as_bytes());
                hasher.update(b":");
                hasher.update(escrow_id.as_bytes());
                hasher.update(b":");
                hasher.update(seq.to_string().as_bytes());

                // Verify the Ed25519 signature via verify_prehashed (takes the digest state)
                let sig = ed25519_dalek::Signature::from_bytes(&chat_sig_bytes);
                pubkey.verify_prehashed(hasher, None, &sig).map_err(|_| {
                    (
                        StatusCode::FORBIDDEN,
                        Json(json!(ApiError::new(
                            "invalid_chat_sig",
                            "Chat signature does not match the sender's registered pubkey"
                        ))),
                    )
                })?;
            }
            None => {
                return Err((StatusCode::FORBIDDEN, Json(json!(ApiError::new("no_chat_key", "Sender has not registered a chat pubkey — submit one via POST /v1/escrows/:id/chat-pubkey first")))));
            }
        }
    }

    let now = chrono::Utc::now().timestamp();
    let msg_id = format!(
        "msg_{}",
        Uuid::new_v4()
            .to_string()
            .split('-')
            .next()
            .unwrap_or_default()
    );
    #[allow(deprecated)]
    let msg = EscrowMessage {
        id: msg_id.clone(),
        escrow_id: escrow_id.clone(),
        sender_address: auth.address.clone(),
        content: String::new(),
        created_at: now,
    };

    queries::insert_message(&state.db, &msg, &body.content_enc, &body.nonce)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    // Enqueue for on-chain anchoring
    state
        .anchor_service
        .enqueue_message(&escrow_id, &msg_id, &body.content_enc);

    Ok(Json(json!({"status": "sent", "message_id": msg.id})))
}

/// GET /v1/escrows/:id/messages — read encrypted message thread with anchor info
pub async fn list(
    State(state): State<AppState>,
    Path(escrow_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
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
            } else if escrow.status == EscrowStatus::Disputed {
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

    let anchored = queries::list_messages_with_anchors(&state.db, &escrow_id)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    let messages: Vec<Value> = anchored
        .iter()
        .map(|m| {
            json!({
                "id": m.id,
                "sender_address": m.sender_address,
                "content_enc": m.content_enc,
                "nonce": m.nonce,
                "created_at": m.created_at,
                "anchor_tx_id": m.anchor_tx_id,
                "anchor_daa_score": m.anchor_daa_score,
                "anchor_batch_hash": m.anchor_batch_hash,
            })
        })
        .collect();

    Ok(Json(json!({
        "messages": messages,
        "total": messages.len() as i64,
        "escrow_id": escrow_id,
    })))
}

/// GET /v1/escrows/:id/messages/anchors — anchor summary for the escrow
pub async fn anchors(
    State(state): State<AppState>,
    Path(escrow_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
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

    let auth_res = AuthContext::from_headers(&headers);
    let allow = match auth_res {
        Ok(auth_ref) => {
            if !state
                .sig_verifier
                .verify_signature(
                    &auth_ref.address,
                    &auth_ref.signature,
                    &format!("anchor:{}", escrow_id),
                )
                .unwrap_or(false)
            {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!(ApiError::new("forbidden", "Invalid signature"))),
                ));
            }
            auth_ref.address == escrow.buyer_address
                || escrow.seller_address.as_deref() == Some(&auth_ref.address)
        }
        Err(_) => false,
    };

    if !allow {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "forbidden",
                "Only escrow parties can view anchor summary"
            ))),
        ));
    }

    let batches = queries::get_anchor_summary(&state.db, &escrow_id)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    let batch_count = batches.len() as i64;

    Ok(Json(json!({
        "escrow_id": escrow_id,
        "batch_count": batch_count,
        "batches": batches,
    })))
}
