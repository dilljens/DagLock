//! Deposit API handlers — security deposit covenant lifecycle.

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

/// POST /v1/escrows/:id/deposit — create/lock a security deposit linked to an escrow
pub async fn create(
    State(state): State<AppState>,
    Path(escrow_id): Path<String>,
    Json(body): Json<CreateDepositRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate the linked escrow exists
    let escrow = queries::get_escrow(&state.db, &escrow_id)
        .await
        .map_err(|_e| crate::types::internal_error())?;
    let _escrow = escrow.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "escrow_not_found",
                format!("No escrow found with id '{escrow_id}'")
            ))),
        )
    })?;

    // Check no deposit already exists for this escrow
    let existing = queries::get_deposit_by_escrow(&state.db, &escrow_id)
        .await
        .map_err(|_e| crate::types::internal_error())?;
    if existing.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(json!(ApiError::new(
                "deposit_exists",
                "A deposit already exists for this escrow"
            ))),
        ));
    }

    if body.deposit_amount <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_amount",
                "Deposit amount must be positive"
            ))),
        ));
    }

    if !body.party1_address.starts_with("kaspa:") || !body.party2_address.starts_with("kaspa:") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_address",
                "Both party addresses must be valid Kaspa addresses"
            ))),
        ));
    }

    let deposit = Deposit {
        id: format!(
            "dep_{}",
            Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .expect("UUID should have a dash")
        ),
        escrow_id: escrow_id.clone(),
        party1_address: body.party1_address,
        party2_address: body.party2_address,
        deposit_amount: body.deposit_amount,
        status: "locked".to_string(),
        deposit_tx_id: body.deposit_tx_id,
        timeout: body.timeout.unwrap_or(0),
        created_at: chrono::Utc::now().timestamp(),
        released_at: None,
        forfeited_at: None,
        forfeited_to: None,
    };

    queries::insert_deposit(&state.db, &deposit)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    Ok((StatusCode::CREATED, Json(json!(deposit))))
}

/// GET /v1/escrows/:id/deposit — get deposit status for an escrow
pub async fn get_by_escrow(
    State(state): State<AppState>,
    Path(escrow_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let deposit = queries::get_deposit_by_escrow(&state.db, &escrow_id)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    match deposit {
        Some(d) => Ok(Json(json!(d))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "deposit_not_found",
                format!("No deposit found for escrow '{escrow_id}'")
            ))),
        )),
    }
}

/// POST /v1/escrows/:id/deposit/release — mutual release of deposits (both signatures)
pub async fn release(
    State(state): State<AppState>,
    Path(escrow_id): Path<String>,
    Json(body): Json<ReleaseDepositRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let deposit = queries::get_deposit_by_escrow(&state.db, &escrow_id)
        .await
        .map_err(|_e| crate::types::internal_error())?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!(ApiError::new(
                    "deposit_not_found",
                    format!("No deposit found for escrow '{escrow_id}'")
                ))),
            )
        })?;

    if deposit.status != "locked" {
        return Err((
            StatusCode::CONFLICT,
            Json(json!(ApiError::new(
                "deposit_already_finalized",
                "Deposit is already finalized"
            ))),
        ));
    }

    if body.party1_address != deposit.party1_address
        || body.party2_address != deposit.party2_address
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "forbidden",
                "Party addresses do not match deposit parties"
            ))),
        ));
    }

    queries::release_deposit(&state.db, &deposit.id)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    Ok(Json(json!({"status": "released", "deposit_id": deposit.id, "escrow_id": escrow_id})))
}

/// POST /v1/escrows/:id/deposit/forfeit — jury forfeit ruling on a deposit
pub async fn forfeit(
    State(state): State<AppState>,
    Path(escrow_id): Path<String>,
    Json(body): Json<ForfeitDepositRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let deposit = queries::get_deposit_by_escrow(&state.db, &escrow_id)
        .await
        .map_err(|_e| crate::types::internal_error())?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!(ApiError::new(
                    "deposit_not_found",
                    format!("No deposit found for escrow '{escrow_id}'")
                ))),
            )
        })?;

    if deposit.status != "locked" {
        return Err((
            StatusCode::CONFLICT,
            Json(json!(ApiError::new(
                "deposit_already_finalized",
                "Deposit is already finalized"
            ))),
        ));
    }

    if !body.forfeited_to.starts_with("kaspa:") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_address",
                "forfeited_to must be a valid Kaspa address"
            ))),
        ));
    }

    queries::forfeit_deposit(&state.db, &deposit.id, &body.forfeited_to)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    Ok(Json(json!({
        "status": "forfeited",
        "deposit_id": deposit.id,
        "escrow_id": escrow_id,
        "forfeited_to": body.forfeited_to
    })))
}

/// POST /v1/deposits/sweep — sweep stale deposits past timeout
pub async fn sweep(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let stale = queries::find_stale_deposits(&state.db)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    let mut swept = Vec::new();
    for deposit in &stale {
        if let Err(e) = queries::sweep_deposit(&state.db, &deposit.id).await {
            tracing::warn!("Failed to sweep deposit {}: {e}", deposit.id);
            continue;
        }
        swept.push(deposit.id.clone());
    }

    Ok(Json(json!({
        "swept": swept,
        "count": swept.len(),
        "total_stale": stale.len()
    })))
}
