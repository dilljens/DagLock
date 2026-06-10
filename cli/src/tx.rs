//! Transaction assembly helpers for the DagLock CLI.
//!
//! Uses `daglock-contracts` to compile covenants and build
//! unsigned transactions. Outputs hex-encoded unsigned tx data.

use daglock_contracts::compile_daglock;

pub struct CreateEscrowResult {
    pub escrow_id: String,
    pub unsigned_tx_hex: String,
    pub _template_hash: Vec<u8>,
    pub amount_sompi: i64,
    pub fee_sompi: i64,
    pub covenant_address: String,
}

/// Parse a KAS amount string (e.g. "5000" or "5000.5") to sompi (i64).
/// Uses integer arithmetic to avoid floating-point precision loss.
pub fn kas_to_sompi(amount_str: &str) -> anyhow::Result<i64> {
    let parts: Vec<&str> = amount_str.split('.').collect();
    let whole = parts[0].parse::<i64>()?;
    let fractional = if parts.len() > 1 {
        let frac_str = parts[1];
        // Pad or truncate to 8 decimal places (sompi precision)
        let padded = format!("{:0<8}", frac_str);
        let truncated = &padded[..8.min(padded.len())];
        truncated.parse::<i64>()?
    } else {
        0
    };
    Ok(whole * 100_000_000 + fractional)
}

pub fn assemble_create_escrow(
    buyer_key: &[u8; 32],
    seller_key: &[u8; 32],
    amount_sompi: i64,
    timeout_secs: u64,
    treasury_key: &[u8; 32],
) -> anyhow::Result<CreateEscrowResult> {
    let fee_sompi = amount_sompi / daglock_shared::FEE_DENOMINATOR as i64;
    let zero_hash = [0u8; 32];
    let now = chrono::Utc::now().timestamp();
    let expiration = now + timeout_secs as i64;

    let compiled = compile_daglock(buyer_key, seller_key, &zero_hash, expiration, treasury_key);

    let hash = blake2b_simd::Params::new()
        .hash_length(8)
        .to_state()
        .update(&compiled.script)
        .update(&now.to_le_bytes())
        .finalize();
    let escrow_id = format!("esc_{}", hex::encode(hash.as_bytes()));

    let (_, _, tpl_hash) = daglock_contracts::template_parts_and_hash(&compiled);

    // Compute P2SH covenant address from compiled script
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

    Ok(CreateEscrowResult {
        escrow_id,
        unsigned_tx_hex: hex::encode(&compiled.script),
        _template_hash: tpl_hash,
        amount_sompi,
        fee_sompi,
        covenant_address,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assemble_creates_covenant_address() {
        let pk = [1u8; 32];
        let treasury = [0u8; 32];
        let result = assemble_create_escrow(&pk, &pk, 100_000_000, 86400, &treasury).unwrap();
        assert!(result.covenant_address.starts_with("kaspatest:p"));
    }

    #[test]
    fn kas_to_sompi_integer() {
        assert_eq!(kas_to_sompi("5000").unwrap(), 500_000_000_000);
    }

    #[test]
    fn kas_to_sompi_decimal() {
        assert_eq!(kas_to_sompi("5000.5").unwrap(), 500_050_000_000);
    }

    #[test]
    fn kas_to_sompi_small_decimal() {
        assert_eq!(kas_to_sompi("0.00000001").unwrap(), 1);
    }

    #[test]
    fn kas_to_sompi_no_decimal() {
        assert_eq!(kas_to_sompi("100").unwrap(), 10_000_000_000);
    }

    #[test]
    fn kas_to_sompi_truncates_excess_decimals() {
        // More than 8 decimal places should be truncated to 8
        // "1.123456789" → 1 KAS + 0.12345678 KAS = 112_345_678 sompi
        assert_eq!(kas_to_sompi("1.123456789").unwrap(), 112_345_678);
    }
}
