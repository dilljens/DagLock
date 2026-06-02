//! Escrow CRUD API handlers.

use axum::http::StatusCode;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::AppState;
use crate::db::queries;
use crate::types::*;

/// List escrows query parameters.
#[derive(Deserialize)]
pub struct ListQuery {
    pub address: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct DisputeRequest {
    pub reason: String,
}

/// GET /v1/escrows
pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let address = params.address.as_deref().unwrap_or("");
    if address.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_address",
                "address query parameter is required"
            ))),
        ));
    }

    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let (escrows, total) = queries::list_escrows_by_address(
        &state.db,
        address,
        params.role.as_deref(),
        params.status.as_deref(),
        limit,
        offset,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new("internal_error", e.to_string()))),
        )
    })?;

    Ok(Json(json!({
        "escrows": escrows,
        "total": total,
        "limit": limit,
        "offset": offset,
    })))
}

/// GET /v1/escrows/{id}
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_escrow(&state.db, &id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new("internal_error", e.to_string()))),
        )
    })?;

    match escrow {
        Some(e) => Ok(Json(json!(e))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "escrow_not_found",
                format!("No escrow found with id '{id}'")
            ))),
        )),
    }
}

/// POST /v1/escrows
pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateEscrowRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate required fields
    if body.amount_sompi <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_amount",
                "amount must be positive"
            ))),
        ));
    }

    let fee_sompi = body.amount_sompi / 200; // 0.5%

    let escrow = Escrow {
        id: format!(
            "esc_{}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        ),
        lock_tx_id: body.lock_tx_id,
        lock_tx_output_index: body.lock_tx_output_index,
        status: EscrowStatus::PendingConfirmation,
        asset_type: body.asset_type.unwrap_or_else(|| "KAS".to_string()),
        buyer_address: body.buyer_address,
        seller_address: body.seller_address,
        amount_sompi: body.amount_sompi,
        fee_sompi,
        template_hash: body.template_hash.unwrap_or_default(),
        expiration_daa_score: body.expiration_daa_score,
        disputed_at: None,
        dispute_reason: None,
        cancelled_at: None,
        expired_at: None,
        created_at: chrono::Utc::now().timestamp(),
        settled_at: None,
        refunded_at: None,
    };

    queries::insert_escrow(&state.db, &escrow)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new("internal_error", e.to_string()))),
            )
        })?;

    Ok((StatusCode::CREATED, Json(json!(escrow))))
}

/// GET /v1/stats
pub async fn stats(State(state): State<AppState>) -> Json<Value> {
    match queries::get_stats(&state.db).await {
        Ok(s) => Json(json!(s)),
        Err(e) => Json(json!(ApiError::new("internal_error", e.to_string()))),
    }
}

/// POST /v1/escrows/{id}/settle
pub async fn settle(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_escrow(&state.db, &id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new("internal_error", e.to_string()))),
        )
    })?;

    match escrow {
        Some(current)
            if matches!(
                current.status,
                EscrowStatus::Settled
                    | EscrowStatus::Refunded
                    | EscrowStatus::Cancelled
                    | EscrowStatus::Expired
                    | EscrowStatus::PendingConfirmation
            ) =>
        {
            Err((
                StatusCode::CONFLICT,
                Json(json!(ApiError::new(
                    "escrow_already_finalized",
                    "Escrow is already finalized"
                ))),
            ))
        }
        Some(_) => {
            queries::update_escrow_status(&state.db, &id, EscrowStatus::Settled)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new("internal_error", e.to_string()))),
                    )
                })?;
            sqlx::query("UPDATE escrows SET settled_at = ?1, refunded_at = NULL WHERE id = ?2")
                .bind(chrono::Utc::now().timestamp())
                .bind(&id)
                .execute(&state.db)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new("internal_error", e.to_string()))),
                    )
                })?;
            Ok(Json(json!({ "status": "settled", "escrow_id": id })))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "escrow_not_found",
                format!("No escrow found with id '{id}'")
            ))),
        )),
    }
}

/// POST /v1/escrows/{id}/refund
pub async fn refund(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_escrow(&state.db, &id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new("internal_error", e.to_string()))),
        )
    })?;

    match escrow {
        Some(current)
            if matches!(
                current.status,
                EscrowStatus::Settled
                    | EscrowStatus::Refunded
                    | EscrowStatus::Cancelled
                    | EscrowStatus::Expired
                    | EscrowStatus::PendingConfirmation
            ) =>
        {
            Err((
                StatusCode::CONFLICT,
                Json(json!(ApiError::new(
                    "escrow_already_finalized",
                    "Escrow is already finalized"
                ))),
            ))
        }
        Some(_) => {
            queries::update_escrow_status(&state.db, &id, EscrowStatus::Refunded)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new("internal_error", e.to_string()))),
                    )
                })?;
            sqlx::query("UPDATE escrows SET refunded_at = ?1, settled_at = NULL WHERE id = ?2")
                .bind(chrono::Utc::now().timestamp())
                .bind(&id)
                .execute(&state.db)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new("internal_error", e.to_string()))),
                    )
                })?;
            Ok(Json(json!({ "status": "refunded", "escrow_id": id })))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "escrow_not_found",
                format!("No escrow found with id '{id}'")
            ))),
        )),
    }
}

/// POST /v1/escrows/{id}/dispute
pub async fn dispute(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<DisputeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_escrow(&state.db, &id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new("internal_error", e.to_string()))),
        )
    })?;

    match escrow {
        Some(current)
            if matches!(
                current.status,
                EscrowStatus::Settled
                    | EscrowStatus::Refunded
                    | EscrowStatus::Cancelled
                    | EscrowStatus::Expired
                    | EscrowStatus::Disputed
            ) =>
        {
            Err((
                StatusCode::CONFLICT,
                Json(json!(ApiError::new(
                    "escrow_already_finalized",
                    "Escrow cannot be disputed"
                ))),
            ))
        }
        Some(_) => {
            queries::mark_escrow_disputed(&state.db, &id, body.reason.as_str())
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new("internal_error", e.to_string()))),
                    )
                })?;
            Ok(Json(json!({ "status": "disputed", "escrow_id": id })))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "escrow_not_found",
                format!("No escrow found with id '{id}'")
            ))),
        )),
    }
}

/// POST /v1/escrows/{id}/cancel
pub async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let escrow = queries::get_escrow(&state.db, &id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new("internal_error", e.to_string()))),
        )
    })?;

    match escrow {
        Some(current)
            if matches!(
                current.status,
                EscrowStatus::Settled
                    | EscrowStatus::Refunded
                    | EscrowStatus::Cancelled
                    | EscrowStatus::Expired
            ) =>
        {
            Err((
                StatusCode::CONFLICT,
                Json(json!(ApiError::new(
                    "escrow_already_finalized",
                    "Escrow cannot be cancelled"
                ))),
            ))
        }
        Some(_) => {
            queries::mark_escrow_cancelled(&state.db, &id)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new("internal_error", e.to_string()))),
                    )
                })?;
            Ok(Json(json!({ "status": "cancelled", "escrow_id": id })))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "escrow_not_found",
                format!("No escrow found with id '{id}'")
            ))),
        )),
    }
}
