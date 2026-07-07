//! Subscription CRUD API handlers.

use axum::http::StatusCode;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::api::AppState;
use crate::db::queries;
use crate::types::*;

/// List subscriptions query parameters.
#[derive(Deserialize)]
pub struct ListQuery {
    pub address: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// POST /v1/subscriptions
pub async fn create(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateSubscriptionRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let svc = crate::services::subscription_service::SubscriptionService::new(
        state.db.clone(),
        state.sig_verifier.clone(),
    );

    let sub = svc
        .create(&body, Some(&headers))
        .await
        .map_err(service_error)?;

    Ok((StatusCode::CREATED, Json(json!(sub))))
}

/// GET /v1/subscriptions
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

    let (subscriptions, total) =
        queries::subscriptions::list_subscriptions_by_address(&state.db, address, limit, offset)
            .await
            .map_err(|_e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!(ApiError::new(
                        "internal_error",
                        "An internal error occurred. Please try again later."
                    ))),
                )
            })?;

    Ok(Json(json!({
        "subscriptions": subscriptions,
        "total": total,
        "limit": limit,
        "offset": offset,
    })))
}

/// GET /v1/subscriptions/{id}
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let sub = queries::subscriptions::get_subscription(&state.db, &id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new(
                    "internal_error",
                    "An internal error occurred. Please try again later."
                ))),
            )
        })?;

    match sub {
        Some(s) => Ok(Json(json!(s))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "subscription_not_found",
                format!("No subscription found with id '{id}'")
            ))),
        )),
    }
}

/// POST /v1/subscriptions/{id}/cancel
pub async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let svc = crate::services::subscription_service::SubscriptionService::new(
        state.db.clone(),
        state.sig_verifier.clone(),
    );

    svc.cancel(&id, &headers)
        .await
        .map(|_| Json(json!({ "status": "cancelled", "subscription_id": id })))
        .map_err(service_error)
}

/// POST /v1/subscriptions/{id}/draw
pub async fn draw(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let svc = crate::services::subscription_service::SubscriptionService::new(
        state.db.clone(),
        state.sig_verifier.clone(),
    );

    svc.draw(&id)
        .await
        .map(|sub| {
            Json(json!({
                "status": "drawn",
                "subscription_id": id,
                "current_period": sub.current_period,
                "max_periods": sub.max_periods,
            }))
        })
        .map_err(service_error)
}

/// Convert a service error to an HTTP error response.
fn service_error(
    e: crate::services::subscription_service::ServiceError,
) -> (StatusCode, Json<Value>) {
    use crate::services::subscription_service::ServiceError;
    let (status, message) = match &e {
        ServiceError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
        ServiceError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        ServiceError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
        ServiceError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
        ServiceError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
        ServiceError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
    };
    (status, Json(json!(ApiError::new(e.error_code(), message))))
}
