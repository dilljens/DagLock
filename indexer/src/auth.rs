//! Authentication module for DagLock API.
//!
//! Two modes:
//!   - `MockVerifier` — any hex string passes (dev/testnet, default)
//!   - `SchnorrVerifier` — real Kaspa Schnorr signature verification
//!
//! Mock auth is rejected on mainnet via a startup safety check.

use crate::types::Escrow;

use kaspa_addresses::{Address, Version};
use kaspa_hashes::{Hash, PersonalMessageSigningHash};
use secp256k1::XOnlyPublicKey;
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
    /// - `X-Daglock-Signature`: Hex-encoded 64-byte Schnorr signature
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

/// Mock verifier — accepts any hex string as valid.
/// Used for dev/testnet. Rejected on mainnet via startup check.
pub struct MockVerifier;

impl MockVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl SignatureVerifier for MockVerifier {
    fn verify_signature(
        &self,
        address: &str,
        _signature: &str,
        _message: &str,
    ) -> AuthResult<bool> {
        warn!(
            "MockVerifier: accepting any signature for address {} (dev mode)",
            address
        );
        Ok(true)
    }
}

/// Real Schnorr signature verifier for Kaspa addresses.
///
/// Verification flow:
/// 1. Parse bech32m address → extract 32-byte x-only public key payload
/// 2. Hash message with PersonalMessageSigningHash (SHA-256d with prefix)
/// 3. Recover XOnlyPublicKey from the 64-byte Schnorr signature
/// 4. Check recovered key's address matches the claimed address
///
/// This matches how KasWare and Kaspium sign messages (BIP-340 Schnorr).
pub struct SchnorrVerifier;

impl SchnorrVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SchnorrVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl SignatureVerifier for SchnorrVerifier {
    fn verify_signature(
        &self,
        address_str: &str,
        signature_hex: &str,
        message: &str,
    ) -> AuthResult<bool> {
        // 1. Parse the Kaspa address to extract the pubkey payload
        let addr: Address = address_str
            .try_into()
            .map_err(|_| AuthError::InvalidSignature {
                address: address_str.to_string(),
            })?;

        // We only support PubKey version (Schnorr x-only, 32 bytes)
        if addr.version != Version::PubKey {
            return Err(AuthError::InvalidSignature {
                address: address_str.to_string(),
            });
        }

        let pubkey_bytes = addr.payload.as_ref();
        if pubkey_bytes.len() != 32 {
            return Err(AuthError::InvalidSignature {
                address: address_str.to_string(),
            });
        }

        // 2. Decode hex signature (should be 64 bytes for Schnorr)
        let sig_bytes = hex::decode(signature_hex).map_err(|_| AuthError::InvalidSignature {
            address: address_str.to_string(),
        })?;

        if sig_bytes.len() != 64 {
            return Err(AuthError::InvalidSignature {
                address: address_str.to_string(),
            });
        }

        // 3. Parse XOnlyPublicKey from the address payload
        let pubkey =
            XOnlyPublicKey::from_slice(pubkey_bytes).map_err(|_| AuthError::InvalidSignature {
                address: address_str.to_string(),
            })?;

        // 4. Hash the message using Kaspa's PersonalMessageSigningHash
        //    This is SHA-256(SHA-256("Kaspa Personal Message") || SHA-256("Kaspa Personal Message") || message)
        let mut hasher = PersonalMessageSigningHash::new();
        hasher.write(message.as_bytes());
        let hash: Hash = hasher.finalize();

        let msg =
            secp256k1::Message::from_digest_slice(hash.as_bytes().as_slice()).map_err(|_| {
                AuthError::InvalidSignature {
                    address: address_str.to_string(),
                }
            })?;

        // 5. Parse Schnorr signature and verify
        let sig = secp256k1::schnorr::Signature::from_slice(&sig_bytes).map_err(|_| {
            AuthError::InvalidSignature {
                address: address_str.to_string(),
            }
        })?;

        match sig.verify(&msg, &pubkey) {
            Ok(()) => Ok(true),
            Err(e) => {
                warn!(
                    "SchnorrVerifier: signature verification failed for {}: {}",
                    address_str, e
                );
                Err(AuthError::InvalidSignature {
                    address: address_str.to_string(),
                })
            }
        }
    }
}

