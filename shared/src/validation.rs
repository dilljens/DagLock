//! Shared validation utilities for DagLock.

use crate::constants::*;
use thiserror::Error;

/// Errors that can occur during validation.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid hex string: {0}")]
    InvalidHex(#[from] hex::FromHexError),

    #[error(
        "Invalid length: expected {expected} bytes ({expected_hex} hex chars), got {got} bytes"
    )]
    InvalidLength {
        expected: usize,
        expected_hex: usize,
        got: usize,
    },

    #[error("Invalid Kaspa address: {0}")]
    InvalidAddress(String),

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Invalid timeout: {0}")]
    InvalidTimeout(String),

    #[error("Invalid template hash: {0}")]
    InvalidTemplateHash(String),

    #[error("Invalid trade hash: {0}")]
    InvalidTradeHash(String),
}

/// Result type for validation operations.
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Validates a trade hash (SHA-256, 32 bytes = 64 hex chars).
///
/// # Arguments
/// * `hash_hex` - Hex-encoded trade hash (64 characters)
///
/// # Returns
/// * `Ok([u8; 32])` - The decoded 32-byte hash
/// * `Err(ValidationError)` - If the hash is invalid
pub fn validate_trade_hash(hash_hex: &str) -> ValidationResult<[u8; TRADE_HASH_LENGTH]> {
    let trimmed = hash_hex.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::InvalidTradeHash(
            "trade hash cannot be empty".to_string(),
        ));
    }

    let bytes = hex::decode(trimmed)?;
    if bytes.len() != TRADE_HASH_LENGTH {
        return Err(ValidationError::InvalidLength {
            expected: TRADE_HASH_LENGTH,
            expected_hex: TRADE_HASH_LENGTH * 2,
            got: bytes.len(),
        });
    }

    let mut arr = [0u8; TRADE_HASH_LENGTH];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

/// Validates a trade hash, returning empty array if input is empty/None (for non-atomic escrows).
///
/// # Arguments
/// * `hash_hex` - Optional hex-encoded trade hash (64 characters)
///
/// # Returns
/// * `Ok([u8; 32])` - The decoded 32-byte hash, or zeroed array if empty
/// * `Err(ValidationError)` - If the hash is provided but invalid
pub fn validate_trade_hash_optional(
    hash_hex: Option<&str>,
) -> ValidationResult<[u8; TRADE_HASH_LENGTH]> {
    match hash_hex {
        Some(h) if !h.trim().is_empty() => validate_trade_hash(h),
        _ => Ok([0u8; TRADE_HASH_LENGTH]),
    }
}

/// Validates a template hash (BLAKE2b-160, 20 bytes = 40 hex chars).
pub fn validate_template_hash(hash_hex: &str) -> ValidationResult<[u8; TEMPLATE_HASH_LENGTH]> {
    let trimmed = hash_hex.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::InvalidTemplateHash(
            "template hash cannot be empty".to_string(),
        ));
    }

    let bytes = hex::decode(trimmed)?;
    if bytes.len() != TEMPLATE_HASH_LENGTH {
        return Err(ValidationError::InvalidLength {
            expected: TEMPLATE_HASH_LENGTH,
            expected_hex: TEMPLATE_HASH_LENGTH * 2,
            got: bytes.len(),
        });
    }

    let mut arr = [0u8; TEMPLATE_HASH_LENGTH];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

