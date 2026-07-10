//! KRC-20 token API endpoints.
//! Aggregates token data from existing offers and escrows.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;

#[derive(Deserialize)]
pub struct TokenListQuery {
    #[allow(dead_code)]
    pub sort: Option<String>,
}

#[derive(Deserialize)]
pub struct ChartQuery {
    pub period: Option<String>,
}

#[derive(Deserialize)]
pub struct DeployTokenRequest {
    pub name: String,
    pub ticker: String,
    pub total_supply: i64,
    pub decimals: Option<i32>,
    pub mint_mode: Option<String>,
    pub owner_address: String,
}

#[derive(Deserialize)]
pub struct UpdateTokenRequest {
    pub covenant_address: Option<String>,
    pub deploy_tx_id: Option<String>,
    pub status: Option<String>,
}

/// GET /v1/tokens
pub async fn list(
    State(state): State<AppState>,
) -> Json<Value> {
    match queries::tokens::list_tokens(&state.db).await {
        Ok(tokens) => Json(json!({
            "tokens": tokens,
            "total": tokens.len(),
        })),
        Err(e) => Json(json!({
            "error": "db_error",
            "message": format!("{e}"),
            "tokens": [],
            "total": 0,
        })),
    }
}

/// GET /v1/tokens/:ticker
pub async fn get(
    State(state): State<AppState>,
    Path(ticker): Path<String>,
) -> Json<Value> {
    let upper = ticker.to_uppercase();
    // First check the registry, then fall back to aggregated data
    match queries::tokens::get_token(&state.db, &upper).await {
        Ok(Some(mut detail)) => {
            // Enrich with registry data if available
            if let Ok(Some(reg)) = queries::tokens::get_registered_token(&state.db, &upper).await {
                detail.summary.ticker = format!("{} (registered)", reg.ticker);
            }
            Json(json!(detail))
        }
        Ok(None) => {
            // Check if it exists in registry but hasn't been traded yet
            if let Ok(Some(reg)) = queries::tokens::get_registered_token(&state.db, &upper).await {
                return Json(json!({
                    "ticker": reg.ticker,
                    "name": reg.name,
                    "total_supply": reg.total_supply,
                    "decimals": reg.decimals,
                    "mint_mode": reg.mint_mode,
                    "status": reg.status,
                    "owner_address": reg.owner_address,
                    "covenant_address": reg.covenant_address,
                    "deploy_tx_id": reg.deploy_tx_id,
                    "price_kas": null,
                    "volume_24h_sompi": 0,
                    "trades_24h": 0,
                    "total_trades": 0,
                    "active_offers": 0,
                    "trades": [],
                }));
            }
            Json(json!({
                "error": "not_found",
                "message": format!("Token '{upper}' not found."),
            }))
        }
        Err(e) => Json(json!({
            "error": "db_error",
            "message": format!("{e}"),
        })),
    }
}

/// GET /v1/tokens/:ticker/chart?period=7d|30d|all
pub async fn chart(
    State(state): State<AppState>,
    Path(ticker): Path<String>,
    Query(query): Query<ChartQuery>,
) -> Json<Value> {
    let period_seconds = match query.period.as_deref() {
        Some("30d") => 2_592_000,
        Some("all") => 31_536_000,
        _ => 604_800,
    };

    let upper = ticker.to_uppercase();
    match queries::tokens::get_token_chart(&state.db, &upper, period_seconds).await {
        Ok(points) => Json(json!({
            "ticker": upper,
            "period": query.period.unwrap_or_else(|| "7d".to_string()),
            "points": points,
        })),
        Err(e) => Json(json!({
            "error": "db_error",
            "message": format!("{e}"),
            "points": [],
        })),
    }
}

