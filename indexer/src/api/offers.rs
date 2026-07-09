//! Offer board API handlers.

use axum::http::StatusCode;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;
use crate::services::webhooks::{self, WebhookEvent};
use crate::types::*;
use crate::websocket::WsEvent;

/// POST /v1/offers
pub async fn create(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateOfferRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Verify caller is the creator
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new("unauthorized", e.to_string()))),
        )
    })?;
    // Verify signature
    let expected_message = format!("create:offer:{}", body.creator_address);
    if !state.sig_verifier.verify_signature(&auth.address, &auth.signature, &expected_message)
        .map_err(|e| (StatusCode::FORBIDDEN, Json(json!(ApiError::new("forbidden", format!("Signature verification failed: {e}"))))))?
    {
        return Err((StatusCode::FORBIDDEN, Json(json!(ApiError::new("forbidden", "Invalid signature")))));
    }
    if auth.address != body.creator_address {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "forbidden",
                "Creator address must match the signed address"
            ))),
        ));
    }
    // Validate addresses
    let valid_addr = |a: &str| crate::api::escrows::validate_kaspa_address(a);
    if !valid_addr(&body.creator_address) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_address",
                "Invalid creator Kaspa address"
            ))),
        ));
    }
    if let Some(ref c) = body.counterparty_address {
        if !valid_addr(c) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!(ApiError::new(
                    "invalid_address",
                    "Invalid counterparty Kaspa address"
                ))),
            ));
        }
    }
    let price_type = body.price_type.unwrap_or_else(|| "fixed".to_string());
    let current_price = if price_type == "market" {
        crate::types::fetch_kas_usd_price().await
    } else {
        None
    };

    let creator_address = &body.creator_address;

    // Rate limit: max 50 offers per address per day
    let recent_count = queries::count_offers_by_creator_recent(&state.db, creator_address, 86400)
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
    if recent_count >= 50 {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!(ApiError::new(
                "rate_limited",
                "Max 50 offers per day per address"
            ))),
        ));
    }

    // Default expiry: 72 hours from now if not specified
    let expires_at = body.expires_at.or_else(|| {
        Some(chrono::Utc::now().timestamp() + 259_200) // 72 hours
    });

    // Determine creator_type: explicit > account_flags > default "user"
    let creator_type = match body.creator_type {
        Some(ref t) if t == "bot" || t == "user" => t.clone(),
        _ => {
            // Check account_flags for this address
            match queries::flags::is_bot_address(&state.db, creator_address).await {
                Ok(true) => "bot".to_string(),
                _ => "user".to_string(),
            }
        }
    };

    let offer = Offer {
        id: format!(
            "off_{}",
            Uuid::new_v4().to_string().replace('-', "")
        ),
        creator_address: creator_address.clone(),
        side: body.side,
        base_asset: body.base_asset,
        quote_asset: body.quote_asset,
        amount_sompi: body.amount_sompi,
        counterparty_address: body.counterparty_address,
        status: "proposed".to_string(),
        expires_at,
        created_at: chrono::Utc::now().timestamp(),
        price_type,
        price_offset: body.price_offset,
        min_price: body.min_price,
        max_price: body.max_price,
        current_price,
        price_currency: "USD".to_string(),
        price_updated_at: if current_price.is_some() {
            Some(chrono::Utc::now().timestamp())
        } else {
            None
        },
        creator_type,
    };

    queries::insert_offer(&state.db, &offer)
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

    webhooks::dispatch(state.db.clone(), WebhookEvent::OfferCreated(&offer.id));
    let _ = state.ws_tx.send(WsEvent::offer_created(&offer.id));
    Ok((StatusCode::CREATED, Json(json!(offer))))
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct OfferQuery {
    pub creator: Option<String>,
    pub asset: Option<String>,
    pub side: Option<String>,
    pub status: Option<String>,
}

