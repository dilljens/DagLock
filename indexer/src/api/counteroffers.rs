//! Counter-offer API handlers.
//! Users can propose modified terms on an existing offer, creating a negotiation thread.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;
use crate::types::Offer;

#[derive(Deserialize)]
pub struct CreateCounterRequest {
    pub amount_sompi: Option<i64>,
    pub price_offset: Option<f64>,
    pub timeout: Option<i64>,
    pub dispute_mode: Option<String>,
    pub message: Option<String>,
}

/// POST /v1/offers/:id/counter
pub async fn create(
    State(state): State<AppState>,
    Path(offer_id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateCounterRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": format!("{}", e)})),
        )
    })?;

    // Fetch the offer to verify it exists and check ownership
    let offer: Offer = queries::offers::get_offer(&state.db, &offer_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "not_found", "message": "Offer not found"})),
            )
        })?;

    if offer.creator_address == auth.address {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "self_counter", "message": "You cannot counter your own offer"})),
        ));
    }

    if offer.status != "proposed" {
        return Err((
            StatusCode::CONFLICT,
            Json(
                json!({"error": "offer_not_available", "message": "This offer is no longer available"}),
            ),
        ));
    }

    // Anti-spam: max 10 pending counters per offer
    let pending_count = queries::counteroffers::count_pending_for_offer(&state.db, &offer_id)
        .await
        .unwrap_or(0);
    if pending_count >= 10 {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(
                json!({"error": "too_many_counters", "message": "Max 10 pending counters per offer"}),
            ),
        ));
    }

    let id = format!("cnt_{}", Uuid::new_v4().to_string().replace('-', ""));
    queries::counteroffers::create_counteroffer(
        &state.db,
        &id,
        &offer_id,
        &auth.address,
        body.amount_sompi,
        body.price_offset,
        body.timeout,
        body.dispute_mode.as_deref(),
        body.message.as_deref(),
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "db_error", "message": format!("{e}")})),
        )
    })?;

    Ok(Json(json!({
        "status": "counter_created",
        "id": id,
        "offer_id": offer_id,
    })))
}

/// GET /v1/offers/:id/counters
pub async fn list(State(state): State<AppState>, Path(offer_id): Path<String>) -> Json<Value> {
    match queries::counteroffers::list_counteroffers(&state.db, &offer_id).await {
        Ok(counters) => Json(json!({
            "counters": counters,
            "total": counters.len(),
        })),
        Err(e) => Json(json!({
            "error": "db_error",
            "message": format!("{e}"),
            "counters": [],
            "total": 0,
        })),
    }
}

/// POST /v1/counteroffers/:id/accept
pub async fn accept(
    State(state): State<AppState>,
    Path(counter_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": format!("{}", e)})),
        )
    })?;

    let counter = queries::counteroffers::get_counteroffer(&state.db, &counter_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "not_found", "message": "Counter-offer not found"})),
            )
        })?;

    if counter.status != "pending" {
        return Err((
            StatusCode::CONFLICT,
            Json(
                json!({"error": "already_processed", "message": "This counter-offer has already been processed"}),
            ),
        ));
    }

    // Only the original offer creator can accept a counter
    let offer: Offer = queries::offers::get_offer(&state.db, &counter.offer_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "offer_gone", "message": "Original offer no longer exists"})),
            )
        })?;

    if offer.creator_address != auth.address {
        return Err((
            StatusCode::FORBIDDEN,
            Json(
                json!({"error": "forbidden", "message": "Only the offer creator can accept counters"}),
            ),
        ));
    }

    if offer.status != "proposed" {
        return Err((
            StatusCode::CONFLICT,
            Json(
                json!({"error": "offer_gone", "message": "This offer has already been accepted or cancelled"}),
            ),
        ));
    }

    // Mark counter as accepted
    queries::counteroffers::update_counteroffer_status(&state.db, &counter_id, "accepted")
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error"})),
            )
        })?;

    // The frontend should now guide both parties to create the escrow with the negotiated terms.
    // The counter-offer data (amount_sompi, etc.) contains the terms to use.

    Ok(Json(json!({
        "status": "counter_accepted",
        "counter_id": counter_id,
        "offer_id": counter.offer_id,
        "terms": {
            "amount_sompi": counter.amount_sompi,
            "price_offset": counter.price_offset,
        },
        "message": "Counter-offer accepted! Create an escrow with the negotiated terms."
    })))
}

/// POST /v1/counteroffers/:id/decline
pub async fn decline(
    State(state): State<AppState>,
    Path(counter_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": format!("{}", e)})),
        )
    })?;

    let counter = queries::counteroffers::get_counteroffer(&state.db, &counter_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "not_found", "message": "Counter-offer not found"})),
            )
        })?;

    // Either the offer creator or the proposer can decline/withdraw
    if auth.address != counter.proposer_address {
        let offer: Offer = queries::offers::get_offer(&state.db, &counter.offer_id)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "db_error"})),
                )
            })?
            .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"error": "offer_gone"}))))?;

        if offer.creator_address != auth.address {
            return Err((
                StatusCode::FORBIDDEN,
                Json(
                    json!({"error": "forbidden", "message": "Only participants in this negotiation can decline"}),
                ),
            ));
        }
    }

    queries::counteroffers::update_counteroffer_status(&state.db, &counter_id, "declined")
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error"})),
            )
        })?;

    Ok(Json(json!({
        "status": "counter_declined",
        "counter_id": counter_id,
    })))
}
