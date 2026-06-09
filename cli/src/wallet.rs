//! Wallet integration for DagLock CLI.
//!
//! Shells out to `kaspawallet` for signing transactions.
//! Falls back to user-provided keys when kaspawallet is not available.

use anyhow::{Context, Result};
use std::process::Command;

/// Sign an unsigned transaction hex with `kaspawallet`.
/// Returns the signed transaction hex.
pub fn sign_with_kaspawallet(unsigned_tx_hex: &str) -> Result<String> {
    let output = Command::new("kaspawallet")
        .arg("sign")
        .arg("--transaction")
        .arg(unsigned_tx_hex)
        .output()
        .context("Failed to run kaspawallet. Is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("kaspawallet sign failed: {}", stderr.trim());
    }

    let result = String::from_utf8(output.stdout).context("Invalid UTF-8 from kaspawallet")?;

    Ok(result.trim().to_string())
}

/// Parse a hex key string into a 32-byte array.
pub fn parse_hex_key(hex_str: &str) -> Result<[u8; 32]> {
    let bytes = hex::decode(hex_str).context("Invalid hex key")?;
    if bytes.len() != 32 {
        anyhow::bail!(
            "Key must be 64 hex characters (32 bytes), got {}",
            hex_str.len()
        );
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

/// Check if kaspawallet is available on the system PATH.
pub fn kaspawallet_available() -> bool {
    Command::new("kaspawallet")
        .arg("--version")
        .output()
        .is_ok()
}