/// Validates a Kaspa address format.
///
/// # Arguments
/// * `address` - The address string to validate
///
/// # Returns
/// * `Ok(())` - If the address is valid
/// * `Err(ValidationError)` - If the address is invalid
pub fn validate_kaspa_address(address: &str) -> ValidationResult<()> {
    let trimmed = address.trim();

    if trimmed.is_empty() {
        return Err(ValidationError::InvalidAddress(
            "address cannot be empty".to_string(),
        ));
    }

    // Check prefix
    let valid_prefixes = [
        KASPA_MAINNET_PREFIX,
        KASPA_TESTNET_PREFIX,
        KASPA_SIMNET_PREFIX,
    ];
    let has_valid_prefix = valid_prefixes.iter().any(|p| trimmed.starts_with(p));

    if !has_valid_prefix {
        return Err(ValidationError::InvalidAddress(format!(
            "address must start with one of: {}",
            valid_prefixes.join(", ")
        )));
    }

    // Basic bech32 character validation
    let prefix_len = valid_prefixes
        .iter()
        .find(|p| trimmed.starts_with(*p))
        .map(|p| p.len())
        .unwrap_or(0);

    if trimmed.len() <= prefix_len {
        return Err(ValidationError::InvalidAddress(
            "address too short".to_string(),
        ));
    }

    let bech32_chars = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let payload = &trimmed[prefix_len..];

    if !payload.chars().all(|c| bech32_chars.contains(c)) {
        return Err(ValidationError::InvalidAddress(
            "address contains invalid bech32 characters".to_string(),
        ));
    }

    Ok(())
}

/// Validates an escrow amount in sompi.
///
/// # Arguments
/// * `amount_sompi` - Amount in sompi (1 KAS = 100_000_000 sompi)
///
/// # Returns
/// * `Ok(())` - If the amount is valid
/// * `Err(ValidationError)` - If the amount is invalid
pub fn validate_amount_sompi(amount_sompi: u64) -> ValidationResult<()> {
    if amount_sompi < MIN_ESCROW_AMOUNT_SOMPI {
        return Err(ValidationError::InvalidAmount(format!(
            "amount must be at least {} sompi",
            MIN_ESCROW_AMOUNT_SOMPI
        )));
    }
    if amount_sompi > MAX_ESCROW_AMOUNT_SOMPI {
        return Err(ValidationError::InvalidAmount(format!(
            "amount exceeds maximum of {} sompi ({} KAS)",
            MAX_ESCROW_AMOUNT_SOMPI,
            MAX_ESCROW_AMOUNT_SOMPI / 100_000_000
        )));
    }
    Ok(())
}

/// Validates a timeout value (Unix timestamp).
///
/// # Arguments
/// * `timeout` - Unix timestamp
/// * `now` - Current Unix timestamp for relative validation
///
/// # Returns
/// * `Ok(())` - If the timeout is valid
/// * `Err(ValidationError)` - If the timeout is invalid
pub fn validate_timeout(timeout: i64, now: i64) -> ValidationResult<()> {
    if timeout <= now {
        return Err(ValidationError::InvalidTimeout(
            "timeout must be in the future".to_string(),
        ));
    }
    let max_timeout = now + MAX_TIMEOUT_SECONDS as i64;
    if timeout > max_timeout {
        return Err(ValidationError::InvalidTimeout(format!(
            "timeout cannot exceed {} seconds from now ({} days)",
            MAX_TIMEOUT_SECONDS,
            MAX_TIMEOUT_SECONDS / 86_400
        )));
    }
    Ok(())
}

/// Validates a timeout duration in seconds (relative to now).
///
/// # Arguments
/// * `timeout_seconds` - Timeout in seconds from now
///
/// # Returns
/// * `Ok(i64)` - The absolute Unix timestamp
/// * `Err(ValidationError)` - If the timeout is invalid
pub fn validate_timeout_duration(timeout_seconds: u64) -> ValidationResult<i64> {
    if timeout_seconds == 0 {
        return Err(ValidationError::InvalidTimeout(
            "timeout must be greater than 0".to_string(),
        ));
    }
    if timeout_seconds > MAX_TIMEOUT_SECONDS {
        return Err(ValidationError::InvalidTimeout(format!(
            "timeout cannot exceed {} seconds ({} days)",
            MAX_TIMEOUT_SECONDS,
            MAX_TIMEOUT_SECONDS / 86_400
        )));
    }
    let now = chrono::Utc::now().timestamp();
    Ok(now + timeout_seconds as i64)
}

