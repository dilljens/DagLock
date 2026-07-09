//! Email notification API handlers.
//! Users can subscribe to email alerts for escrow events.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use rand::Rng;

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;

#[derive(Deserialize)]
pub struct SubscribeRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub code: String,
}

#[derive(Deserialize)]
pub struct UpdatePreferencesRequest {
    pub notify_created: Option<bool>,
    pub notify_settled: Option<bool>,
    pub notify_disputed: Option<bool>,
    pub notify_refunded: Option<bool>,
    pub notify_expired: Option<bool>,
}

/// POST /v1/notifications/subscribe
pub async fn subscribe(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<SubscribeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": format!("{}", e)})),
        )
    })?;

    // Validate email
    if !body.email.contains('@') || !body.email.contains('.') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_email", "message": "Invalid email address"})),
        ));
    }

    // Generate verification code
    let code: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
    let code_upper = code.to_uppercase();

    queries::upsert_subscription(&state.db, &auth.address, &body.email, &code_upper)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error", "message": format!("{e}")})),
            )
        })?;

    // Send verification email if configured
    if let Some(email_service) = state.email_service.as_ref() {
        if email_service.is_configured() {
            if let Err(e) = email_service.send_verification(&body.email, &auth.address, &code_upper).await {
                tracing::warn!("Failed to send verification email: {e}");
            }
        }
    }

    Ok(Json(json!({
        "status": "subscribed",
        "message": "Verification code sent to email. Use POST /v1/notifications/verify to confirm.",
        "email": body.email,
    })))
}

/// POST /v1/notifications/verify
pub async fn verify(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<VerifyRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": format!("{}", e)})),
        )
    })?;

    let verified = queries::verify_email(&state.db, &auth.address, &body.code.to_uppercase())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error", "message": format!("{e}")})),
            )
        })?;

    if !verified {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_code", "message": "Invalid verification code"})),
        ));
    }

    Ok(Json(json!({
        "status": "verified",
        "message": "Email verified! You will now receive escrow notifications.",
    })))
}

/// GET /v1/notifications
pub async fn get(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": format!("{}", e)})),
        )
    })?;

    let sub = queries::get_subscription(&state.db, &auth.address)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error", "message": format!("{e}")})),
            )
        })?;

    match sub {
        Some(s) => Ok(Json(json!(s))),
        None => Ok(Json(json!({
            "address": auth.address,
            "email": null,
            "email_verified": false,
            "notify_created": true,
            "notify_settled": true,
            "notify_disputed": true,
            "notify_refunded": true,
            "notify_expired": true,
        }))),
    }
}

/// PATCH /v1/notifications
pub async fn update_preferences(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpdatePreferencesRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": format!("{}", e)})),
        )
    })?;

    queries::update_preferences(
        &state.db,
        &auth.address,
        body.notify_created,
        body.notify_settled,
        body.notify_disputed,
        body.notify_refunded,
        body.notify_expired,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "db_error", "message": format!("{e}")})),
        )
    })?;

    Ok(Json(json!({"status": "preferences_updated"})))
}
