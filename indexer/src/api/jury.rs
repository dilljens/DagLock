//! Jury API handlers — community dispute resolution.

use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;
use crate::types::*;

pub fn juror_count_and_threshold(amount_sompi: i64) -> (i64, i64) {
    let kas = amount_sompi / 100_000_000;
    if kas < 10_000 {
        (3, 2)
    } else if kas < 100_000 {
        (5, 3)
    } else {
        (9, 5)
    }
}

/// POST /v1/jury/register — opt in as a juror
pub async fn register(
    State(state): State<AppState>,
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

    // Verify signature
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, "jury:register")
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "invalid_signature",
                "Signature does not match the claimed address"
            ))),
        ));
    }

    // Check minimum requirements
    let rep = queries::get_reputation(&state.db, &auth.address)
        .await
        .map_err(|_e| crate::types::internal_error())?;
    if rep.trade_count < 10 || rep.score < 3.0 {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "insufficient_reputation",
                "Need at least 10 trades and 3.0+ score to be a juror"
            ))),
        ));
    }

    queries::register_juror(&state.db, &auth.address)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    Ok(Json(
        json!({"status": "registered", "address": auth.address}),
    ))
}

/// POST /v1/jury/unregister
pub async fn unregister(
    State(state): State<AppState>,
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

    // Verify signature
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, "jury:unregister")
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "invalid_signature",
                "Signature does not match the claimed address"
            ))),
        ));
    }

    let removed = queries::unregister_juror(&state.db, &auth.address)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    if !removed {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "not_registered",
                "You are not registered as a juror"
            ))),
        ));
    }

    Ok(Json(
        json!({"status": "unregistered", "address": auth.address}),
    ))
}

/// GET /v1/jury/cases — list active jury cases for the caller
pub async fn list_cases(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Expire stale cases (72h timeout) before listing
    let _ = queries::expire_stale_jury_cases(&state.db).await;
    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new(
                "unauthorized",
                "X-Daglock-* headers required"
            ))),
        )
    })?;

    let cases = queries::list_active_jury_cases_for_juror(&state.db, &auth.address)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    Ok(Json(json!({"cases": cases, "total": cases.len() as i64})))
}

/// GET /v1/jury/cases/active/:address — count active cases for a juror (no auth)
pub async fn active_cases(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _ = queries::expire_stale_jury_cases(&state.db).await;

    let cases = queries::list_active_jury_cases_for_juror(&state.db, &address)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    Ok(Json(json!({"count": cases.len() as i64, "cases": cases})))
}

/// GET /v1/jury/cases/:id — get case details
pub async fn get_case(
    State(state): State<AppState>,
    Path(case_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let case = queries::get_jury_case(&state.db, &case_id)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    match case {
        Some(c) => Ok(Json(json!(c))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "case_not_found",
                format!("No jury case found with id '{case_id}'")
            ))),
        )),
    }
}

/// POST /v1/jury/cases/:id/vote — cast a vote
pub async fn cast_vote(
    State(state): State<AppState>,
    Path(case_id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CastVoteRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if body.vote != "seller_wins" && body.vote != "buyer_wins" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_vote",
                "Vote must be 'seller_wins' or 'buyer_wins'"
            ))),
        ));
    }

    let auth = AuthContext::from_headers(&headers).map_err(|_e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new(
                "unauthorized",
                "X-Daglock-* headers required"
            ))),
        )
    })?;

    // Verify signature
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &format!("vote:{}", case_id))
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "invalid_signature",
                "Signature does not match the claimed address"
            ))),
        ));
    }

    // Verify this juror is assigned to this case
    let case = queries::get_jury_case(&state.db, &case_id)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    let case = case.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!(ApiError::new(
                "case_not_found",
                format!("No jury case found with id '{case_id}'")
            ))),
        )
    })?;

    if case.status != "voting" {
        return Err((
            StatusCode::CONFLICT,
            Json(json!(ApiError::new(
                "case_closed",
                "This jury case is no longer accepting votes"
            ))),
        ));
    }

    if !case.jurors.contains(&auth.address) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new(
                "not_juror",
                "You are not assigned to this case"
            ))),
        ));
    }

    queries::cast_jury_vote(
        &state.db,
        &case_id,
        &auth.address,
        &body.vote,
        body.reasoning.as_deref(),
    )
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

    // Check if this vote triggers a verdict
    let verdict = queries::check_jury_verdict(&state.db, &case_id)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    Ok(Json(json!({
        "status": "voted",
        "vote": body.vote,
        "verdict": verdict,
    })))
}

