//! Offer board API handlers.

use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::AppState;
use crate::db::queries;
use crate::types::*;

/// POST /v1/offers
pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateOfferRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let offer = Offer {
        id: format!(
            "off_{}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        ),
        creator_address: body.creator_address,
        side: body.side,
        base_asset: body.base_asset,
        quote_asset: body.quote_asset,
        amount_sompi: body.amount_sompi,
        counterparty_address: body.counterparty_address,
        status: "proposed".to_string(),
        expires_at: body.expires_at,
        created_at: chrono::Utc::now().timestamp(),
    };

    queries::insert_offer(&state.db, &offer)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new("internal_error", e.to_string()))),
            )
        })?;

    Ok((StatusCode::CREATED, Json(json!(offer))))
}

/// GET /v1/offers
pub async fn list(State(state): State<AppState>) -> Json<Value> {
    match queries::list_offers(&state.db, None, None, None).await {
        Ok((offers, total)) => Json(json!({
            "offers": offers,
            "total": total,
        })),
        Err(e) => Json(json!(ApiError::new("internal_error", e.to_string()))),
    }
}

/// POST /v1/offers/{id}/accept
pub async fn accept(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AcceptOfferRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check offer exists and is proposed
    let offer = queries::get_offer(&state.db, &id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new("internal_error", e.to_string()))),
        )
    })?;

    match offer {
        Some(o) if o.status == "proposed" => {
            queries::accept_offer(&state.db, &id, &body.counterparty_address)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new("internal_error", e.to_string()))),
                    )
                })?;

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
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let offer = queries::get_offer(&state.db, &id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new("internal_error", e.to_string()))),
        )
    })?;

    match offer {
        Some(o) if o.status == "proposed" || o.status == "accepted" => {
            queries::update_offer_status(&state.db, &id, "cancelled")
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new("internal_error", e.to_string()))),
                    )
                })?;

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
