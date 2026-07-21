//! Covenant compilation API — compile any DagLock covenant from the indexer.
//!
//! This lets wallets, bots, and integrators compile covenants without
//! running the SilverScript compiler locally. The indexer handles
//! compilation and returns ready-to-deploy bytecode + address.

use axum::http::StatusCode;
use axum::{extract::State, Json};
use kaspa_addresses::{Address, Prefix, Version};
use kaspa_hashes::Hash;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

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
    State(state): State<AppState>,
    Json(body): Json<CompileRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match body.template.as_str() {
        "daglock" => compile_daglock_template(&state, &body.params),
        "daglock_arbiter" => compile_arbiter_template(&state, &body.params),
        "daglock_vault" => compile_vault_template(&state, &body.params),
        "daglock_vault_softlock" => compile_vault_softlock_template(&state, &body.params),
        "daglock_vault_multisig" => compile_vault_multisig_template(&state, &body.params),
        "daglock_krc20" => compile_krc20_template(&state, &body.params),
        "daglock_subscription" => compile_subscription_template(&state, &body.params),
        other => Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "unknown_template",
                format!("Unknown template '{other}'. Options: daglock, daglock_arbiter, daglock_vault, daglock_vault_softlock, daglock_vault_multisig, daglock_krc20, daglock_subscription")
            ))),
        )),
    }
}

fn hex_param(
    params: &std::collections::HashMap<String, String>,
    key: &str,
) -> Result<Vec<u8>, (StatusCode, Json<Value>)> {
    let val = params.get(key).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "missing_param",
                format!("Missing required param '{key}'")
            ))),
        )
    })?;
    hex::decode(val).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_param",
                format!("Param '{key}' must be valid hex")
            ))),
        )
    })
}

fn int_param(
    params: &std::collections::HashMap<String, String>,
    key: &str,
) -> Result<i64, (StatusCode, Json<Value>)> {
    let val = params.get(key).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "missing_param",
                format!("Missing required param '{key}'")
            ))),
        )
    })?;
    val.parse::<i64>().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_param",
                format!("Param '{key}' must be a valid integer")
            ))),
        )
    })
}

fn enforce_treasury(state: &AppState, treasury: &[u8]) -> Result<(), (StatusCode, Json<Value>)> {
    if let Some(ref canonical) = state.treasury_pubkey {
        let provided = hex::encode(treasury);
        if &provided != canonical {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!(ApiError::new(
                    "treasury_mismatch",
                    format!(
                        "Canonical treasury key is {canonical}. The provided key does not match."
                    )
                ))),
            ));
        }
    }
    Ok(())
}

fn optional_or_enforced_treasury(
    params: &std::collections::HashMap<String, String>,
    state: &AppState,
) -> Result<Vec<u8>, (StatusCode, Json<Value>)> {
    match params.get("treasury_key") {
        Some(hex_str) => {
            let key = hex::decode(hex_str).map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!(ApiError::new(
                        "invalid_param",
                        "treasury_key must be valid hex"
                    ))),
                )
            })?;
            enforce_treasury(state, &key)?;
            Ok(key)
        }
        None => {
            if let Some(ref canonical) = state.treasury_pubkey {
                hex::decode(canonical).map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!(ApiError::new(
                            "config_error",
                            "Invalid treasury_pubkey in server config"
                        ))),
                    )
                })
            } else {
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!(ApiError::new(
                        "missing_param",
                        "treasury_key is required when no canonical treasury is configured"
                    ))),
                ))
            }
        }
    }
}

fn compile_result(
    compiled: &daglock_contracts::silverscript_lang::compiler::CompiledContract,
    network: &str,
) -> Json<Value> {
    let (template_prefix, template_suffix, template_hash) =
        daglock_contracts::template_parts_and_hash(compiled);

    // Derive the covenant address (P2SH) from the script
    let script_hash = Sha256::digest(&compiled.script);
    let net_prefix = match network {
        "mainnet" => Prefix::Mainnet,
        n if n.starts_with("testnet-") => Prefix::Testnet,
        n if n.starts_with("simnet-") => Prefix::Simnet,
        n if n.starts_with("devnet-") => Prefix::Devnet,
        _ => Prefix::Testnet, // default to testnet for unknown
    };
    let covenant_address = Address::new(net_prefix, Version::ScriptHash, &script_hash);

    Json(json!({
        "script": hex::encode(&compiled.script),
        "template_hash": hex::encode(&template_hash),
        "template_prefix": hex::encode(&template_prefix),
        "template_suffix": hex::encode(&template_suffix),
        "covenant_address": covenant_address.to_string(),
        "abi": compiled.abi.iter().map(|e| {
            json!({
                "name": e.name,

            })
        }).collect::<Vec<_>>(),
    }))
}