/// Calculates the protocol fee for a given amount.
///
/// # Arguments
/// * `amount_sompi` - Amount in sompi
///
/// # Returns
/// * Fee in sompi (amount / FEE_DENOMINATOR)
pub fn calculate_fee(amount_sompi: u64) -> u64 {
    amount_sompi / FEE_DENOMINATOR
}

/// Calculates the net amount after fee deduction.
///
/// # Arguments
/// * `amount_sompi` - Gross amount in sompi
///
/// # Returns
/// * Net amount in sompi (amount - fee)
pub fn calculate_net_amount(amount_sompi: u64) -> u64 {
    amount_sompi - calculate_fee(amount_sompi)
}

/// Converts KAS string (e.g., "100.5") to sompi.
///
/// # Arguments
/// * `kas_str` - Amount in KAS as string
///
/// # Returns
/// * Amount in sompi
/// * `Err(ValidationError)` - If the string is invalid
pub fn kas_to_sompi(kas_str: &str) -> ValidationResult<u64> {
    let trimmed = kas_str.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::InvalidAmount(
            "amount cannot be empty".to_string(),
        ));
    }

    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.len() > 2 {
        return Err(ValidationError::InvalidAmount(
            "invalid amount format: too many decimal points".to_string(),
        ));
    }

    let whole: u64 = parts[0]
        .parse()
        .map_err(|_| ValidationError::InvalidAmount("invalid whole number part".to_string()))?;

    let fractional = if parts.len() == 2 {
        let frac_str = parts[1];
        if frac_str.len() > 8 {
            return Err(ValidationError::InvalidAmount(
                "too many decimal places (max 8)".to_string(),
            ));
        }
        let padded = format!("{:0<8}", frac_str);
        padded
            .parse()
            .map_err(|_| ValidationError::InvalidAmount("invalid fractional part".to_string()))?
    } else {
        0
    };

    let sompi = whole
        .checked_mul(100_000_000)
        .and_then(|w| w.checked_add(fractional))
        .ok_or_else(|| ValidationError::InvalidAmount("amount overflow".to_string()))?;

    validate_amount_sompi(sompi)?;
    Ok(sompi)
}

/// Converts sompi to KAS string with 8 decimal places.
///
/// # Arguments
/// * `sompi` - Amount in sompi
///
/// # Returns
/// * Formatted string (e.g., "100.50000000")
pub fn sompi_to_kas(sompi: u64) -> String {
    let whole = sompi / 100_000_000;
    let frac = sompi % 100_000_000;
    format!("{}.{:08}", whole, frac)
}

