#![allow(dead_code)]
//! App registration and API key management for integrator access.

use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};

use crate::api::AppState;
use crate::db::queries;
use crate::ratelimit::ApiTier;
use crate::types::*;
/// Verify X-Daglock-Api-Key header and return app_id on success.
/// Also populates the rate-limiter tier cache.
async fn verify_api_key(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<String, (StatusCode, Json<Value>)> {
    let api_key = headers
        .get("x-daglock-api-key")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!(ApiError::new(
                    "unauthorized",
                    "X-Daglock-Api-Key header required."
                ))),
            )
        })?;

    let key_hash = blake2b_simd::Params::new()
        .hash_length(32)
        .hash(api_key.as_bytes())
        .as_bytes()
        .to_vec();

    let key_info: Option<(String, String, bool)> = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT a.id, k.tier, k.webhooks_enabled FROM apps a
         INNER JOIN api_keys k ON k.app_id = a.id
         WHERE k.key_hash = ?1 AND k.is_active = 1 AND a.is_active = 1",
    )
    .bind(&key_hash)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "Failed to verify API key"
            ))),
        )
    })?
    .map(|(id, tier, wh)| (id, tier, wh != 0));

    match key_info {
        Some((id, tier, _webhooks_enabled)) => {
            // Populate rate-limiter tier cache
            state
                .rate_limiter
                .cache_tier(key_hash.clone(), ApiTier::from(tier.as_str()));

            // Touch last_used_at
            let _ = sqlx::query("UPDATE api_keys SET last_used_at = ?1 WHERE key_hash = ?2")
                .bind(chrono::Utc::now().timestamp())
                .bind(&key_hash)
                .execute(&state.db)
                .await;
            Ok(id)
        }
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new(
                "unauthorized",
                "Invalid or revoked API key"
            ))),
        )),
    }
}

/// POST /v1/apps/register
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterAppRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    if body.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new("invalid_name", "App name is required"))),
        ));
    }
    // Validate owner address
    if !crate::api::escrows::validate_kaspa_address(&body.owner_address) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_address",
                "Invalid owner Kaspa address"
            ))),
        ));
    }

    let (app, api_key) = queries::register_app(
        &state.db,
        body.name.trim(),
        body.callback_url.as_deref(),
        &body.owner_address,
    )
    .await
    .map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "Failed to register app"
            ))),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!(RegisterAppResponse {
            app,
            api_key,
            warning: "Save this API key — it will only be shown once.".to_string(),
        })),
    ))
}

/// GET /v1/apps/:id
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // API key auth required
    let _caller_app_id = verify_api_key(&state, &headers).await?;
    // Verify caller owns this app
    if _caller_app_id != id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "forbidden",
                "API key does not belong to this app"
            ))),
        ));
    }
    let app = queries::get_app(&state.db, &id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "An internal error occurred."
            ))),
        )
    })?;

    match app {
        Some(a) => Ok(Json(json!(a))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "app_not_found",
                format!("No app found with id '{id}'")
            ))),
        )),
    }
}

/// GET /v1/apps/:id/keys
pub async fn list_keys(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // API key auth required
    let _caller_app_id = verify_api_key(&state, &headers).await?;
    if _caller_app_id != id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "forbidden",
                "API key does not belong to this app"
            ))),
        ));
    }
    let keys = queries::list_api_keys(&state.db, &id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "An internal error occurred."
            ))),
        )
    })?;

    Ok(Json(json!({ "keys": keys, "total": keys.len() })))
}

/// POST /v1/apps/:id/keys
pub async fn create_key(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // API key auth required
    let _caller_app_id = verify_api_key(&state, &headers).await?;
    if _caller_app_id != id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "forbidden",
                "API key does not belong to this app"
            ))),
        ));
    }
    // Verify app exists
    let app = queries::get_app(&state.db, &id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "An internal error occurred."
            ))),
        )
    })?;

    if app.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new("app_not_found", "App not found"))),
        ));
    }

    // Generate new key
    let key_plaintext = format!(
        "dl_sk_{}",
        uuid::Uuid::new_v4().to_string().replace('-', "")
    );
    let key_hash = blake2b_simd::Params::new()
        .hash_length(32)
        .hash(key_plaintext.as_bytes())
        .as_bytes()
        .to_vec();

    let now = chrono::Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO api_keys (key_hash, app_id, label, created_at, is_active)
         VALUES (?1, ?2, 'additional', ?3, 1)",
    )
    .bind(&key_hash)
    .bind(&id)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "Failed to create key"
            ))),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "api_key": key_plaintext,
            "app_id": id,
            "warning": "Save this API key — it will only be shown once."
        })),
    ))
}

/// Verify X-Daglock-Admin header matches the configured admin token.
fn verify_admin(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<(), (StatusCode, Json<Value>)> {
    let admin_token = state.admin_token.as_deref().ok_or_else(|| {
        (
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "admin_disabled",
                "Admin endpoints are not configured on this server."
            ))),
        )
    })?;

    let header_token = headers
        .get("x-daglock-admin")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!(ApiError::new(
                    "unauthorized",
                    "X-Daglock-Admin header required."
                ))),
            )
        })?;

    if header_token != admin_token {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new("forbidden", "Invalid admin token."))),
        ));
    }

    Ok(())
}

/// PATCH /v1/apps/:id/keys/:key_id/tier
///
/// Admin-only. Updates the API key's tier and webhooks_enabled flag.
/// Body: { "tier": "free" | "pro" | "whale" }
pub async fn update_key_tier(
    State(state): State<AppState>,
    Path((app_id, key_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    verify_admin(&state, &headers)?;

    let tier = body
        .get("tier")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!(ApiError::new("invalid_tier", "Field 'tier' is required (free, pro, whale)."))),
            )
        })?;

    match tier {
        "free" | "pro" | "whale" => {}
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!(ApiError::new(
                    "invalid_tier",
                    "Tier must be one of: free, pro, whale."
                ))),
            ));
        }
    }

    let updated = queries::update_key_tier(&state.db, &key_id, &app_id, tier).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new(
                "internal_error",
                "Failed to update key tier."
            ))),
        )
    })?;

    if !updated {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new("key_not_found", "Key not found."))),
        ));
    }

    Ok(Json(json!({
        "status": "updated",
        "key_id": key_id,
        "app_id": app_id,
        "tier": tier,
    })))
}

/// DELETE /v1/apps/:id/keys/:key_id
pub async fn delete_key(
    State(state): State<AppState>,
    Path((app_id, key_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // API key auth required
    let _caller_app_id = verify_api_key(&state, &headers).await?;
    if _caller_app_id != app_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "forbidden",
                "API key does not belong to this app"
            ))),
        ));
    }
    let revoked = queries::revoke_api_key(&state.db, &key_id, &app_id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new(
                    "internal_error",
                    "Failed to revoke key"
                ))),
            )
        })?;

    if revoked {
        Ok(Json(json!({ "status": "revoked" })))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new("key_not_found", "Key not found"))),
        ))
    }
}
