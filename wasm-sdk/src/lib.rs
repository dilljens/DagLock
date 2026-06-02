//! DagLock WASM SDK — Browser-side transaction assembly.
//!
//! Provides JavaScript-accessible functions for compiling DagLock covenants
//! and assembling unsigned transactions. All cryptographic operations
//! happen in Rust/WASM — no private keys are ever exposed.

use wasm_bindgen::prelude::*;

// Re-export contracts for compilation
use daglock_contracts::{compile_daglock, template_parts_and_hash};

/// Compile a DagLock KAS covenant and return the result as a JSON string.
///
/// # Arguments
/// * `buyer_key` - 32-byte hex-encoded buyer public key
/// * `seller_key` - 32-byte hex-encoded seller public key
/// * `trade_hash` - 32-byte hex-encoded SHA-256 hash (or zeros for no atomic swap)
/// * `timeout` - Unix timestamp after which refund is allowed
/// * `treasury_key` - 32-byte hex-encoded treasury public key
///
/// # Returns
/// JSON string with: `{ script, template_hash, escrow_id, amount_sompi, fee_sompi }`
#[wasm_bindgen]
pub fn compile_escrow(
    buyer_key: &str,
    seller_key: &str,
    trade_hash: &str,
    timeout: i64,
    treasury_key: &str,
) -> Result<String, JsError> {
    // Parse hex strings to bytes
    let buyer_bytes = hex::decode(buyer_key)
        .map_err(|e| JsError::new(&format!("Invalid buyer key: {}", e)))?;
    let seller_bytes = hex::decode(seller_key)
        .map_err(|e| JsError::new(&format!("Invalid seller key: {}", e)))?;
    let trade_hash_bytes = hex::decode(trade_hash)
        .map_err(|e| JsError::new(&format!("Invalid trade hash: {}", e)))?;
    let treasury_bytes = hex::decode(treasury_key)
        .map_err(|e| JsError::new(&format!("Invalid treasury key: {}", e)))?;

    // Validate lengths
    if buyer_bytes.len() != 32 {
        return Err(JsError::new("Buyer key must be 32 bytes"));
    }
    if seller_bytes.len() != 32 {
        return Err(JsError::new("Seller key must be 32 bytes"));
    }
    if trade_hash_bytes.len() != 32 {
        return Err(JsError::new("Trade hash must be 32 bytes"));
    }
    if treasury_bytes.len() != 32 {
        return Err(JsError::new("Treasury key must be 32 bytes"));
    }

    // Compile covenant
    let compiled = compile_daglock(
        &buyer_bytes,
        &seller_bytes,
        &trade_hash_bytes,
        timeout,
        &treasury_bytes,
    );

    // Extract template hash
    let (prefix, suffix, template_hash) = template_parts_and_hash(&compiled);

    // Generate escrow ID from script hash
    let escrow_id = blake2b_simd::Params::new()
        .hash_length(8)
        .hash(&compiled.script)
        .to_hex();

    // Calculate fee (0.5% = 1/200)
    // Note: amount must be set by the caller before broadcast
    let result = serde_json::json!({
        "script": hex::encode(&compiled.script),
        "template_hash": hex::encode(&template_hash),
        "escrow_id": format!("esc_{}", escrow_id),
        "entrypoints": ["release", "swap", "refund"],
        "prefix": hex::encode(&prefix),
        "suffix": hex::encode(&suffix),
    });

    serde_json::to_string(&result)
        .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

/// Verify that a script matches a DagLock template hash.
///
/// # Arguments
/// * `script_hex` - Hex-encoded script bytes
/// * `template_hash_hex` - Hex-encoded 20-byte template hash
///
/// # Returns
/// true if the script matches the template
#[wasm_bindgen]
pub fn verify_template_match(script_hex: &str, template_hash_hex: &str) -> Result<bool, JsError> {
    let script = hex::decode(script_hex)
        .map_err(|e| JsError::new(&format!("Invalid script: {}", e)))?;
    let template_hash = hex::decode(template_hash_hex)
        .map_err(|e| JsError::new(&format!("Invalid template hash: {}", e)))?;

    if template_hash.len() != 20 {
        return Err(JsError::new("Template hash must be 20 bytes"));
    }

    // Compute script hash
    let computed_hash = blake2b_simd::Params::new()
        .hash_length(20)
        .hash(&script)
        .as_bytes()
        .to_vec();

    Ok(computed_hash == template_hash)
}

/// Get the DagLock protocol fee for a given amount.
///
/// # Arguments
/// * `amount_sompi` - Amount in sompi (smallest unit)
///
/// # Returns
/// JSON string with: `{ amount_sompi, fee_sompi, fee_percentage }`
#[wasm_bindgen]
pub fn calculate_fee(amount_sompi: i64) -> Result<String, JsError> {
    let fee_sompi = amount_sompi / 200;

    let result = serde_json::json!({
        "amount_sompi": amount_sompi,
        "fee_sompi": fee_sompi,
        "fee_percentage": 0.5,
    });

    serde_json::to_string(&result)
        .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_escrow_returns_valid_json() {
        let zero_key = "0000000000000000000000000000000000000000000000000000000000000000";
        let result = compile_escrow(zero_key, zero_key, zero_key, 1_700_000_000, zero_key);
        assert!(result.is_ok());

        let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(json["script"].as_str().unwrap().len() > 0);
        assert!(json["template_hash"].as_str().unwrap().len() > 0);
        assert!(json["escrow_id"].as_str().unwrap().starts_with("esc_"));
    }

    #[test]
    fn calculate_fee_correct() {
        let result = calculate_fee(1_000_000_000).unwrap();
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["fee_sompi"], 5_000_000);
    }
}
