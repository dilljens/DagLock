//! Vault API handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::Json as JsonBody;
use serde_json::{json, Value};

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;
use crate::types::*;
use sha2::{Digest, Sha256};

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
        Ok(vaults) => {
            let total = queries::count_vaults_by_owner(&state.db, &owner)
                .await
                .unwrap_or(vaults.len() as i64);
            Json(json!(VaultListResponse { vaults, total }))
        }
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
///
/// Create a vault. Requires auth headers proving ownership of the address.
/// The owner_address is set from the authenticated address, not the request body.
pub async fn create(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    JsonBody(body): JsonBody<CreateVaultRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": "X-Daglock-* headers required"})),
        )
    })?;

    // Verify signature: the owner proves they control this address
    let expected_message = "create:vault".to_string();
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &expected_message)
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                Json(json!({"error": "forbidden", "message": "Invalid signature"})),
            )
        })?
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "message": "Invalid signature"})),
        ));
    }

    // Validate addresses
    let valid = crate::api::escrows::validate_kaspa_address(&body.owner_address);
    if !valid {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_address", "message": "Invalid owner Kaspa address"})),
        ));
    }
    if let Some(ref b) = body.beneficiary_address {
        if !crate::api::escrows::validate_kaspa_address(b) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(
                    json!({"error": "invalid_address", "message": "Invalid beneficiary Kaspa address"}),
                ),
            ));
        }
    }

    // Validate amount
    if body.amount_sompi <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_amount", "message": "Amount must be positive"})),
        ));
    }

    // Validate vault type has a matching covenant template configured
    let template_configured = match body.vault_type {
        VaultType::Time => true,
        VaultType::Beneficiary => state.daglock_vault_softlock_template.is_some(),
        VaultType::Multisig => state.daglock_vault_multisig_template.is_some(),
        VaultType::Deadman | VaultType::Inheritance => {
            return Err((
                StatusCode::NOT_IMPLEMENTED,
                Json(
                    json!({"error": "not_implemented", "message": format!("Vault type '{:?}' does not have a covenant yet. Available: Time, Beneficiary, Multisig", body.vault_type)}),
                ),
            ));
        }
    };
    if !template_configured {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"error": "template_not_configured", "message": format!("Vault type '{:?}' requires a template hash configured server-side (--daglock-vault-{}-template)", body.vault_type, match body.vault_type { VaultType::Beneficiary => "softlock", VaultType::Multisig => "multisig", _ => "unknown" })}),
            ),
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

    let vault_id = format!(
        "vault_{}",
        uuid::Uuid::new_v4().to_string().replace('-', "")
    );

    // Owner address is taken from the authenticated user, not the request body
    let vault = crate::types::Vault {
        id: vault_id.clone(),
        owner_address: auth.address.clone(),
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
        owner_pubkey_hex: body.owner_pubkey_hex.clone(),
        sweep_tx_id: None,
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
///
/// Withdraw from a vault. Requires auth headers proving ownership.
/// Message format: "withdraw:{vault_id}"
pub async fn withdraw(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    JsonBody(_body): JsonBody<WithdrawVaultRequest>,
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

    // Auth: extract and verify signature
    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": "X-Daglock-* headers required"})),
        )
    })?;

    let expected_message = format!("withdraw:{}", id);
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &expected_message)
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                Json(json!({"error": "forbidden", "message": "Invalid signature"})),
            )
        })?
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "message": "Invalid signature"})),
        ));
    }

    // Verify ownership — address comes from auth (already verified via sig)
    if auth.address != vault.owner_address {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "message": "Only the vault owner can withdraw"})),
        ));
    }

    // Verify vault is locked
    if vault.status != VaultStatus::Locked {
        return Err((
            StatusCode::CONFLICT,
            Json(
                json!({"error": "invalid_status", "message": format!("Vault is already {}", serde_json::to_string(&vault.status).unwrap_or_default())}),
            ),
        ));
    }

    // Check timeout
    let now = chrono::Utc::now().timestamp();
    if now < vault.timeout {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"error": "timeout_not_reached", "message": "Cannot withdraw before timeout"}),
            ),
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

/// POST /v1/vaults/:id/transfer
///
/// Transfer vault ownership to a beneficiary. Requires auth headers.
/// NOTE: This only updates the beneficiary address in the indexer database.
/// The on-chain covenant is immutable.
pub async fn transfer(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    JsonBody(body): JsonBody<TransferVaultRequest>,
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

    // Auth: extract and verify signature
    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": "X-Daglock-* headers required"})),
        )
    })?;

    let expected_message = format!("transfer:{}", id);
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &expected_message)
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                Json(json!({"error": "forbidden", "message": "Invalid signature"})),
            )
        })?
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "message": "Invalid signature"})),
        ));
    }

    // Verify ownership
    if auth.address != vault.owner_address {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "message": "Only the vault owner can transfer"})),
        ));
    }

    // Update beneficiary
    queries::update_vault_beneficiary(&state.db, &id, &body.beneficiary_address)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": format!("{e}")})),
            )
        })?;

    Ok(Json(json!({
        "status": "transferred",
        "vault_id": id,
        "beneficiary_address": body.beneficiary_address,
    })))
}