/// GET /v1/jury/candidates — list eligible jurors
pub async fn list_candidates(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let jurors = queries::list_eligible_jurors_simple(&state.db)
        .await
        .map_err(|_e| crate::types::internal_error())?;

    Ok(Json(
        json!({"candidates": jurors, "total": jurors.len() as i64}),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn juror_count_small_escrow() {
        let (count, threshold) = juror_count_and_threshold(500_000_000_000); // 5_000 KAS
        assert_eq!(count, 3, "Under 10K KAS -> 3 jurors");
        assert_eq!(threshold, 2, "Under 10K KAS -> threshold 2");
    }

    #[test]
    fn juror_count_medium_escrow() {
        let (count, threshold) = juror_count_and_threshold(5_000_000_000_000); // 50_000 KAS
        assert_eq!(count, 5, "10K-100K KAS -> 5 jurors");
        assert_eq!(threshold, 3, "10K-100K KAS -> threshold 3");
    }

    #[test]
    fn juror_count_large_escrow() {
        let (count, threshold) = juror_count_and_threshold(50_000_000_000_000); // 500_000 KAS
        assert_eq!(count, 9, "Over 100K KAS -> 9 jurors");
        assert_eq!(threshold, 5, "Over 100K KAS -> threshold 5");
    }

    #[test]
    fn juror_count_boundary_just_below_10k() {
        let (count, _) = juror_count_and_threshold(999_999_999_999); // 9_999.99 KAS
        assert_eq!(count, 3, "Just below 10K KAS -> 3 jurors");
    }

    #[test]
    fn juror_count_boundary_at_10k() {
        let (count, _) = juror_count_and_threshold(1_000_000_000_000); // 10_000 KAS
        assert_eq!(count, 5, "At 10K KAS -> 5 jurors");
    }

    #[test]
    fn juror_count_boundary_at_100k() {
        let (count, _) = juror_count_and_threshold(10_000_000_000_000); // 100_000 KAS
        assert_eq!(count, 9, "At 100K KAS -> 9 jurors");
    }

    #[test]
    fn jury_selection_distribution_is_reasonably_uniform() {
        use std::collections::HashMap;

        let pool: Vec<usize> = (0..100).collect();
        let mut counts: HashMap<usize, usize> = HashMap::new();

        // Run selection 10_000 times and track which indices get picked
        let needed = 20;
        for _ in 0..10_000 {
            let mut indices: Vec<usize> = (0..pool.len()).collect();
            for i in (pool.len() - needed..pool.len()).rev() {
                let j = rand::thread_rng().gen_range(0..=i);
                indices.swap(i, j);
            }
            let selected: Vec<&usize> = indices[pool.len() - needed..].iter().collect();
            for &idx in selected {
                *counts.entry(idx).or_default() += 1;
            }
        }

        // Every index should have been selected roughly equally
        let total_selections: usize = counts.values().sum();
        let expected_per_index = total_selections / pool.len();
        let tolerance = (expected_per_index as f64 * 0.25) as usize; // 25% tolerance

        for &count in counts.values() {
            assert!(
                count.abs_diff(expected_per_index) <= tolerance,
                "Selection count {count} for an index outside expected range \
                 ({expected_per_index} ± {tolerance})"
            );
        }
    }

    #[test]
    fn jury_selection_picks_correct_number() {
        let pool: Vec<i32> = (0..50).collect();
        let needed = 10;
        let mut indices: Vec<usize> = (0..pool.len()).collect();
        for i in (pool.len() - needed..pool.len()).rev() {
            let j = rand::thread_rng().gen_range(0..=i);
            indices.swap(i, j);
        }
        let selected: Vec<i32> = indices[pool.len() - needed..]
            .iter()
            .map(|&i| pool[i])
            .collect();

        assert_eq!(
            selected.len(),
            needed,
            "Should select exactly {needed} jurors"
        );
        // All selected should be unique
        let mut unique = selected.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), needed, "All selected jurors should be unique");
    }
}