fn compile_daglock_template(
    state: &AppState,
    params: &std::collections::HashMap<String, String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let buyer = hex_param(params, "buyer_key")?;
    let seller = hex_param(params, "seller_key")?;
    let trade_hash = hex_param(params, "trade_hash")?;
    let timeout = int_param(params, "timeout")?;
    let treasury = optional_or_enforced_treasury(params, state)?;

    if buyer.len() != 32 || seller.len() != 32 || trade_hash.len() != 32 || treasury.len() != 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_param",
                "Keys and hashes must be 32 bytes (64 hex chars)"
            ))),
        ));
    }

    let compiled =
        daglock_contracts::compile_daglock(&buyer, &seller, &trade_hash, timeout, &treasury);
    Ok(compile_result(&compiled, &state.network))
}

fn compile_arbiter_template(
    state: &AppState,
    params: &std::collections::HashMap<String, String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let buyer = hex_param(params, "buyer_key")?;
    let seller = hex_param(params, "seller_key")?;
    let trade_hash = hex_param(params, "trade_hash")?;
    let timeout = int_param(params, "timeout")?;
    let treasury = optional_or_enforced_treasury(params, state)?;
    let arbiter = hex_param(params, "arbiter_key")?;

    // Reject zero-key arbiter — without an arbiter, refundAfterTimeout is unreachable
    if arbiter.iter().all(|&b| b == 0) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new("invalid_param", "Arbiter key cannot be all-zeroes. Use daglock.sil template if you don't want a mediator."))),
        ));
    }

    if buyer.len() != 32
        || seller.len() != 32
        || trade_hash.len() != 32
        || treasury.len() != 32
        || arbiter.len() != 32
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_param",
                "All keys must be 32 bytes (64 hex chars)"
            ))),
        ));
    }

    let compiled = daglock_contracts::compile_daglock_arbiter(
        &buyer,
        &seller,
        &trade_hash,
        timeout,
        &treasury,
        &arbiter,
    );
    Ok(compile_result(&compiled, &state.network))
}

fn compile_vault_template(
    state: &AppState,
    params: &std::collections::HashMap<String, String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let owner = hex_param(params, "owner_key")?;
    let lock_duration = int_param(params, "lock_duration")?;
    let treasury = optional_or_enforced_treasury(params, state)?;
    let heir = params
        .get("heir_key")
        .map(|v| hex::decode(v).unwrap_or(vec![0u8; 32]))
        .unwrap_or_else(|| vec![0u8; 32]);
    let inherit_lock_duration = params
        .get("inherit_lock_duration")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    if owner.len() != 32 || treasury.len() != 32 || heir.len() != 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_param",
                "Keys must be 32 bytes (64 hex chars)"
            ))),
        ));
    }
    let compiled = daglock_contracts::compile_daglock_vault(
        &owner,
        lock_duration,
        &treasury,
        &heir,
        inherit_lock_duration,
    );
    Ok(compile_result(&compiled, &state.network))
}

fn compile_vault_softlock_template(
    state: &AppState,
    params: &std::collections::HashMap<String, String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let owner = hex_param(params, "owner_key")?;
    let beneficiary = hex_param(params, "beneficiary_key")?;
    let password_hash = hex_param(params, "password_hash")?;
    let lock_duration = int_param(params, "lock_duration")?;
    let treasury = optional_or_enforced_treasury(params, state)?;
    if owner.len() != 32
        || beneficiary.len() != 32
        || password_hash.len() != 32
        || treasury.len() != 32
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_param",
                "Keys and password_hash must be 32 bytes (64 hex chars)"
            ))),
        ));
    }
    let compiled = daglock_contracts::compile_daglock_vault_softlock(
        &owner,
        &beneficiary,
        &password_hash,
        lock_duration,
        &treasury,
    );
    Ok(compile_result(&compiled, &state.network))
}

