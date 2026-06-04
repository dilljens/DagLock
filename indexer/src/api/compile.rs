//! Covenant compilation API — compile any DagLock covenant from the indexer.
//!
//! This lets wallets, bots, and integrators compile covenants without
//! running the SilverScript compiler locally. The indexer handles
//! compilation and returns ready-to-deploy bytecode + address.

use axum::http::StatusCode;
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use daglock_contracts::silverscript_lang::ast::Expr;

use crate::api::AppState;
use crate::types::*;

#[derive(Deserialize)]
pub struct CompileRequest {
    /// Covenant template name: "daglock", "daglock_arbiter", "daglock_krc20", "daglock_vault"
    pub template: String,
    /// Constructor parameters as key-value pairs
    pub params: std::collections::HashMap<String, String>,
}

/// POST /v1/compile
/// Compile a covenant template with the given constructor parameters.
pub async fn compile(
    State(_state): State<AppState>,
    Json(body): Json<CompileRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match body.template.as_str() {
        "daglock" => compile_daglock_template(&body.params),
        "daglock_arbiter" => compile_arbiter_template(&body.params),
        "daglock_vault" => compile_vault_template(&body.params),
        "daglock_krc20" => compile_krc20_template(&body.params),
        other => Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "unknown_template",
                format!("Unknown template '{other}'. Options: daglock, daglock_arbiter, daglock_vault, daglock_krc20")
            ))),
        )),
    }
}

fn hex_param(params: &std::collections::HashMap<String, String>, key: &str) -> Result<Vec<u8>, (StatusCode, Json<Value>)> {
    let val = params.get(key).ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(json!(ApiError::new("missing_param", format!("Missing required param '{key}'")))))
    })?;
    hex::decode(val).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(json!(ApiError::new("invalid_param", format!("Param '{key}' must be valid hex")))))
    })
}

fn int_param(params: &std::collections::HashMap<String, String>, key: &str) -> Result<i64, (StatusCode, Json<Value>)> {
    let val = params.get(key).ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(json!(ApiError::new("missing_param", format!("Missing required param '{key}'")))))
    })?;
    val.parse::<i64>().map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(json!(ApiError::new("invalid_param", format!("Param '{key}' must be a valid integer")))))
    })
}

fn compile_result(compiled: &daglock_contracts::silverscript_lang::compiler::CompiledContract) -> Json<Value> {
    let (prefix, suffix, template_hash) = daglock_contracts::template_parts_and_hash(compiled);
    Json(json!({
        "script": hex::encode(&compiled.script),
        "template_hash": hex::encode(&template_hash),
        "template_prefix": hex::encode(&prefix),
        "template_suffix": hex::encode(&suffix),
        "abi": compiled.abi.iter().map(|e| {
            json!({
                "name": e.name,

            })
        }).collect::<Vec<_>>(),
    }))
}

fn compile_daglock_template(params: &std::collections::HashMap<String, String>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let buyer = hex_param(params, "buyer_key")?;
    let seller = hex_param(params, "seller_key")?;
    let trade_hash = hex_param(params, "trade_hash")?;
    let timeout = int_param(params, "timeout")?;
    let treasury = hex_param(params, "treasury_key")?;

    if buyer.len() != 32 || seller.len() != 32 || trade_hash.len() != 32 || treasury.len() != 32 {
        return Err((StatusCode::BAD_REQUEST, Json(json!(ApiError::new("invalid_param", "Keys and hashes must be 32 bytes (64 hex chars)")))));
    }

    let compiled = daglock_contracts::compile_daglock(&buyer, &seller, &trade_hash, timeout, &treasury);
    Ok(compile_result(&compiled))
}

fn compile_arbiter_template(params: &std::collections::HashMap<String, String>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let buyer = hex_param(params, "buyer_key")?;
    let seller = hex_param(params, "seller_key")?;
    let trade_hash = hex_param(params, "trade_hash")?;
    let timeout = int_param(params, "timeout")?;
    let treasury = hex_param(params, "treasury_key")?;
    let arbiter = hex_param(params, "arbiter_key")?;

    if buyer.len() != 32 || seller.len() != 32 || trade_hash.len() != 32 || treasury.len() != 32 || arbiter.len() != 32 {
        return Err((StatusCode::BAD_REQUEST, Json(json!(ApiError::new("invalid_param", "All keys must be 32 bytes (64 hex chars)")))));
    }

    let compiled = daglock_contracts::compile_daglock_arbiter(&buyer, &seller, &trade_hash, timeout, &treasury, &arbiter);
    Ok(compile_result(&compiled))
}

fn compile_vault_template(params: &std::collections::HashMap<String, String>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let owner = hex_param(params, "owner_key")?;
    let timeout = int_param(params, "timeout")?;

    if owner.len() != 32 {
        return Err((StatusCode::BAD_REQUEST, Json(json!(ApiError::new("invalid_param", "Owner key must be 32 bytes (64 hex chars)")))));
    }

    let compiled = daglock_contracts::compile_daglock_vault(&owner, timeout);
    Ok(compile_result(&compiled))
}

fn compile_krc20_template(_params: &std::collections::HashMap<String, String>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Err((StatusCode::NOT_IMPLEMENTED, Json(json!(ApiError::new("not_implemented", "KRC-20 compilation requires KCC-20 template metadata. Not yet available via REST API.")))))
}