/// POST /v1/vaults/:id/sweep
///
/// Sweep vault after timeout. Anyone can trigger.
/// No auth required — the covenant enforces timeout.
pub async fn sweep_vault(
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

    if vault.status != VaultStatus::Locked {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "invalid_status", "message": "Vault is not locked"})),
        ));
    }

    let now = chrono::Utc::now().timestamp();
    if now < vault.timeout {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "timeout_not_reached", "message": "Cannot sweep before timeout"})),
        ));
    }

    queries::update_vault_status(&state.db, &id, "expired")
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": format!("{e}")})),
            )
        })?;

    Ok(Json(json!({
        "status": "expired",
        "vault_id": id,
        "message": "Vault swept after timeout."
    })))
}

/// POST /v1/vaults/:id/relock
///
/// Relock vault — owner extends the timer.
pub async fn relock_vault(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    JsonBody(body): JsonBody<RelockVaultRequest>,
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

    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": "X-Daglock-* headers required"})),
        )
    })?;

    let expected_message = format!("relock:{}", id);
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &expected_message)
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                Json(json!({"error": "forbidden", "message": "Invalid signature"})),
            )
        })?
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "message": "Invalid signature"})),
        ));
    }

    if auth.address != vault.owner_address {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "message": "Only the vault owner can relock"})),
        ));
    }

    if vault.status != VaultStatus::Locked {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "invalid_status", "message": "Vault is not locked"})),
        ));
    }

    let now = chrono::Utc::now().timestamp();
    if body.new_timeout <= now {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_timeout", "message": "New timeout must be in the future"})),
        ));
    }

    sqlx::query("UPDATE vaults SET timeout = ?1 WHERE id = ?2")
        .bind(body.new_timeout)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": format!("{e}")})),
            )
        })?;

    Ok(Json(json!({
        "status": "relocked",
        "vault_id": id,
        "timeout": body.new_timeout,
        "message": "Vault timer extended."
    })))
}

/// POST /v1/vaults/:id/early-exit
///
/// Early exit — owner cancels the vault before timeout.
pub async fn early_exit(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
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

    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": "X-Daglock-* headers required"})),
        )
    })?;

    let expected_message = format!("early_exit:{}", id);
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &expected_message)
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                Json(json!({"error": "forbidden", "message": "Invalid signature"})),
            )
        })?
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "message": "Invalid signature"})),
        ));
    }

    if auth.address != vault.owner_address {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "message": "Only the vault owner can early-exit"})),
        ));
    }

    if vault.status != VaultStatus::Locked {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "invalid_status", "message": "Vault is not locked"})),
        ));
    }

    queries::update_vault_status(&state.db, &id, "expired")
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": format!("{e}")})),
            )
        })?;

    Ok(Json(json!({
        "status": "expired",
        "vault_id": id,
        "message": "Vault expired early by owner."
    })))
}

/// POST /v1/vaults/:id/heir-withdraw
///
/// Heir withdraw — beneficiary withdraws after timeout.
pub async fn heir_withdraw(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
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

    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": "X-Daglock-* headers required"})),
        )
    })?;

    let expected_message = format!("heir_withdraw:{}", id);
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &expected_message)
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                Json(json!({"error": "forbidden", "message": "Invalid signature"})),
            )
        })?
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "message": "Invalid signature"})),
        ));
    }

    if vault.beneficiary_address.as_deref() != Some(&auth.address) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "message": "Only the beneficiary/heir can withdraw"})),
        ));
    }

    if vault.status != VaultStatus::Locked {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "invalid_status", "message": "Vault is not locked"})),
        ));
    }

    let now = chrono::Utc::now().timestamp();
    if now < vault.timeout {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "timeout_not_reached", "message": "Cannot withdraw before timeout"})),
        ));
    }

    queries::update_vault_status(&state.db, &id, "transferred")
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database_error", "message": format!("{e}")})),
            )
        })?;

    Ok(Json(json!({
        "status": "transferred",
        "vault_id": id,
        "message": "Vault transferred to beneficiary."
    })))
}

#[derive(serde::Deserialize)]
pub struct ListParams {
    pub owner: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct RelockVaultRequest {
    pub new_timeout: i64,
}
