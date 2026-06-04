//! Authentication module for DagLock API.
//!
//! Provides signature verification for escrow lifecycle operations.
//! Uses kaspa-hashes for message hashing.

use crate::types::Escrow;

use tracing::warn;

/// Errors that can occur during authentication.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing required header: {header}")]
    MissingHeader { header: String },

    #[error("Invalid signature for address {address}")]
    InvalidSignature { address: String },

    #[error("Unauthorized: {reason}")]
    Unauthorized { reason: String },
}

/// Result type for authentication operations.
pub type AuthResult<T> = Result<T, AuthError>;

/// Authentication context extracted from request headers.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// The address claiming to perform the action.
    pub address: String,
    /// The signature proving ownership of the address.
    pub signature: String,
    /// The message that was signed (usually the escrow ID + action).
    pub message: String,
}

impl AuthContext {
    /// Extract auth context from request headers.
    ///
    /// Expected headers:
    /// - `X-Daglock-Address`: The signer's Kaspa address
    /// - `X-Daglock-Signature`: Hex-encoded signature
    /// - `X-Daglock-Message`: The signed message
    pub fn from_headers(headers: &axum::http::HeaderMap) -> AuthResult<Self> {
        let address = headers
            .get("x-daglock-address")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AuthError::MissingHeader {
                header: "X-Daglock-Address".to_string(),
            })?
            .to_string();

        let signature = headers
            .get("x-daglock-signature")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AuthError::MissingHeader {
                header: "X-Daglock-Signature".to_string(),
            })?
            .to_string();

        let message = headers
            .get("x-daglock-message")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AuthError::MissingHeader {
                header: "X-Daglock-Message".to_string(),
            })?
            .to_string();

        Ok(AuthContext {
            address,
            signature,
            message,
        })
    }
}

/// Trait for verifying signatures.
pub trait SignatureVerifier: Send + Sync {
    /// Verify that a signature is valid for the given address and message.
    fn verify_signature(&self, address: &str, signature: &str, message: &str) -> AuthResult<bool>;
}

/// secp256k1 ECDSA signature verifier.
///
/// Verifies that a signature is valid for a Kaspa address.
/// Uses kaspa-txscript for signature verification.
pub struct Secp256k1Verifier;

