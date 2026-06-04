//! Vault API handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::Json as JsonBody;
use serde_json::{json, Value};

use crate::api::AppState;
use crate::db::queries;
use crate::types::{
    CreateVaultRequest, VaultListResponse, VaultStatus, WithdrawVaultRequest,
};

/// GET /v1/vaults?owner=...
pub async fn list(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<ListParams>,
) -> Json<Value> {
    let owner = match params.owner {
        Some(o) => o,
        None => {
            return Json(json!({
                "error": "missing_query_parameter",
                "message": "Query parameter 'owner' is required"
            }));
        }
    };

    match queries::list_vaults_by_owner(&state.db, &owner).await {
        Ok(vaults) => Json(json!(VaultListResponse {
            vaults,
            total: 0, // TODO: count query
        })),
        Err(e) => Json(json!({
            "error": "database_error",
            "message": format!("Failed to list vaults: {e}")
        })),
    }
}

/// GET /v1/vaults/:id
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let vault = queries::get_vault(&state.db, &id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": format!("{e}")})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "not_found", "message": format!("Vault '{id}' not found")})),
            )
        })?;

    Ok(Json(json!(vault)))
}

/// POST /v1/vaults
pub async fn create(
    State(state): State<AppState>,
    JsonBody(body): JsonBody<CreateVaultRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate amount
    if body.amount_sompi <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_amount", "message": "Amount must be positive"})),
        ));
    }

    // Validate timeout is in the future
    let now = chrono::Utc::now().timestamp();
    if body.timeout <= now {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_timeout", "message": "Timeout must be in the future"})),
        ));
    }

    let vault_id = format!("vault_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());

    let vault = crate::types::Vault {
        id: vault_id.clone(),
        owner_address: body.owner_address.clone(),
        beneficiary_address: body.beneficiary_address.clone(),
        vault_type: body.vault_type,
        status: VaultStatus::Locked,
        amount_sompi: body.amount_sompi,
        timeout: body.timeout,
        lock_tx_id: body.lock_tx_id.clone(),
        lock_tx_output_index: body.lock_tx_output_index,
        created_at: now,
        unlocked_at: None,
        expires_at: None,
    };

    queries::insert_vault(&state.db, &vault)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": format!("{e}")})),
            )
        })?;

    Ok((StatusCode::CREATED, Json(json!(vault))))
}

/// POST /v1/vaults/:id/withdraw
pub async fn withdraw(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<WithdrawVaultRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let vault = queries::get_vault(&state.db, &id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": format!("{e}")})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "not_found", "message": format!("Vault '{id}' not found")})),
            )
        })?;

    // Verify ownership
    if vault.owner_address != body.owner_address {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "message": "Only the vault owner can withdraw"})),
        ));
    }

    // Verify vault is locked
    if vault.status != VaultStatus::Locked {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "invalid_status", "message": format!("Vault is already {}", serde_json::to_string(&vault.status).unwrap_or_default())})),
        ));
    }

    // Check timeout
    let now = chrono::Utc::now().timestamp();
    if now < vault.timeout {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "timeout_not_reached", "message": "Cannot withdraw before timeout"})),
        ));
    }

    // Update status
    queries::update_vault_status(&state.db, &id, "unlocked")
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": format!("{e}")})),
            )
        })?;

    Ok(Json(json!({
        "status": "unlocked",
        "vault_id": id,
        "message": "Vault unlocked. Funds can now be withdrawn."
    })))
}

#[derive(serde::Deserialize)]
pub struct ListParams {
    pub owner: Option<String>,
}
