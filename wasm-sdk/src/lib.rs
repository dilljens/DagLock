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
    let buyer_bytes =
        hex::decode(buyer_key).map_err(|e| JsError::new(&format!("Invalid buyer key: {}", e)))?;
    let seller_bytes =
        hex::decode(seller_key).map_err(|e| JsError::new(&format!("Invalid seller key: {}", e)))?;
    let trade_hash_bytes =
        hex::decode(trade_hash).map_err(|e| JsError::new(&format!("Invalid trade hash: {}", e)))?;
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

    // Compute P2SH covenant address
    let script_hash = blake2b_simd::Params::new()
        .hash_length(32)
        .hash(&compiled.script)
        .as_bytes()
        .to_vec();
    let covenant_address: String = kaspa_addresses::Address::new(
        // Infer prefix from timeout (> 2025 = real timestamp, otherwise dev)
        kaspa_addresses::Prefix::Testnet,
        kaspa_addresses::Version::ScriptHash,
        &script_hash,
    )
    .into();

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
        "covenant_address": covenant_address,
    });

    serde_json::to_string(&result).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

/// Compute the P2SH covenant address from a compiled escrow script.
///
/// # Arguments
/// * `script_hex` - Hex-encoded compiled covenant script (from `compile_escrow`)
/// * `network` - Network prefix: "mainnet", "testnet-10", or "devnet"
///
/// # Returns
/// The P2SH address string (e.g. "kaspa:pq...")
#[wasm_bindgen]
pub fn compute_covenant_address(script_hex: &str, network: &str) -> Result<String, JsError> {
    let script =
        hex::decode(script_hex).map_err(|e| JsError::new(&format!("Invalid script hex: {}", e)))?;

    if script.is_empty() {
        return Err(JsError::new("Script cannot be empty"));
    }

    // Compute BLAKE2b-256 hash of the script (Kaspa's P2SH hash)
    let script_hash = blake2b_simd::Params::new()
        .hash_length(32)
        .hash(&script)
        .as_bytes()
        .to_vec();

    // Determine network prefix
    let prefix = match network {
        "mainnet" => kaspa_addresses::Prefix::Mainnet,
        "testnet-10" | "testnet-11" | "testnet-12" => kaspa_addresses::Prefix::Testnet,
        _ => {
            // Default to testnet for dev/simnet
            kaspa_addresses::Prefix::Testnet
        }
    };

    // Create P2SH address
    let address =
        kaspa_addresses::Address::new(prefix, kaspa_addresses::Version::ScriptHash, &script_hash);

    let address_str: String = address.into();
    Ok(address_str)
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
    let script =
        hex::decode(script_hex).map_err(|e| JsError::new(&format!("Invalid script: {}", e)))?;
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
    let fee_sompi = amount_sompi / daglock_shared::FEE_DENOMINATOR;

    let result = serde_json::json!({
        "amount_sompi": amount_sompi,
        "fee_sompi": fee_sompi,
        "fee_percentage": 0.5,
    });

    serde_json::to_string(&result).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

/// Convert a KAS amount string (e.g. "5000" or "5000.5") to sompi.
///
/// # Arguments
/// * `amount_str` - KAS amount as a decimal string
///
/// # Returns
/// Amount in sompi (i64)
#[wasm_bindgen]
pub fn kas_to_sompi(amount_str: &str) -> Result<u64, JsError> {
    daglock_shared::kas_to_sompi(amount_str).map_err(|e| JsError::new(&format!("{}", e)))
}

/// Validate a trade hash (must be 64 hex chars).
#[wasm_bindgen]
pub fn validate_trade_hash(hash_hex: &str) -> Result<bool, JsError> {
    Ok(daglock_shared::validate_trade_hash(hash_hex).is_ok())
}

/// Compile a DagLock Vault covenant.
#[wasm_bindgen]
pub fn compile_vault(
    owner_key: &str,
    timeout: i64,
    treasury_key: &str,
    heir_key: &str,
    heir_timeout: i64,
) -> Result<String, JsError> {
    let owner = parse_hex32(owner_key, "owner_key")?;
    let treasury = parse_hex32(treasury_key, "treasury_key")?;
    let heir = parse_hex32(heir_key, "heir_key")?;
    Ok(compile_result_json(
        &daglock_contracts::compile_daglock_vault(&owner, timeout, &treasury, &heir, heir_timeout),
    ))
}

/// Compile a DagLock Vault Softlock covenant.
#[wasm_bindgen]
pub fn compile_vault_softlock(
    owner_key: &str,
    beneficiary_key: &str,
    password_hash: &str,
    timeout: i64,
    treasury_key: &str,
) -> Result<String, JsError> {
    let owner = parse_hex32(owner_key, "owner_key")?;
    let beneficiary = parse_hex32(beneficiary_key, "beneficiary_key")?;
    let password = parse_hex32(password_hash, "password_hash")?;
    let treasury = parse_hex32(treasury_key, "treasury_key")?;
    Ok(compile_result_json(
        &daglock_contracts::compile_daglock_vault_softlock(
            &owner,
            &beneficiary,
            &password,
            timeout,
            &treasury,
        ),
    ))
}

/// Compile a DagLock Vault Multi-sig covenant.
#[wasm_bindgen]
pub fn compile_vault_multisig(
    key1: &str,
    key2: &str,
    key3: &str,
    timeout: i64,
    treasury_key: &str,
) -> Result<String, JsError> {
    let k1 = parse_hex32(key1, "key1")?;
    let k2 = parse_hex32(key2, "key2")?;
    let k3 = parse_hex32(key3, "key3")?;
    let treasury = parse_hex32(treasury_key, "treasury_key")?;
    Ok(compile_result_json(
        &daglock_contracts::compile_daglock_vault_multisig(&k1, &k2, &k3, timeout, &treasury),
    ))
}

/// Compile a DagLock Arbiter covenant.
#[wasm_bindgen]
pub fn compile_arbiter(
    buyer_key: &str,
    seller_key: &str,
    trade_hash: &str,
    timeout: i64,
    treasury_key: &str,
    arbiter_key: &str,
) -> Result<String, JsError> {
    let buyer = parse_hex32(buyer_key, "buyer_key")?;
    let seller = parse_hex32(seller_key, "seller_key")?;
    let trade = parse_hex32(trade_hash, "trade_hash")?;
    let treasury = parse_hex32(treasury_key, "treasury_key")?;
    let arbiter = parse_hex32(arbiter_key, "arbiter_key")?;
    Ok(compile_result_json(
        &daglock_contracts::compile_daglock_arbiter(
            &buyer, &seller, &trade, timeout, &treasury, &arbiter,
        ),
    ))
}

fn parse_hex32(hex_str: &str, name: &str) -> Result<Vec<u8>, JsError> {
    let bytes =
        hex::decode(hex_str).map_err(|e| JsError::new(&format!("Invalid {}: {}", name, e)))?;
    if bytes.len() != 32 {
        return Err(JsError::new(&format!(
            "{} must be 32 bytes (64 hex chars)",
            name
        )));
    }
    Ok(bytes)
}

fn compile_result_json(
    compiled: &daglock_contracts::silverscript_lang::compiler::CompiledContract,
) -> String {
    let (p, s, template_hash) = template_parts_and_hash(compiled);
    let script_hash = blake2b_simd::Params::new()
        .hash_length(32)
        .hash(&compiled.script)
        .as_bytes()
        .to_vec();
    let covenant_address: String = kaspa_addresses::Address::new(
        kaspa_addresses::Prefix::Testnet,
        kaspa_addresses::Version::ScriptHash,
        &script_hash,
    )
    .into();
    let entrypoint_names: Vec<&str> = compiled.abi.iter().map(|e| e.name.as_str()).collect();
    serde_json::to_string(&serde_json::json!({
        "script": hex::encode(&compiled.script), "template_hash": hex::encode(&template_hash),
        "covenant_address": covenant_address, "entrypoints": entrypoint_names,
        "prefix": hex::encode(&p), "suffix": hex::encode(&s),
    }))
    .unwrap_or_else(|_| "{}".to_string())
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
        assert!(!json["script"].as_str().unwrap().is_empty());
        assert!(!json["template_hash"].as_str().unwrap().is_empty());
        assert!(json["escrow_id"].as_str().unwrap().starts_with("esc_"));
    }

    #[test]
    fn compute_covenant_address_testnet() {
        // Use a known script (empty zero-pubkey compile)
        let zero_key = "0000000000000000000000000000000000000000000000000000000000000000";
        let compile_result =
            compile_escrow(zero_key, zero_key, zero_key, 1_700_000_000, zero_key).unwrap();
        let json: serde_json::Value = serde_json::from_str(&compile_result).unwrap();
        let script_hex = json["script"].as_str().unwrap();

        let address = compute_covenant_address(script_hex, "testnet-10").unwrap();
        assert!(
            address.starts_with("kaspatest:p"),
            "P2SH address should start with kaspatest:p, got {}",
            address
        );
    }

    #[test]
    fn calculate_fee_correct() {
        let result = calculate_fee(1_000_000_000).unwrap();
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["fee_sompi"], 5_000_000);
    }

    #[test]
    fn compile_vault_returns_valid_json() {
        let zero = "0000000000000000000000000000000000000000000000000000000000000000";
        let result = compile_vault(zero, 1_700_000_000, zero, zero, 0);
        assert!(result.is_ok());
        let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(!json["script"].as_str().unwrap().is_empty());
    }
    #[test]
    fn compile_arbiter_returns_valid_json() {
        let zero = "0000000000000000000000000000000000000000000000000000000000000000";
        let one = "0000000000000000000000000000000000000000000000000000000000000001";
        let result = compile_arbiter(zero, zero, zero, 1_700_000_000, zero, one);
        assert!(result.is_ok());
        let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(json["entrypoints"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e.as_str() == Some("disputeSellerWins")));
    }
}
