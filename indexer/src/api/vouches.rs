//! Vouch API handlers — Web of Trust.

use axum::http::StatusCode;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;
use crate::types::*;

#[derive(Deserialize)]
pub struct VouchQuery {
    pub subject: Option<String>,
    pub voucher: Option<String>,
}

/// POST /v1/vouches
///
/// Vouch for an address. Requires auth headers.
pub async fn create(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateVouchRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Extract authenticated address
    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new(
                "unauthorized",
                "X-Daglock-* headers required"
            ))),
        )
    })?;

    // Cannot vouch for yourself
    if auth.address == body.subject_address {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "self_vouch",
                "You cannot vouch for yourself"
            ))),
        ));
    }

    // Must have at least 3 trades to vouch (anti-Sybil)
    let own_rep = queries::get_reputation(&state.db, &auth.address)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new(
                    "internal_error",
                    "An internal error occurred."
                ))),
            )
        })?;
    if own_rep.trade_count < 3 || own_rep.settled_count < 1 {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "insufficient_reputation",
                "You need at least 3 trades and 1 settlement to vouch"
            ))),
        ));
    }

    // Require valid subject address
    if !body.subject_address.starts_with("kaspa:") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_address",
                "Invalid subject Kaspa address"
            ))),
        ));
    }

    let now = chrono::Utc::now().timestamp();
    let vouch = Vouch {
        id: format!(
            "vch_{}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        ),
        voucher_address: auth.address,
        subject_address: body.subject_address,
        escrow_id: body.escrow_id,
        note: body.note,
        created_at: now,
        expires_at: now + 180 * 86_400, // 6 months
    };

    queries::insert_vouch(&state.db, &vouch)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new(
                    "internal_error",
                    "An internal error occurred."
                ))),
            )
        })?;

    Ok(Json(json!({"status":"created","vouch":vouch})))
}

/// DELETE /v1/vouches/{id}
///
/// Unvouch (revoke a vouch). Must be the original voucher.
pub async fn delete(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new(
                "unauthorized",
                "X-Daglock-* headers required"
            ))),
        )
    })?;

    let deleted = queries::delete_vouch(&state.db, &id, &auth.address)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new(
                    "internal_error",
                    "An internal error occurred."
                ))),
            )
        })?;

    if deleted {
        Ok(Json(json!({ "status": "deleted", "vouch_id": id })))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "vouch_not_found",
                "Vouch not found or you are not the voucher"
            ))),
        ))
    }
}

/// GET /v1/vouches
///
/// List vouches by subject or voucher.
pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<VouchQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (vouches, total): (Vec<Vouch>, i64) = match (&params.subject, &params.voucher) {
        (Some(subject), _) => {
            let v = queries::list_vouches_for_subject(&state.db, subject)
                .await
                .map_err(|_e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new(
                            "internal_error",
                            "An internal error occurred."
                        ))),
                    )
                })?;
            let count = v.len() as i64;
            (v, count)
        }
        (_, Some(voucher_addr)) => {
            let v = queries::list_vouches_by_voucher(&state.db, voucher_addr)
                .await
                .map_err(|_e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new(
                            "internal_error",
                            "An internal error occurred."
                        ))),
                    )
                })?;
            let count = v.len() as i64;
            (v, count)
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!(ApiError::new(
                    "missing_param",
                    "Provide 'subject' or 'voucher' query parameter"
                ))),
            ));
        }
    };

    Ok(Json(json!({
        "vouches": vouches,
        "total": total,
    })))
}
