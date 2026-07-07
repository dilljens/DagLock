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
    if body.content_enc.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!(ApiError::new("invalid_content", "content_enc must be non-empty hex")))));
    }
    if hex::decode(&body.content_enc).is_err() {
        return Err((StatusCode::BAD_REQUEST, Json(json!(ApiError::new("invalid_content", "content_enc must be valid hex")))));
    }

    if body.nonce.len() != 24 || hex::decode(&body.nonce).is_err() {
        return Err((StatusCode::BAD_REQUEST, Json(json!(ApiError::new("invalid_nonce", "nonce must be 24 hex chars (12 bytes)")))));
    }

    if body.chat_sig.len() != 128 || hex::decode(&body.chat_sig).is_err() {
        return Err((StatusCode::BAD_REQUEST, Json(json!(ApiError::new("invalid_chat_sig", "chat_sig must be 128 hex chars (64 bytes)")))));
    }

    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (StatusCode::UNAUTHORIZED, Json(json!(ApiError::new("unauthorized", "X-Daglock-* headers required"))))
    })?;

    let expected_message = format!("message:{}", escrow_id);
    if !state.sig_verifier.verify_signature(&auth.address, &auth.signature, &expected_message)
        .map_err(|e| (StatusCode::FORBIDDEN, Json(json!(ApiError::new("forbidden", format!("Signature verification failed: {e}"))))))?
    {
        return Err((StatusCode::FORBIDDEN, Json(json!(ApiError::new("forbidden", "Invalid signature")))));
    }

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

    let seq = queries::count_messages(&state.db, &escrow_id)
        .await
        .unwrap_or(0)
        + 1;

    let mut hasher = Sha256::new();
    hasher.update(body.content_enc.as_bytes());
    hasher.update(body.nonce.as_bytes());
    hasher.update(escrow_id.as_bytes());
    hasher.update(seq.to_string().as_bytes());
    let _signed_hash = hasher.finalize();

    if !state.mock_chat_sig {
        tracing::warn!("chat_sig verification not yet implemented — accepting with --mock-chat-sig={}", state.mock_chat_sig);
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
    state.anchor_service.enqueue_message(&escrow_id, &msg_id, &body.content_enc);

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
                    Json(json!(ApiError::new(
                        "forbidden",
                        "Invalid signature"
                    ))),
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
