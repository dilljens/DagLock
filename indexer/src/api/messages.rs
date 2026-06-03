//! Escrow-threaded messaging API — encrypted at rest with AES-256-GCM.

use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::crypto;
use crate::db::queries;
use crate::types::*;

/// POST /v1/escrows/:id/messages — send a message
pub async fn send(
    State(state): State<AppState>,
    Path(escrow_id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<SendMessageRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate content length
    if body.content.is_empty() || body.content.len() > 1024 {
        return Err((StatusCode::BAD_REQUEST, Json(json!(ApiError::new("invalid_content", "Content must be 1-1024 characters")))));
    }

    // Auth
    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (StatusCode::UNAUTHORIZED, Json(json!(ApiError::new("unauthorized", "X-Daglock-* headers required"))))
    })?;

    // Verify escrow exists and sender is a party
    let escrow = queries::get_escrow(&state.db, &escrow_id).await.map_err(|_e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(ApiError::new("internal_error", "An internal error occurred."))))
    })?;
    let escrow = escrow.ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!(ApiError::new("escrow_not_found", format!("No escrow found with id '{escrow_id}'"))))))?;

    if auth.address != escrow.buyer_address && escrow.seller_address.as_deref() != Some(&auth.address) {
        return Err((StatusCode::FORBIDDEN, Json(json!(ApiError::new("forbidden", "Only escrow parties can send messages")))));
    }

    // Encrypt
    let (content_enc, nonce) = crypto::encrypt_message(&body.content);

    let now = chrono::Utc::now().timestamp();
    let msg = EscrowMessage {
        id: format!("msg_{}", Uuid::new_v4().to_string().split('-').next().unwrap()),
        escrow_id: escrow_id.clone(),
        sender_address: auth.address.clone(),
        content: body.content,
        created_at: now,
    };

    queries::insert_message(&state.db, &msg, &content_enc, &nonce).await.map_err(|_e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(ApiError::new("internal_error", "An internal error occurred."))))
    })?;

    Ok(Json(json!({"status":"sent","message":msg})))
}

/// GET /v1/escrows/:id/messages — read message thread
pub async fn list(
    State(state): State<AppState>,
    Path(escrow_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Verify escrow exists
    let escrow = queries::get_escrow(&state.db, &escrow_id).await.map_err(|_e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(ApiError::new("internal_error", "An internal error occurred."))))
    })?;
    let escrow = escrow.ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!(ApiError::new("escrow_not_found", format!("No escrow found with id '{escrow_id}'"))))))?;

    // Auth: parties can always read. Jurors can read during a dispute.
    let allow = match AuthContext::from_headers(&headers) {
        Ok(auth) => {
            // Check if party
            if auth.address == escrow.buyer_address || escrow.seller_address.as_deref() == Some(&auth.address) {
                true
            } else if escrow.status == crate::types::EscrowStatus::Disputed {
                // Check if juror on this escrow
                let jury_case = queries::get_jury_case_by_escrow(&state.db, &escrow_id).await.unwrap_or(None);
                match jury_case {
                    Some(c) => c.jurors.contains(&auth.address),
                    None => false,
                }
            } else {
                false
            }
        }
        Err(_) => false,
    };

    if !allow {
        return Err((StatusCode::FORBIDDEN, Json(json!(ApiError::new("forbidden", "Only escrow parties or assigned jurors can read messages")))));
    }

    // Fetch and decrypt
    let raw = queries::list_messages_raw(&state.db, &escrow_id).await.map_err(|_e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(ApiError::new("internal_error", "An internal error occurred."))))
    })?;

    let mut messages = Vec::new();
    for (sender, content_enc, nonce, created_at) in &raw {
        let decrypted = crypto::decrypt_message(content_enc, nonce)
            .unwrap_or_else(|| "[encrypted message — key unavailable]".to_string());
        messages.push(EscrowMessage {
            id: String::new(),
            escrow_id: escrow_id.clone(),
            sender_address: sender.clone(),
            content: decrypted,
            created_at: *created_at,
        });
    }

    Ok(Json(json!({
        "messages": messages,
        "total": messages.len() as i64,
        "escrow_id": escrow_id,
    })))
}
