//! Trade feedback API handlers.
//! Buyers/sellers can leave 1-5 star ratings + comments after settlement.

use axum::extract::Path;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;

#[derive(Deserialize)]
pub struct CreateFeedbackRequest {
    pub rating: i32,
    pub comment: Option<String>,
}

/// POST /v1/escrows/:id/feedback
pub async fn create(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<CreateFeedbackRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": format!("{}", e)})),
        )
    })?;

    // Validate rating
    if body.rating < 1 || body.rating > 5 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_rating", "message": "Rating must be between 1 and 5"})),
        ));
    }

    // Fetch escrow to verify it's settled and reviewer is involved
    let escrow = queries::escrows::get_escrow(&state.db, &id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error", "message": format!("{e}")})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "not_found", "message": "Escrow not found"})),
            )
        })?;

    use crate::types::EscrowStatus;
    if !matches!(escrow.status, EscrowStatus::Settled) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "not_settled", "message": "Can only leave feedback on settled escrows"})),
        ));
    }

    // Only buyer or seller can leave feedback
    let buyer = escrow.buyer_address.to_string();
    let seller = escrow.seller_address.as_ref().map(|s| s.to_string());
    if auth.address != buyer && Some(&auth.address) != seller.as_ref() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "not_participant", "message": "Only buyer or seller can leave feedback"})),
        ));
    }

    let feedback_id = format!("fb_{}", Uuid::new_v4().to_string().replace('-', ""));
    queries::feedback::upsert_feedback(
        &state.db,
        &feedback_id,
        &id,
        &auth.address,
        body.rating,
        body.comment.as_deref(),
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "db_error", "message": format!("{e}")})),
        )
    })?;

    Ok(Json(json!({
        "status": "feedback_submitted",
        "id": feedback_id,
        "rating": body.rating,
    })))
}

/// GET /v1/escrows/:id/feedback
pub async fn list(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Value> {
    let feedback = queries::feedback::get_feedback_for_escrow(&state.db, &id)
        .await
        .unwrap_or_default();

    let total = feedback.len();
    let average_rating = if total > 0 {
        feedback.iter().map(|f| f.rating as f64).sum::<f64>() / total as f64
    } else {
        0.0
    };

    Json(json!({
        "feedback": feedback,
        "average_rating": (average_rating * 100.0).round() / 100.0,
        "total": total,
    }))
}