/// Helper to create the correct verifier based on config.
/// Panics if mock_auth is true on mainnet.
pub fn create_verifier(network: &str, mock_auth: bool) -> Box<dyn SignatureVerifier> {
    if mock_auth {
        if network == "mainnet" {
            panic!(
                "REFUSING TO START: --mock-auth is set but network is mainnet. \
                 This would allow anyone to impersonate any user. \
                 Remove --mock-auth to use real Schnorr signature verification."
            );
        }
        warn!("Using mock authentication — any signature will be accepted. Never use this on mainnet.");
        Box::new(MockVerifier::new())
    } else {
        Box::new(SchnorrVerifier::new())
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

/// Extract auth context from headers and verify a signature.
/// Returns Ok(()) if the signature is valid.
#[allow(dead_code)]
pub fn verify_auth(
    headers: &axum::http::HeaderMap,
    verifier: &dyn SignatureVerifier,
    expected_message: &str,
) -> AuthResult<AuthContext> {
    let auth = AuthContext::from_headers(headers)?;

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

    Ok(auth)
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
            buyer_address:
                "kaspatest:qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpk58a75"
                    .to_string(),
            seller_address: Some(
                "kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya"
                    .to_string(),
            ),
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
            price_at_creation: None,
            price_currency: None,
            trade_hash: None,
            price_lock_time: None,
            price_at_settlement: None,
            price_source: None,
        }
    }

    #[test]
    fn mock_verifier_accepts_anything() {
        let verifier = MockVerifier::new();
        assert!(verifier
            .verify_signature("kaspatest:test", "abcd", "settle:esc_test")
            .is_ok());
        assert!(verifier
            .verify_signature("kaspatest:test", "", "settle:esc_test")
            .is_ok());
        assert!(verifier
            .verify_signature("kaspatest:test", "garbage", "")
            .is_ok());
    }

    #[test]
    fn schnorr_verifier_rejects_invalid_inputs() {
        let verifier = SchnorrVerifier::new();

        // Invalid address
        assert!(verifier
            .verify_signature("not-an-address", "abcd", "test")
            .is_err());

        // Invalid hex signature
        assert!(verifier
            .verify_signature(
                "kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya",
                "not-hex",
                "test"
            )
            .is_err());

        // Wrong-length signature (too short)
        assert!(verifier
            .verify_signature(
                "kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya",
                "aabb",
                "test"
            )
            .is_err());
    }

    #[test]
    fn schnorr_verifier_rejects_bad_signature() {
        let verifier = SchnorrVerifier::new();
        // A valid 64-byte hex that isn't a real signature
        let fake_sig = "ab".repeat(64);

        let result = verifier.verify_signature(
            "kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya",
            &fake_sig,
            "test message",
        );
        assert!(result.is_err());
    }

    #[test]
    fn verify_settle_buyer_authorized() {
        let verifier = MockVerifier::new();
        let escrow = test_escrow();
        let auth = AuthContext {
            address: "kaspatest:qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpk58a75"
                .to_string(),
            signature: "any_hex".to_string(),
            message: "settle:esc_test".to_string(),
        };
        assert!(verify_settle_authorization(&escrow, &auth, &verifier).is_ok());
    }

    #[test]
    fn verify_settle_seller_authorized() {
        let verifier = MockVerifier::new();
        let escrow = test_escrow();
        let auth = AuthContext {
            address: "kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya"
                .to_string(),
            signature: "sig123".to_string(),
            message: "settle:esc_test".to_string(),
        };
        assert!(verify_settle_authorization(&escrow, &auth, &verifier).is_ok());
    }

    #[test]
    fn verify_settle_unauthorized_address() {
        let verifier = MockVerifier::new();
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
        let verifier = MockVerifier::new();
        let escrow = test_escrow();

        // Buyer can refund
        let buyer_auth = AuthContext {
            address: "kaspatest:qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpk58a75"
                .to_string(),
            signature: "sig123".to_string(),
            message: "refund:esc_test".to_string(),
        };
        assert!(verify_refund_authorization(&escrow, &buyer_auth, &verifier).is_ok());

        // Seller cannot refund
        let seller_auth = AuthContext {
            address: "kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya"
                .to_string(),
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

    #[test]
    fn create_verifier_panics_on_mainnet_mock() {
        let result = std::panic::catch_unwind(|| {
            create_verifier("mainnet", true);
        });
        assert!(result.is_err());
    }

    #[test]
    fn create_verifier_returns_schnorr() {
        let v = create_verifier("mainnet", false);
        assert!(v.verify_signature("kaspa:test", "hex", "msg").is_err()); // Schnorr rejects bad data
    }

    #[test]
    fn create_verifier_returns_mock() {
        let v = create_verifier("testnet-12", true);
        assert!(v.verify_signature("kaspa:test", "anything", "msg").is_ok());
    }

    #[test]
    fn script_hash_address_rejected() {
        let verifier = SchnorrVerifier::new();
        // ScriptHash address (version 8) — should be rejected since we need PubKey (version 0)
        let result =
            verifier.verify_signature("kaspatest:pq99546ray", "ab".repeat(64).as_str(), "test");
        assert!(result.is_err());
    }
}