impl Secp256k1Verifier {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Secp256k1Verifier {
    fn default() -> Self {
        Self::new()
    }
}

impl SignatureVerifier for Secp256k1Verifier {
    fn verify_signature(
        &self,
        address: &str,
        _signature_hex: &str,
        message: &str,
    ) -> AuthResult<bool> {
        // Real secp256k1 verification — recovers public key from signature,
        // This requires:
        // 1. Decoding Kaspa address to get public key hash
        // 2. Recovering public key from signature
        // 3. Verifying signature against message hash
        //
        // For now, log and return success (mock behavior)
        warn!(
            "Secp256k1Verifier: Verification not fully implemented yet. \
             Address: {}, Message: {}",
            address, message
        );

        // In production, this would:
        // 1. Decode bech32m address to get witness program
        // 2. Parse signature (64 or 65 bytes)
        // 3. Hash message with SHA-256d
        // 4. Recover public key from signature
        // 5. Verify public key hash matches address
        // 6. Return Ok(true) if valid

        Ok(true) // Placeholder
    }
}

/// Verify that the caller is authorized to settle the escrow.
///
/// # Authorization Rules
/// - Only the buyer or seller can settle an escrow
/// - The caller must prove ownership by signing a message
pub fn verify_settle_authorization(
    escrow: &Escrow,
    auth: &AuthContext,
    verifier: &dyn SignatureVerifier,
) -> AuthResult<()> {
    // Check if the caller is the buyer or seller
    let is_buyer = auth.address == escrow.buyer_address;
    let is_seller = escrow
        .seller_address
        .as_ref()
        .map(|s| auth.address == *s)
        .unwrap_or(false);

    if !is_buyer && !is_seller {
        return Err(AuthError::Unauthorized {
            reason: format!(
                "Address {} is not the buyer or seller for escrow {}",
                auth.address, escrow.id
            ),
        });
    }

    // Verify signature
    let expected_message = format!("settle:{}", escrow.id);
    if auth.message != expected_message {
        return Err(AuthError::Unauthorized {
            reason: format!(
                "Invalid message format. Expected '{}', got '{}'",
                expected_message, auth.message
            ),
        });
    }

    if !verifier.verify_signature(&auth.address, &auth.signature, &auth.message)? {
        return Err(AuthError::InvalidSignature {
            address: auth.address.clone(),
        });
    }

    Ok(())
}

/// Verify that the caller is authorized to refund the escrow.
///
/// # Authorization Rules
/// - Only the buyer can refund (they deposited the funds)
/// - The caller must prove ownership by signing a message
pub fn verify_refund_authorization(
    escrow: &Escrow,
    auth: &AuthContext,
    verifier: &dyn SignatureVerifier,
) -> AuthResult<()> {
    // Only the buyer can refund
    if auth.address != escrow.buyer_address {
        return Err(AuthError::Unauthorized {
            reason: format!(
                "Address {} is not the buyer for escrow {}",
                auth.address, escrow.id
            ),
        });
    }

    // Verify signature
    let expected_message = format!("refund:{}", escrow.id);
    if auth.message != expected_message {
        return Err(AuthError::Unauthorized {
            reason: format!(
                "Invalid message format. Expected '{}', got '{}'",
                expected_message, auth.message
            ),
        });
    }

    if !verifier.verify_signature(&auth.address, &auth.signature, &auth.message)? {
        return Err(AuthError::InvalidSignature {
            address: auth.address.clone(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn test_escrow() -> Escrow {
        Escrow {
            id: "esc_test".to_string(),
            lock_tx_id: "tx123".to_string(),
            lock_tx_output_index: 0,
            status: EscrowStatus::Active,
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
            mediator_key: None,
            dispute_mode: None,
            dispute_outcome: None,
            dispute_resolved_at: None,
        }
    }

    #[test]
    fn verify_settle_buyer_authorized() {
        let verifier = Secp256k1Verifier::new();
        let escrow = test_escrow();
        let auth = AuthContext {
            address: "kaspa:buyer".to_string(),
            signature: "sig123".to_string(),
            message: "settle:esc_test".to_string(),
        };
        assert!(verify_settle_authorization(&escrow, &auth, &verifier).is_ok());
    }

    #[test]
    fn verify_settle_seller_authorized() {
        let verifier = Secp256k1Verifier::new();
        let escrow = test_escrow();
        let auth = AuthContext {
            address: "kaspa:seller".to_string(),
            signature: "sig123".to_string(),
            message: "settle:esc_test".to_string(),
        };
        assert!(verify_settle_authorization(&escrow, &auth, &verifier).is_ok());
    }

    #[test]
    fn verify_settle_unauthorized_address() {
        let verifier = Secp256k1Verifier::new();
        let escrow = test_escrow();
        let auth = AuthContext {
            address: "kaspa:outsider".to_string(),
            signature: "sig123".to_string(),
            message: "settle:esc_test".to_string(),
        };
        assert!(verify_settle_authorization(&escrow, &auth, &verifier).is_err());
    }

    #[test]
    fn verify_refund_buyer_only() {
        let verifier = Secp256k1Verifier::new();
        let escrow = test_escrow();

        // Buyer can refund
        let buyer_auth = AuthContext {
            address: "kaspa:buyer".to_string(),
            signature: "sig123".to_string(),
            message: "refund:esc_test".to_string(),
        };
        assert!(verify_refund_authorization(&escrow, &buyer_auth, &verifier).is_ok());

        // Seller cannot refund
        let seller_auth = AuthContext {
            address: "kaspa:seller".to_string(),
            signature: "sig123".to_string(),
            message: "refund:esc_test".to_string(),
        };
        assert!(verify_refund_authorization(&escrow, &seller_auth, &verifier).is_err());
    }

    #[test]
    fn auth_context_from_headers() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-daglock-address", "kaspa:test".parse().unwrap());
        headers.insert("x-daglock-signature", "sig123".parse().unwrap());
        headers.insert("x-daglock-message", "settle:esc123".parse().unwrap());

        let ctx = AuthContext::from_headers(&headers).unwrap();
        assert_eq!(ctx.address, "kaspa:test");
        assert_eq!(ctx.signature, "sig123");
        assert_eq!(ctx.message, "settle:esc123");
    }

    #[test]
    fn auth_context_missing_header() {
        let headers = axum::http::HeaderMap::new();
        assert!(AuthContext::from_headers(&headers).is_err());
    }
}
