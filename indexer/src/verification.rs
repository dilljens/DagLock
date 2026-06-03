//! On-chain verification module for DagLock escrows.
//!
//! Provides verification of UTXO existence and signature validation.
//! Currently uses a mock implementation; will be replaced with wRPC-based
//! verification when the node connection is fully implemented.

use crate::types::Escrow;
use tracing::warn;

/// Errors that can occur during verification.
#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("UTXO not found on-chain: {tx_id}:{output_index}")]
    UtxoNotFound { tx_id: String, output_index: u32 },



    #[error("Verification failed: {0}")]
    Other(String),
}

/// Result type for verification operations.
pub type VerificationResult<T> = Result<T, VerificationError>;

/// Trait for verifying escrow-related on-chain data.
///
/// # Implementors
/// - `MockVerifier`: Always returns success (for testing)
/// - `WrpcVerifier`: Uses wRPC client to verify on-chain (TODO)
pub trait EscrowVerifier: Send + Sync {
    /// Verify that the escrow UTXO exists on-chain.
    fn verify_utxo_exists(&self, escrow: &Escrow) -> VerificationResult<bool>;

    /// Verify that a signature is valid for the given address.
    fn verify_signature(
        &self,
        address: &str,
        signature: &[u8],
        message: &[u8],
    ) -> VerificationResult<bool>;
}

/// Mock verifier that always returns success.
///
/// Use for testing or when on-chain verification is not available.
/// WARNING: Do not use in production — provides no actual security.
pub struct MockVerifier;

impl EscrowVerifier for MockVerifier {
    fn verify_utxo_exists(&self, _escrow: &Escrow) -> VerificationResult<bool> {
        // Mock always returns true — UTXO is "verified"
        warn!("Using MockVerifier — no actual UTXO verification performed");
        Ok(true)
    }

    fn verify_signature(
        &self,
        _address: &str,
        _signature: &[u8],
        _message: &[u8],
    ) -> VerificationResult<bool> {
        // Mock always returns true — signature is "valid"
        warn!("Using MockVerifier — no actual signature verification performed");
        Ok(true)
    }
}

/// Verify escrow can be settled.
///
/// Checks:
/// 1. Escrow exists in the database
/// 2. Escrow is in a settleable state (active)
/// 3. UTXO exists on-chain (if verifier is available)
pub fn verify_escrow_settleable(
    escrow: &Escrow,
    verifier: &dyn EscrowVerifier,
) -> VerificationResult<()> {
    // Check status
    if !matches!(escrow.status, crate::types::EscrowStatus::Active) {
        return Err(VerificationError::Other(format!(
            "Escrow is not in active state: {:?}",
            escrow.status
        )));
    }

    // Verify UTXO exists on-chain
    if !verifier.verify_utxo_exists(escrow)? {
        return Err(VerificationError::UtxoNotFound {
            tx_id: escrow.lock_tx_id.clone(),
            output_index: escrow.lock_tx_output_index,
        });
    }

    Ok(())
}

/// Verify escrow can be refunded.
///
/// Checks:
/// 1. Escrow exists in the database
/// 2. Escrow is in a refundable state (active)
/// 3. UTXO exists on-chain (if verifier is available)
pub fn verify_escrow_refundable(
    escrow: &Escrow,
    verifier: &dyn EscrowVerifier,
) -> VerificationResult<()> {
    // Check status
    if !matches!(escrow.status, crate::types::EscrowStatus::Active) {
        return Err(VerificationError::Other(format!(
            "Escrow is not in active state: {:?}",
            escrow.status
        )));
    }

    // Verify UTXO exists on-chain
    if !verifier.verify_utxo_exists(escrow)? {
        return Err(VerificationError::UtxoNotFound {
            tx_id: escrow.lock_tx_id.clone(),
            output_index: escrow.lock_tx_output_index,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn test_escrow(status: EscrowStatus) -> Escrow {
        Escrow {
            id: "esc_test".to_string(),
            lock_tx_id: "tx123".to_string(),
            lock_tx_output_index: 0,
            status,
            asset_type: "KAS".to_string(),
            buyer_address: "kaspa:buyer".to_string(),
            seller_address: Some("kaspa:seller".to_string()),
            amount_sompi: 1_000_000_000,
            fee_sompi: 5_000_000,
            template_hash: vec![1, 2, 3],
            expiration_daa_score: Some(1000),
            disputed_at: None,
            dispute_reason: None,
            cancelled_at: None,
            expired_at: None,
            created_at: 1_700_000_000,
            settled_at: None,
            refunded_at: None,
        }
    }

    #[test]
    fn mock_verifier_always_succeeds() {
        let verifier = MockVerifier;
        let escrow = test_escrow(EscrowStatus::Active);
        assert!(verifier.verify_utxo_exists(&escrow).unwrap());
    }

    #[test]
    fn verify_escrow_settleable_with_active_escrow() {
        let verifier = MockVerifier;
        let escrow = test_escrow(EscrowStatus::Active);
        assert!(verify_escrow_settleable(&escrow, &verifier).is_ok());
    }

    #[test]
    fn verify_escrow_settleable_fails_for_settled() {
        let verifier = MockVerifier;
        let escrow = test_escrow(EscrowStatus::Settled);
        assert!(verify_escrow_settleable(&escrow, &verifier).is_err());
    }

    #[test]
    fn verify_escrow_refundable_fails_for_expired() {
        let verifier = MockVerifier;
        let escrow = test_escrow(EscrowStatus::Expired);
        assert!(verify_escrow_refundable(&escrow, &verifier).is_err());
    }
}