/// POST /v1/tokens/deploy
/// Register a new KRC-20 token in the indexer. The deployer then broadcasts
/// the covenant transaction separately.
pub async fn deploy(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<DeployTokenRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": format!("{}", e)})),
        )
    })?;

    if auth.address != body.owner_address {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "address_mismatch", "message": "Owner address must match the signed address"})),
        ));
    }

    let ticker = body.ticker.trim().to_uppercase();
    let name = body.name.trim().to_string();
    let mint_mode = body.mint_mode.unwrap_or_else(|| "fixed".to_string());

    // Validation
    if ticker.len() < 3 || ticker.len() > 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_ticker", "message": "Ticker must be 3-8 characters"})),
        ));
    }
    if !ticker.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_ticker", "message": "Ticker must be alphanumeric"})),
        ));
    }
    if name.len() < 2 || name.len() > 64 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_name", "message": "Name must be 2-64 characters"})),
        ));
    }
    if body.total_supply <= 0 || body.total_supply > 1_000_000_000_000 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_supply", "message": "Supply must be between 1 and 1,000,000,000,000"})),
        ));
    }
    let decimals = body.decimals.unwrap_or(8);
    if decimals < 0 || decimals > 18 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_decimals", "message": "Decimals must be 0-18"})),
        ));
    }
    if !["fixed", "mintable", "burnable"].contains(&mint_mode.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_mint_mode", "message": "Mint mode must be 'fixed', 'mintable', or 'burnable'"})),
        ));
    }

    // Check for duplicate ticker
    if let Ok(Some(_)) = queries::tokens::get_registered_token(&state.db, &ticker).await {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "ticker_taken", "message": format!("'{ticker}' is already registered")})),
        ));
    }

    let id = format!("tok_{}", Uuid::new_v4().to_string().replace('-', ""));
    queries::tokens::register_token(
        &state.db,
        &id,
        &ticker,
        &name,
        body.total_supply,
        decimals,
        &mint_mode,
        &body.owner_address,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "db_error", "message": format!("{e}")})),
        )
    })?;

    Ok(Json(json!({
        "status": "token_registered",
        "id": id,
        "ticker": ticker,
        "name": name,
        "total_supply": body.total_supply,
        "decimals": decimals,
        "mint_mode": mint_mode,
        "message": "Token registered! To deploy on-chain, compile and broadcast a KRC-20 covenant. After broadcasting, PATCH /v1/tokens/:ticker with the covenant address and TX ID.",
    })))
}

/// PATCH /v1/tokens/:ticker
/// Update token deployment status (after user broadcasts the covenant).
pub async fn update(
    State(state): State<AppState>,
    Path(ticker): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UpdateTokenRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized", "message": format!("{}", e)})),
        )
    })?;

    let upper = ticker.to_uppercase();
    let reg = queries::tokens::get_registered_token(&state.db, &upper)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "not_found", "message": format!("Token '{upper}' not registered")})),
            )
        })?;

    if reg.owner_address.as_deref() != Some(&auth.address) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "message": "Only the token owner can update"})),
        ));
    }

    let new_status = body.status.as_deref().unwrap_or("active");

    // # ponytail: Token deployment verification is skipped — we don't verify that
    // deploy_tx_id actually contains a DagLockKRC20 covenant on-chain.
    // Upgrade path: query the tx via wRPC, extract output scripts, verify
    // the KRC-20 covenant template hash matches. Requires wRPC connection.

    queries::tokens::update_token_status(
        &state.db,
        &upper,
        new_status,
        body.covenant_address.as_deref(),
        body.deploy_tx_id.as_deref(),
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "db_error", "message": format!("{e}")})),
        )
    })?;

    Ok(Json(json!({
        "status": "updated",
        "ticker": upper,
        "new_status": new_status,
    })))
}

/// GET /v1/tokens/registered — list registered tokens from the registry
pub async fn registered_list(
    State(state): State<AppState>,
    Query(_query): Query<TokenListQuery>,
) -> Json<Value> {
    match queries::tokens::list_registered_tokens(&state.db, None).await {
        Ok(tokens) => Json(json!({
            "tokens": tokens,
            "total": tokens.len(),
        })),
        Err(e) => Json(json!({
            "error": "db_error",
            "message": format!("{e}"),
            "tokens": [],
            "total": 0,
        })),
    }
}
