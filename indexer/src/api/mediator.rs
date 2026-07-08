//! AI Mediation API handlers — non-binding dispute resolution before jury.

use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;
use crate::services::ai_mediator::AiMediator;
use crate::services::escrow_service::EscrowService;
use crate::types::*;

/// POST /v1/escrows/:id/mediate — submit claims and trigger AI mediation.
pub async fn mediate(
    State(state): State<AppState>,
    Path(escrow_id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<MediationRequest>,
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

    // Verify signature
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, "mediation:submit")
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

    // Fetch the escrow
    let escrow = queries::get_escrow(&state.db, &escrow_id)
        .await
        .map_err(|_e| crate::types::internal_error())?
        .ok_or_else(|| crate::types::not_found("escrow", &escrow_id))?;

    // Only disputed escrows can be mediated
    if escrow.status != EscrowStatus::Disputed {
        return Err((
            StatusCode::CONFLICT,
            Json(json!(ApiError::new(
                "escrow_not_disputed",
                "Only disputed escrows can be mediated"
            ))),
        ));
    }

    // Verify the caller is a party
    let is_buyer = auth.address == escrow.buyer_address;
    let is_seller = escrow.seller_address.as_deref() == Some(&auth.address);
    if !is_buyer && !is_seller {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "forbidden",
                "Only escrow parties can initiate mediation"
            ))),
        ));
    }

    // Check mediation not already in progress
    if escrow.mediation_status.as_deref() == Some("pending")
        || escrow.mediation_status.as_deref() == Some("completed")
    {
        return Err((
            StatusCode::CONFLICT,
            Json(json!(ApiError::new(
                "mediation_in_progress",
                "Mediation is already in progress or completed for this escrow"
            ))),
        ));
    }

    // Get AI mediator config
    let api_key = state.ai_mediator_api_key.clone().or_else(|| {
        std::env::var("AI_MEDIATOR_API_KEY").ok()
    }).ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!(ApiError::new(
                "ai_mediator_unavailable",
                "AI mediator is not configured"
            ))),
        )
    })?;

    let model = state.ai_mediator_model.clone().unwrap_or_else(|| "deepseek-chat".to_string());
    let mediator = AiMediator::new(api_key, model);

    // Fetch chat messages for context and decrypt them
    let raw_messages = queries::list_messages_raw(&state.db, &escrow_id)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    let mediation_msgs: Vec<MediationMessage> = raw_messages
        .into_iter()
        .filter_map(|(sender, content_enc, nonce, ts)| {
            let plaintext = crate::crypto::decrypt_message(&content_enc, &nonce)?;
            let role = if sender == escrow.buyer_address {
                "buyer".to_string()
            } else {
                "seller".to_string()
            };
            Some(MediationMessage {
                role,
                content: plaintext,
                timestamp: ts,
            })
        })
        .collect();

    // Initiate mediation in DB
    queries::initiate_mediation(
        &state.db,
        &escrow_id,
        &body.buyer_claim,
        &body.seller_claim,
    )
    .await
    .map_err(|_e| crate::types::internal_error())?;

    // Run AI mediation
    let result = mediator
        .mediate(
            &mediation_msgs,
            &body.buyer_claim,
            &body.seller_claim,
            escrow.amount_sompi,
        )
        .await;

    let result = match result {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("AI mediation failed for {}: {e}", escrow_id);
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!(ApiError::new(
                    "mediation_failed",
                    &format!("AI mediation temporarily unavailable: {e}. Try again later.")
                ))),
            ));
        }
    };

    // Store the result
    queries::store_mediation_result(&state.db, &escrow_id, &result)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    // Fetch expires_at
    let status = queries::get_mediation_status(&state.db, &escrow_id)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    let expires_at = status
        .as_ref()
        .and_then(|(_, _, exp)| *exp)
        .unwrap_or_else(|| chrono::Utc::now().timestamp() + 86_400);

    Ok(Json(json!(MediationResponse {
        case_id: escrow_id.clone(),
        recommendation: Some(result),
        expires_at,
        mediation_status: "completed".to_string(),
    })))
}