fn compile_vault_multisig_template(
    state: &AppState,
    params: &std::collections::HashMap<String, String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let key1 = hex_param(params, "key1")?;
    let treasury = optional_or_enforced_treasury(params, state)?;
    let lock_duration = int_param(params, "lock_duration")?;
    let key2 = match params.get("key2") {
        Some(h) => hex::decode(h).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!(ApiError::new(
                    "invalid_param",
                    "key2 must be valid hex"
                ))),
            )
        })?,
        None => vec![0u8; 32],
    };
    let key3 = match params.get("key3") {
        Some(h) => hex::decode(h).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!(ApiError::new(
                    "invalid_param",
                    "key3 must be valid hex"
                ))),
            )
        })?,
        None => vec![0u8; 32],
    };
    if key1.len() != 32 || treasury.len() != 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_param",
                "key1 and treasury_key must be 32 bytes"
            ))),
        ));
    }
    let compiled = daglock_contracts::compile_daglock_vault_multisig(
        &key1,
        &key2,
        &key3,
        lock_duration,
        &treasury,
    );
    Ok(compile_result(&compiled, &state.network))
}

fn compile_krc20_template(
    state: &AppState,
    params: &std::collections::HashMap<String, String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let buyer = hex_param(params, "buyer_key")?;
    let seller = hex_param(params, "seller_key")?;
    let trade_hash = params
        .get("trade_hash")
        .map(|v| hex::decode(v).unwrap_or(vec![0u8; 32]))
        .unwrap_or_else(|| vec![0u8; 32]);
    let timeout = int_param(params, "timeout")?;
    let treasury = optional_or_enforced_treasury(params, state)?;

    // KCC-20 template metadata — required for ICC validation
    // If not provided, compile in dev mode (kcc20TemplatePrefixLen = 0)
    let kcc20_prefix_len = params
        .get("kcc20_template_prefix_len")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let kcc20_suffix_len = params
        .get("kcc20_template_suffix_len")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let kcc20_expected_hash = params
        .get("kcc20_expected_template_hash")
        .map(|v| hex::decode(v).unwrap_or(vec![0u8; 32]))
        .unwrap_or_else(|| vec![0u8; 32]);
    let kcc20_prefix = params
        .get("kcc20_template_prefix")
        .map(|v| hex::decode(v).unwrap_or_default())
        .unwrap_or_default();
    let kcc20_suffix = params
        .get("kcc20_template_suffix")
        .map(|v| hex::decode(v).unwrap_or_default())
        .unwrap_or_default();
    let kcc20_covenant_id = params
        .get("kcc20_covenant_id")
        .map(|v| hex::decode(v).unwrap_or(vec![0u8; 32]))
        .unwrap_or_else(|| vec![0u8; 32]);

    if buyer.len() != 32 || seller.len() != 32 || treasury.len() != 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_param",
                "buyer_key, seller_key, and treasury_key must be 32 bytes (64 hex chars)"
            ))),
        ));
    }
    if trade_hash.len() != 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_param",
                "trade_hash must be 32 bytes (64 hex chars)"
            ))),
        ));
    }
    if kcc20_expected_hash.len() != 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_param",
                "kcc20_expected_template_hash must be 32 bytes (64 hex chars)"
            ))),
        ));
    }
    if kcc20_covenant_id.len() != 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_param",
                "kcc20_covenant_id must be 32 bytes (64 hex chars)"
            ))),
        ));
    }

    let compiled = daglock_contracts::compile_daglock_krc20(
        &buyer,
        &seller,
        &trade_hash,
        timeout,
        &treasury,
        kcc20_prefix_len,
        kcc20_suffix_len,
        &kcc20_expected_hash,
        &kcc20_prefix,
        &kcc20_suffix,
        &kcc20_covenant_id,
    );
    Ok(compile_result(&compiled, &state.network))
}

fn compile_subscription_template(
    state: &AppState,
    params: &std::collections::HashMap<String, String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let payer_key = hex_param(params, "payer_key")?;
    let recipient_key = hex_param(params, "recipient_key")?;
    let total_amount = int_param(params, "total_amount")?;
    let installment_amount = int_param(params, "installment_amount")?;
    let interval_seconds = int_param(params, "interval_seconds")?;
    let start_time = int_param(params, "start_time")?;
    let current_period = int_param(params, "current_period").unwrap_or(0);
    let treasury = optional_or_enforced_treasury(params, state)?;

    if payer_key.len() != 32 || recipient_key.len() != 32 || treasury.len() != 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_param",
                "payer_key, recipient_key, and treasury_key must be 32 bytes (64 hex chars)"
            ))),
        ));
    }
    if total_amount <= 0 || installment_amount <= 0 || installment_amount > total_amount {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_param",
                "total_amount and installment_amount must be positive, installment <= total"
            ))),
        ));
    }

    let compiled = daglock_contracts::compile_daglock_subscription(
        &payer_key,
        &recipient_key,
        total_amount,
        installment_amount,
        interval_seconds,
        start_time,
        current_period,
        &treasury,
    );
    Ok(compile_result(&compiled, &state.network))
}
