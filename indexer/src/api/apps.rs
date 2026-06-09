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
use crate::types::*;

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
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
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
            Json(json!(ApiError::new("app_not_found", format!("No app found with id '{id}'")))),
        )),
    }
}

/// GET /v1/apps/:id/keys
pub async fn list_keys(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let keys = queries::list_api_keys(&state.db, &id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new("internal_error", "An internal error occurred."))),
        )
    })?;

    Ok(Json(json!({ "keys": keys, "total": keys.len() })))
}

/// POST /v1/apps/:id/keys
pub async fn create_key(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Verify app exists
    let app = queries::get_app(&state.db, &id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new("internal_error", "An internal error occurred."))),
        )
    })?;

    if app.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new("app_not_found", "App not found"))),
        ));
    }

    // Generate new key
    let key_plaintext = format!("dl_sk_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
    let key_hash = blake2b_simd::Params::new()
        .hash_length(32)
        .hash(key_plaintext.as_bytes())
        .as_bytes()
        .to_vec();

    let now = chrono::Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO api_keys (key_hash, app_id, label, created_at, is_active)
         VALUES (?1, ?2, 'additional', ?3, 1)"
    )
    .bind(&key_hash)
    .bind(&id)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new("internal_error", "Failed to create key"))),
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

/// DELETE /v1/apps/:id/keys/:key_id
pub async fn delete_key(
    State(state): State<AppState>,
    Path((app_id, key_id)): Path<(String, String)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let revoked = queries::revoke_api_key(&state.db, &key_id, &app_id).await.map_err(|_e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ApiError::new("internal_error", "Failed to revoke key"))),
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
