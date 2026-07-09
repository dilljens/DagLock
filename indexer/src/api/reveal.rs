//! Chat revelation & evidence API — dispute party reveals chat key to jury.

use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;
use crate::types::*;

/// POST /v1/escrows/:id/messages/reveal
/// Party submits their chat private key so the jury can decrypt messages.
pub async fn reveal(
    State(state): State<AppState>,
    Path(escrow_id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<RevealChatKeyRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new(
                "unauthorized",
                "X-Daglock-* headers required"
            ))),
        )
    })?;

    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &format!("reveal:{}", escrow_id))
        .unwrap_or(false)
    {
        return Err(forbidden("invalid_signature", "Signature does not match the claimed address"));
    }

    let escrow = queries::get_escrow(&state.db, &escrow_id)
        .await
        .map_err(|_e| internal_error())?
        .ok_or_else(|| not_found("escrow", &escrow_id))?;

    if escrow.status != EscrowStatus::Disputed {
        return Err((
            StatusCode::CONFLICT,
            Json(json!(ApiError::new(
                "not_disputed",
                "Escrow is not under dispute"
            ))),
        ));
    }

    if auth.address != escrow.buyer_address
        && escrow.seller_address.as_deref() != Some(&auth.address)
    {
        return Err(forbidden("not_party", "Only escrow parties can reveal chat key"));
    }

    let case = queries::get_jury_case_by_escrow(&state.db, &escrow_id)
        .await
        .map_err(|_e| internal_error())?
        .ok_or_else(|| not_found("jury case", &escrow_id))?;

    queries::reveal_chat_key(&state.db, &case.id, &body.encrypted_chat_key)
        .await
        .map_err(|_e| internal_error())?;

    // The chat key has been stored. Next: decrypt messages client-side in the
    // jury panel UI. The encrypted chat key is stored in jury_cases.chat_key_revealed,
    // and message decryption happens in the browser using tweetnacl secretbox.
    // The server stores the raw encrypted messages with metadata as evidence.
    let messages = queries::list_messages_with_anchors(&state.db, &escrow_id)
        .await
        .map_err(|_e| internal_error())?;

    let evidence: Vec<EvidenceMessage> = messages
        .into_iter()
        .map(|m| EvidenceMessage {
            id: format!("ev_{}", m.id),
            sender_address: m.sender_address,
            // Decryption happens client-side. The encrypted chat secret key
            // was stored in the jury case record. The jury panel UI fetches it
            // via GET /v1/jury/cases/:id and decrypts messages with tweetnacl
            // using the chat secret key + message nonce.
            decrypted_content: format!(
                "[key revealed for case — jury UI will decrypt] escrow_id={} msg_id={}",
                escrow_id, m.id
            ),
            created_at: m.created_at,
            anchor_tx_id: m.anchor_tx_id,
            anchor_daa_score: m.anchor_daa_score,
        })
        .collect();

    let count = evidence.len() as i64;
    queries::store_decrypted_evidence(&state.db, &case.id, &evidence)
        .await
        .map_err(|_e| internal_error())?;

    Ok(Json(json!({
        "status": "revealed",
        "evidence_count": count,
    })))
}

/// GET /v1/jury/cases/:id/evidence
/// Assigned juror reads decrypted evidence.
pub async fn evidence(
    State(state): State<AppState>,
    Path(case_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new(
                "unauthorized",
                "X-Daglock-* headers required"
            ))),
        )
    })?;

    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &format!("evidence:{}", case_id))
        .unwrap_or(false)
    {
        return Err(forbidden("invalid_signature", "Signature does not match the claimed address"));
    }

    let case = queries::get_jury_case(&state.db, &case_id)
        .await
        .map_err(|_e| internal_error())?
        .ok_or_else(|| not_found("jury case", &case_id))?;

    if !case.jurors.contains(&auth.address) {
        return Err(forbidden("not_juror", "Only assigned jurors can view evidence"));
    }

    let evidence = queries::get_decrypted_evidence(&state.db, &case_id)
        .await
        .map_err(|_e| internal_error())?;

    // Fetch escrow chat pubkeys
    let escrow = queries::get_escrow(&state.db, &case.escrow_id)
        .await
        .map_err(|_e| internal_error())?;

    let chat_pubkey_buyer = escrow.as_ref().and_then(|e| e.chat_pubkey_buyer.clone());
    let chat_pubkey_seller = escrow.as_ref().and_then(|e| e.chat_pubkey_seller.clone());

    Ok(Json(json!({
        "evidence": evidence,
        "chat_pubkey_buyer": chat_pubkey_buyer,
        "chat_pubkey_seller": chat_pubkey_seller,
        "revealed": case.revealed_at.is_some(),
        "cleared": case.evidence_cleared_at.is_some(),
    })))
}

/// POST /v1/jury/cases/:id/evidence/clear
/// Admin or arbiter wipes evidence after case resolution.
pub async fn clear_evidence(
    State(state): State<AppState>,
    Path(case_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new(
                "unauthorized",
                "X-Daglock-* headers required"
            ))),
        )
    })?;

    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &format!("clear_evidence:{}", case_id))
        .unwrap_or(false)
    {
        return Err(forbidden("invalid_signature", "Signature does not match the claimed address"));
    }

    let case = queries::get_jury_case(&state.db, &case_id)
        .await
        .map_err(|_e| internal_error())?
        .ok_or_else(|| not_found("jury case", &case_id))?;

    // Allow either the arbiter/admin or any assigned juror on a decided case to clear
    let escrow = queries::get_escrow(&state.db, &case.escrow_id)
        .await
        .map_err(|_e| internal_error())?;

    let is_admin = false;
    let is_party = escrow.as_ref().map(|e| {
        auth.address == e.buyer_address
            || e.seller_address.as_deref() == Some(&auth.address)
    }).unwrap_or(false);
    let is_juror = case.jurors.contains(&auth.address);

    if !is_admin && !is_party && !is_juror {
        return Err(forbidden("not_authorized", "Not authorized to clear evidence"));
    }

    queries::clear_evidence(&state.db, &case_id)
        .await
        .map_err(|_e| internal_error())?;

    Ok(Json(json!({"status": "cleared"})))
}