/// POST /v1/escrows/:id/mediate/:party/accept — accept mediation outcome.
pub async fn accept(
    State(state): State<AppState>,
    Path((escrow_id, party)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<MediationAccept>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if party != "buyer" && party != "seller" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_party",
                "Party must be 'buyer' or 'seller'"
            ))),
        ));
    }

    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new(
                "unauthorized",
                "X-Daglock-* headers required"
            ))),
        )
    })?;

    // Verify signature
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &format!("mediation:accept:{escrow_id}"))
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

    if !body.accept {
        return Err((
            StatusCode::CONFLICT,
            Json(json!(ApiError::new(
                "mediation_declined",
                "Mediation declined — dispute will escalate to jury"
            ))),
        ));
    }

    let accepted = queries::accept_mediation(&state.db, &escrow_id, &party)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    if !accepted {
        return Err((
            StatusCode::CONFLICT,
            Json(json!(ApiError::new(
                "mediation_already_accepted",
                "You have already accepted the mediation outcome"
            ))),
        ));
    }

    // Check if both parties have accepted
    let both = queries::check_mediation_both_accepted(&state.db, &escrow_id)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    if both {
        // Execute the mediation outcome
        let escrow = queries::get_escrow(&state.db, &escrow_id)
            .await
            .map_err(|_e| crate::types::internal_error())?
            .ok_or_else(|| crate::types::not_found("escrow", &escrow_id))?;

        let result_json = escrow.mediation_result.as_deref().unwrap_or("{}");
        let result: MediationResult = serde_json::from_str(result_json).unwrap_or(MediationResult {
            outcome: MediationOutcome::Refund,
            buyer_share_basis: 10000,
            reasoning: "Default: mediation outcome could not be parsed".to_string(),
        });

        match result.outcome {
            MediationOutcome::Refund => {
                let svc = EscrowService::new(
                    state.db.clone(),
                    &state.ws_tx,
                    state.sig_verifier.clone(),
                    state.verifier.clone(),
                    state.email_service.clone(),
                );
                svc.force_refund(&escrow_id).await.map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new("internal_error", &e.to_string()))),
                    )
                })?;
            }
            MediationOutcome::Payout => {
                let svc = EscrowService::new(
                    state.db.clone(),
                    &state.ws_tx,
                    state.sig_verifier.clone(),
                    state.verifier.clone(),
                    state.email_service.clone(),
                );
                svc.force_settle(&escrow_id).await.map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new("internal_error", &e.to_string()))),
                    )
                })?;
            }
            MediationOutcome::Split => {
                // For split, update escrow with split metadata
                let buyer_share = result.buyer_share_basis;
                queries::resolve_dispute_with_split(&state.db, &escrow_id, "mediation_split", buyer_share)
                    .await
                    .map_err(|_e| crate::types::internal_error())?;
            }
        }

        return Ok(Json(json!({
            "status": "mediation_accepted",
            "escrow_id": escrow_id,
            "outcome_executed": true,
            "outcome": result.outcome,
        })));
    }

    Ok(Json(json!({
        "status": "mediation_accepted",
        "party": party,
        "escrow_id": escrow_id,
        "waiting_for_other": true,
    })))
}

/// GET /v1/escrows/:id/mediate — get mediation status.
pub async fn status(
    State(state): State<AppState>,
    Path(escrow_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_escrow(&state.db, &escrow_id)
        .await
        .map_err(|_e| crate::types::internal_error())?
        .ok_or_else(|| crate::types::not_found("escrow", &escrow_id))?;

    let mediation_status = escrow.mediation_status.unwrap_or_default();
    if mediation_status.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "mediation_not_found",
                "No mediation has been initiated for this escrow"
            ))),
        ));
    }

    let result: Option<MediationResult> = escrow
        .mediation_result
        .as_deref()
        .and_then(|r| serde_json::from_str(r).ok());

    let (buyer_accepted, seller_accepted) = (
        escrow.mediation_buyer_accepted.unwrap_or(false),
        escrow.mediation_seller_accepted.unwrap_or(false),
    );

    Ok(Json(json!({
        "escrow_id": escrow_id,
        "mediation_status": mediation_status,
        "recommendation": result,
        "expires_at": escrow.mediation_expires_at,
        "buyer_accepted": buyer_accepted,
        "seller_accepted": seller_accepted,
        "both_accepted": buyer_accepted && seller_accepted,
    })))
}