/// GET /v1/offers
pub async fn list(State(state): State<AppState>, Query(params): Query<OfferQuery>) -> Json<Value> {
    if let Some(ref creator) = params.creator {
        // Filter by creator
        return match queries::list_offers_by_creator(&state.db, creator).await {
            Ok((offers, total)) => Json(json!({ "offers": offers, "total": total })),
            Err(_) => Json(json!({ "offers": [], "total": 0 })),
        };
    }

    match queries::list_offers(&state.db, None, None, None).await {
        Ok((offers, total)) => Json(json!({
            "offers": offers,
            "total": total,
        })),
        Err(_e) => Json(json!(ApiError::new(
            "internal_error",
            "An internal error occurred."
        ))),
    }
}

/// POST /v1/offers/{id}/accept
pub async fn accept(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<AcceptOfferRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new("unauthorized", e.to_string()))),
        )
    })?;
    // Verify signature
    let expected_message = format!("accept:offer:{}", id);
    if !state.sig_verifier.verify_signature(&auth.address, &auth.signature, &expected_message)
        .map_err(|e| (StatusCode::FORBIDDEN, Json(json!(ApiError::new("forbidden", format!("Signature verification failed: {e}"))))))?
    {
        return Err((StatusCode::FORBIDDEN, Json(json!(ApiError::new("forbidden", "Invalid signature")))));
    }
    if auth.address != body.counterparty_address {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "forbidden",
                "Counterparty address must match the signed address"
            ))),
        ));
    }
    // Check offer exists and is proposed
    let offer = queries::get_offer(&state.db, &id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "An internal error occurred."
            ))),
        )
    })?;

    match offer {
        Some(o) if o.status == "proposed" => {
            // Cannot accept your own offer
            if body.counterparty_address == o.creator_address {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!(ApiError::new(
                        "self_accept",
                        "You cannot accept your own offer"
                    ))),
                ));
            }
            queries::accept_offer(&state.db, &id, &body.counterparty_address)
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

            webhooks::dispatch(state.db.clone(), WebhookEvent::OfferAccepted(&o.id));
            let _ = state.ws_tx.send(WsEvent::offer_accepted(&o.id));
            Ok(Json(json!({ "status": "accepted", "offer_id": id })))
        }
        Some(_) => Err((
            StatusCode::CONFLICT,
            Json(json!(ApiError::new(
                "offer_not_available",
                "Offer is no longer available"
            ))),
        )),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "offer_not_found",
                format!("No offer found with id '{id}'")
            ))),
        )),
    }
}

/// POST /v1/offers/{id}/cancel
pub async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let offer = queries::get_offer(&state.db, &id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "An internal error occurred."
            ))),
        )
    })?;

    match offer {
        Some(o) if o.status == "proposed" || o.status == "accepted" => {
            // Auth: only the creator can cancel
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
            let expected_message = format!("cancel:offer:{}", id);
            if !state.sig_verifier.verify_signature(&auth.address, &auth.signature, &expected_message)
                .map_err(|e| (StatusCode::FORBIDDEN, Json(json!(ApiError::new("forbidden", format!("Signature verification failed: {e}"))))))?
            {
                return Err((StatusCode::FORBIDDEN, Json(json!(ApiError::new("forbidden", "Invalid signature")))));
            }
            if auth.address != o.creator_address {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!(ApiError::new(
                        "forbidden",
                        "Only the creator can cancel this offer"
                    ))),
                ));
            }
            queries::update_offer_status(&state.db, &id, "cancelled")
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

            let _ = state.ws_tx.send(WsEvent::offer_cancelled(&id));
            Ok(Json(json!({ "status": "cancelled", "offer_id": id })))
        }
        Some(_) => Err((
            StatusCode::CONFLICT,
            Json(json!(ApiError::new(
                "offer_finalized",
                "Offer already finalized"
            ))),
        )),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "offer_not_found",
                format!("No offer found with id '{id}'")
            ))),
        )),
    }
}