/// Parses a hex string into a fixed-size byte array.
///
/// # Arguments
/// * `hex_str` - Hex-encoded string
/// * `expected_len` - Expected byte length
///
/// # Returns
/// * `Ok([u8; N])` - The decoded bytes
/// * `Err(ValidationError)` - If the hex is invalid or wrong length
pub fn parse_hex<const N: usize>(hex_str: &str) -> ValidationResult<[u8; N]> {
    let trimmed = hex_str.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::InvalidHex(
            hex::FromHexError::InvalidStringLength,
        ));
    }

    let bytes = hex::decode(trimmed)?;
    if bytes.len() != N {
        return Err(ValidationError::InvalidLength {
            expected: N,
            expected_hex: N * 2,
            got: bytes.len(),
        });
    }

    let mut arr = [0u8; N];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_trade_hash_valid() {
        let hash = "a".repeat(64);
        let result = validate_trade_hash(&hash);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 32);
    }

    #[test]
    fn validate_trade_hash_invalid_length() {
        let hash = "a".repeat(62);
        let result = validate_trade_hash(&hash);
        assert!(result.is_err());
    }

    #[test]
    fn validate_trade_hash_invalid_hex() {
        let hash = "g".repeat(64);
        let result = validate_trade_hash(&hash);
        assert!(result.is_err());
    }

    #[test]
    fn validate_trade_hash_optional_empty() {
        let result = validate_trade_hash_optional(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0u8; 32]);

        let result = validate_trade_hash_optional(Some(""));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0u8; 32]);

        let result = validate_trade_hash_optional(Some("   "));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0u8; 32]);
    }

    #[test]
    fn validate_template_hash_valid() {
        let hash = "a".repeat(40);
        let result = validate_template_hash(&hash);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 20);
    }

    #[test]
    fn validate_kaspa_address_mainnet() {
        assert!(validate_kaspa_address("kaspa:qdyzkrhd74v6cetrv4fhv").is_ok());
    }

    #[test]
    fn validate_kaspa_address_testnet() {
        assert!(validate_kaspa_address(
            "kaspatest:qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpk58a75"
        )
        .is_ok());
    }

    #[test]
    fn validate_kaspa_address_invalid_prefix() {
        assert!(validate_kaspa_address("bitcoin:123").is_err());
    }

    #[test]
    fn validate_kaspa_address_invalid_chars() {
        assert!(validate_kaspa_address("kaspa:invalid!chars").is_err());
    }

    #[test]
    fn validate_amount_sompi_valid() {
        assert!(validate_amount_sompi(100_000_000).is_ok()); // 1 KAS
        assert!(validate_amount_sompi(1).is_ok()); // 1 sompi
    }

    #[test]
    fn validate_amount_sompi_too_large() {
        assert!(validate_amount_sompi(MAX_ESCROW_AMOUNT_SOMPI + 1).is_err());
    }

    #[test]
    fn validate_timeout_duration_valid() {
        let result = validate_timeout_duration(86_400); // 24 hours
        assert!(result.is_ok());
        assert!(result.unwrap() > chrono::Utc::now().timestamp());
    }

    #[test]
    fn validate_timeout_duration_zero() {
        assert!(validate_timeout_duration(0).is_err());
    }

    #[test]
    fn calculate_fee_correct() {
        assert_eq!(calculate_fee(200_000_000), 1_000_000); // 2 KAS -> 0.01 KAS fee
        assert_eq!(calculate_fee(100_000_000), 500_000); // 1 KAS -> 0.005 KAS fee
        assert_eq!(calculate_fee(199), 0); // < 200 sompi -> 0 fee (integer division)
    }

    #[test]
    fn calculate_net_amount_correct() {
        assert_eq!(calculate_net_amount(200_000_000), 199_000_000);
        assert_eq!(calculate_net_amount(100_000_000), 99_500_000);
    }

    #[test]
    fn kas_to_sompi_valid() {
        assert_eq!(kas_to_sompi("1").unwrap(), 100_000_000);
        assert_eq!(kas_to_sompi("1.5").unwrap(), 150_000_000);
        assert_eq!(kas_to_sompi("0.00000001").unwrap(), 1);
        assert_eq!(kas_to_sompi("1000000").unwrap(), 100_000_000_000_000); // 1M KAS
    }

    #[test]
    fn kas_to_sompi_invalid() {
        assert!(kas_to_sompi("").is_err());
        assert!(kas_to_sompi("abc").is_err());
        assert!(kas_to_sompi("1.2.3").is_err());
        assert!(kas_to_sompi("1.123456789").is_err()); // too many decimals
    }

    #[test]
    fn sompi_to_kas_formatting() {
        assert_eq!(sompi_to_kas(100_000_000), "1.00000000");
        assert_eq!(sompi_to_kas(150_000_000), "1.50000000");
        assert_eq!(sompi_to_kas(1), "0.00000001");
        assert_eq!(sompi_to_kas(0), "0.00000000");
    }

    #[test]
    fn parse_hex_valid() {
        let result = parse_hex::<32>("aa".repeat(32).as_str());
        assert!(result.is_ok());
        assert_eq!(result.unwrap()[0], 0xaa);
    }

    #[test]
    fn fee_constants() {
        assert_eq!(FEE_DENOMINATOR, 200);
        assert_eq!(FEE_BASIS_POINTS, 50);
    }
}
