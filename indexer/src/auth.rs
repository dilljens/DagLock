#![allow(dead_code)]
//! Authentication module for DagLock API.
//!
//! Two modes:
//!   - `MockVerifier` — any hex string passes (dev/testnet, default)
//!   - `SchnorrVerifier` — real Kaspa Schnorr signature verification
//!
//! Mock auth is rejected on mainnet via a startup safety check.
//!
//! # Replay Protection
//!
//! Message format (version 2, with replay protection):
//!   `{action}:{escrow_id}:{timestamp}:{nonce_hex}`
//!
//! Message format (version 1, backward compatible):
//!   `{action}:{escrow_id}`
//!
//! Version 2 messages include a Unix timestamp (±5 min window) and a
//! 20-byte BLAKE2b-160 nonce (40 hex chars). Nonces are stored in the
//! database to prevent replay. Version 1 messages skip replay checks.

use crate::types::Escrow;

use kaspa_addresses::{Address, Version};
use kaspa_hashes::{Hash, PersonalMessageSigningHash};
use secp256k1::XOnlyPublicKey;
use sqlx::{Pool, Sqlite};
use tracing::warn;

/// Max clock drift for nonce timestamps in seconds (5 minutes).
const MAX_CLOCK_DRIFT_SECONDS: i64 = 300;

/// Length of nonce in bytes (BLAKE2b-160 = 20 bytes).
const NONCE_LENGTH: usize = 20;

/// Length of nonce hex string (40 chars).
const NONCE_HEX_LENGTH: usize = 40;

/// Errors that can occur during authentication.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing required header: {header}")]
    MissingHeader { header: String },

    #[error("Invalid signature for address {address}")]
    InvalidSignature { address: String },

    #[error("Unauthorized: {reason}")]
    Unauthorized { reason: String },

    #[error("Replay detected: nonce already used for {action}:{escrow_id}")]
    ReplayDetected { action: String, escrow_id: String },

    #[error("Invalid message format: {detail}")]
    InvalidMessage { detail: String },

    #[error("Timestamp outside allowed window: {timestamp} (now: {now}, drift: {drift}s)")]
    TimestampOutOfWindow {
        timestamp: i64,
        now: i64,
        drift: i64,
    },
}

/// Result type for authentication operations.
pub type AuthResult<T> = Result<T, AuthError>;

/// Parsed auth message with replay protection data.
#[derive(Debug)]
pub struct ParsedMessage {
    /// The action (settle, refund, dispute, cancel)
    pub action: String,
    /// The escrow ID
    pub escrow_id: String,
    /// The original message string
    pub full_message: String,
    /// Replay protection nonce (20 bytes) — None if using legacy format
    pub nonce: Option<Vec<u8>>,
    /// Unix timestamp from message — None if using legacy format
    pub timestamp: Option<i64>,
}

/// Try to parse a version 2 message: `action:id:ts:nonce_hex`
/// or fall back to version 1: `action:id`
pub(crate) fn parse_message(message: &str) -> AuthResult<ParsedMessage> {
    let parts: Vec<&str> = message.split(':').collect();

    if parts.len() == 4 {
        // Version 2: action:id:timestamp:nonce_hex
        let action = parts[0].to_string();
        let escrow_id = parts[1].to_string();

        // Validate action
        match action.as_str() {
            "settle" | "refund" | "dispute" | "cancel" | "evidence" | "vote" | "vouch"
            | "messages" => {}
            _ => {
                return Err(AuthError::InvalidMessage {
                    detail: format!("Unknown action: {action}"),
                });
            }
        }

        // Parse timestamp
        let timestamp: i64 = parts[2].parse().map_err(|_| AuthError::InvalidMessage {
            detail: format!("Invalid timestamp: {}", parts[2]),
        })?;

        // Check timestamp within window
        let now = chrono::Utc::now().timestamp();
        let drift = (now - timestamp).abs();
        if drift > MAX_CLOCK_DRIFT_SECONDS {
            return Err(AuthError::TimestampOutOfWindow {
                timestamp,
                now,
                drift,
            });
        }

        // Validate nonce hex (must be 40 hex chars = 20 bytes)
        let nonce_hex = parts[3];
        if nonce_hex.len() != NONCE_HEX_LENGTH {
            return Err(AuthError::InvalidMessage {
                detail: format!(
                    "Nonce must be {NONCE_HEX_LENGTH} hex chars, got {}",
                    nonce_hex.len()
                ),
            });
        }

        let nonce = hex::decode(nonce_hex).map_err(|_| AuthError::InvalidMessage {
            detail: "Nonce is not valid hex".to_string(),
        })?;

        Ok(ParsedMessage {
            action,
            escrow_id,
            full_message: message.to_string(),
            nonce: Some(nonce),
            timestamp: Some(timestamp),
        })
    } else if parts.len() == 2 {
        // Version 1 (legacy): action:id
        let action = parts[0].to_string();
        let escrow_id = parts[1].to_string();
        Ok(ParsedMessage {
            action,
            escrow_id,
            full_message: message.to_string(),
            nonce: None,
            timestamp: None,
        })
    } else {
        Err(AuthError::InvalidMessage {
            detail: format!(
                "Expected format 'action:id' or 'action:id:timestamp:nonce', got '{}' with {} parts",
                message,
                parts.len()
            ),
        })
    }
}

/// Verify the nonce against the DB (stored or already used).
pub(crate) async fn verify_nonce(
    pool: &Pool<Sqlite>,
    parsed: &ParsedMessage,
    address: &str,
) -> AuthResult<()> {
    if let Some(ref nonce) = parsed.nonce {
        let action = &parsed.action;
        let escrow_id = &parsed.escrow_id;
        let timestamp = parsed.timestamp.unwrap_or(0);

        // Check if nonce already exists (replay attack)
        let exists = crate::db::queries::check_auth_nonce_exists(pool, nonce)
            .await
            .map_err(|e| AuthError::Unauthorized {
                reason: format!("Nonce check failed: {e}"),
            })?;

        if exists {
            return Err(AuthError::ReplayDetected {
                action: action.clone(),
                escrow_id: escrow_id.clone(),
            });
        }

        // Store the nonce
        crate::db::queries::store_auth_nonce(pool, nonce, action, escrow_id, address, timestamp)
            .await
            .map_err(|e| AuthError::Unauthorized {
                reason: format!("Failed to store nonce: {e}"),
            })?;
    }
    // Legacy format (no nonce) — skip replay check
    Ok(())
}

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
    /// - `X-Daglock-Message`: The signed message (format: "action:id" or "action:id:ts:nonce")
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
/// - Message format: "settle:escrow_id" (v1) or "settle:escrow_id:ts:nonce" (v2)
pub async fn verify_settle_authorization(
    escrow: &Escrow,
    auth: &AuthContext,
    verifier: &dyn SignatureVerifier,
    pool: &Pool<Sqlite>,
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

    // Parse and validate the message
    let parsed = parse_message(&auth.message)?;

    // Verify action matches
    if parsed.action != "settle" {
        return Err(AuthError::InvalidMessage {
            detail: format!("Expected action 'settle', got '{}'", parsed.action),
        });
    }

    // Verify escrow_id matches
    if parsed.escrow_id != escrow.id {
        return Err(AuthError::Unauthorized {
            reason: format!(
                "Message escrow_id '{}' does not match request '{}'",
                parsed.escrow_id, escrow.id
            ),
        });
    }

    // Verify signature
    if !verifier.verify_signature(&auth.address, &auth.signature, &auth.message)? {
        return Err(AuthError::InvalidSignature {
            address: auth.address.clone(),
        });
    }

    // Replay protection
    verify_nonce(pool, &parsed, &auth.address).await?;

    Ok(())
}

/// Verify that the caller is authorized to refund the escrow.
///
/// # Authorization Rules
/// - Only the buyer can refund (they deposited the funds)
/// - The caller must prove ownership by signing a message
/// - Message format: "refund:escrow_id" (v1) or "refund:escrow_id:ts:nonce" (v2)
pub async fn verify_refund_authorization(
    escrow: &Escrow,
    auth: &AuthContext,
    verifier: &dyn SignatureVerifier,
    pool: &Pool<Sqlite>,
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

    // Parse and validate the message
    let parsed = parse_message(&auth.message)?;

    // Verify action matches
    if parsed.action != "refund" {
        return Err(AuthError::InvalidMessage {
            detail: format!("Expected action 'refund', got '{}'", parsed.action),
        });
    }

    // Verify escrow_id matches
    if parsed.escrow_id != escrow.id {
        return Err(AuthError::Unauthorized {
            reason: format!(
                "Message escrow_id '{}' does not match request '{}'",
                parsed.escrow_id, escrow.id
            ),
        });
    }

    // Verify signature
    if !verifier.verify_signature(&auth.address, &auth.signature, &auth.message)? {
        return Err(AuthError::InvalidSignature {
            address: auth.address.clone(),
        });
    }

    // Replay protection
    verify_nonce(pool, &parsed, &auth.address).await?;

    Ok(())
}

/// Extract auth context from headers and verify a signature.
/// Returns Ok(AuthContext) if the signature is valid.

/// Verify that the caller is authorized to cancel the escrow.
pub async fn verify_cancel_authorization(
    escrow: &Escrow,
    auth: &AuthContext,
    verifier: &dyn SignatureVerifier,
    pool: &Pool<Sqlite>,
) -> AuthResult<()> {
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
    let parsed = parse_message(&auth.message)?;
    if parsed.action != "cancel" {
        return Err(AuthError::InvalidMessage {
            detail: format!("Expected action 'cancel', got '{}'", parsed.action),
        });
    }
    if parsed.escrow_id != escrow.id {
        return Err(AuthError::Unauthorized {
            reason: format!(
                "Message escrow_id '{}' does not match request '{}'",
                parsed.escrow_id, escrow.id
            ),
        });
    }
    if !verifier.verify_signature(&auth.address, &auth.signature, &auth.message)? {
        return Err(AuthError::InvalidSignature {
            address: auth.address.clone(),
        });
    }
    verify_nonce(pool, &parsed, &auth.address).await?;
    Ok(())
}

/// Generate a nonce for replay-protected messages.
/// Returns a hex-encoded 20-byte BLAKE2b-160 hash.
pub fn generate_nonce() -> String {
    let random_bytes = rand::random::<[u8; 16]>();
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let input = [&timestamp.to_le_bytes()[..], &random_bytes[..]].concat();
    let hash = blake2b_simd::Params::new()
        .hash_length(NONCE_LENGTH)
        .hash(&input);
    hex::encode(hash.as_bytes())
}

/// Generate a replay-protected message.
///
/// # Arguments
/// * `action` - The action (settle, refund, dispute, cancel)
/// * `escrow_id` - The escrow identifier
///
/// # Returns
/// * A message string in format: "action:escrow_id:timestamp:nonce_hex"
pub fn generate_replay_protected_message(action: &str, escrow_id: &str) -> String {
    let timestamp = chrono::Utc::now().timestamp();
    let nonce = generate_nonce();
    format!("{action}:{escrow_id}:{timestamp}:{nonce}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    async fn test_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::db::schema::migrate(&pool).await.unwrap();
        pool
    }

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
            price_type: None,
            invoice_id: None,
            memo: None,
            auto_settle_timeout: None,
            mediation_status: None,
            mediation_buyer_claim: None,
            mediation_seller_claim: None,
            mediation_result: None,
            mediation_expires_at: None,
            mediation_buyer_accepted: None,
            mediation_seller_accepted: None,
            chat_pubkey_buyer: None,
            chat_pubkey_seller: None,
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

        assert!(verifier
            .verify_signature("not-an-address", "abcd", "test")
            .is_err());

        assert!(verifier
            .verify_signature(
                "kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya",
                "not-hex",
                "test"
            )
            .is_err());

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
        let fake_sig = "ab".repeat(64);

        let result = verifier.verify_signature(
            "kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya",
            &fake_sig,
            "test message",
        );
        assert!(result.is_err());
    }

    #[test]
    fn parse_message_v1_legacy_format() {
        let parsed = parse_message("settle:esc_123").unwrap();
        assert_eq!(parsed.action, "settle");
        assert_eq!(parsed.escrow_id, "esc_123");
        assert!(parsed.nonce.is_none());
        assert!(parsed.timestamp.is_none());
    }

    #[test]
    fn parse_message_v2_with_replay_protection() {
        let nonce_hex = generate_nonce();
        assert_eq!(nonce_hex.len(), 40);
        let msg = format!(
            "settle:esc_123:{}:{}",
            chrono::Utc::now().timestamp(),
            nonce_hex
        );
        let parsed = parse_message(&msg).unwrap();
        assert_eq!(parsed.action, "settle");
        assert_eq!(parsed.escrow_id, "esc_123");
        assert!(parsed.nonce.is_some());
        assert_eq!(parsed.nonce.as_ref().unwrap().len(), 20);
        assert!(parsed.timestamp.is_some());
    }

    #[test]
    fn parse_message_invalid_format() {
        // Too few parts
        assert!(parse_message("just_action").is_err());
        // Too many parts (5+)
        assert!(parse_message("a:b:c:d:e").is_err());
    }

    #[test]
    fn parse_message_stale_timestamp() {
        let nonce_hex = generate_nonce();
        let old_ts = chrono::Utc::now().timestamp() - 600; // 10 min ago (outside 5 min window)
        let msg = format!("settle:esc_123:{old_ts}:{nonce_hex}");
        let result = parse_message(&msg);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Timestamp outside allowed window"));
    }

    #[test]
    fn parse_message_invalid_nonce_length() {
        let ts = chrono::Utc::now().timestamp();
        let msg = format!("settle:esc_123:{ts}:aabbcc"); // too short
        let result = parse_message(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn generate_replay_protected_message_creates_valid_format() {
        let msg = generate_replay_protected_message("settle", "esc_123");
        let parsed = parse_message(&msg).unwrap();
        assert_eq!(parsed.action, "settle");
        assert_eq!(parsed.escrow_id, "esc_123");
        assert!(parsed.nonce.is_some());
        assert!(parsed.timestamp.is_some());
    }

    #[tokio::test]
    async fn verify_settle_buyer_authorized() {
        let verifier = MockVerifier::new();
        let escrow = test_escrow();
        let auth = AuthContext {
            address: "kaspatest:qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpk58a75"
                .to_string(),
            signature: "any_hex".to_string(),
            message: "settle:esc_test".to_string(),
        };
        assert!(
            verify_settle_authorization(&escrow, &auth, &verifier, &test_pool().await)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn verify_settle_seller_authorized() {
        let verifier = MockVerifier::new();
        let escrow = test_escrow();
        let auth = AuthContext {
            address: "kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya"
                .to_string(),
            signature: "sig123".to_string(),
            message: "settle:esc_test".to_string(),
        };
        assert!(
            verify_settle_authorization(&escrow, &auth, &verifier, &test_pool().await)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn verify_settle_unauthorized_address() {
        let verifier = MockVerifier::new();
        let escrow = test_escrow();
        let auth = AuthContext {
            address: "kaspa:outsider".to_string(),
            signature: "sig123".to_string(),
            message: "settle:esc_test".to_string(),
        };
        assert!(
            verify_settle_authorization(&escrow, &auth, &verifier, &test_pool().await)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn verify_refund_buyer_only() {
        let verifier = MockVerifier::new();
        let escrow = test_escrow();

        // Buyer can refund
        let buyer_auth = AuthContext {
            address: "kaspatest:qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpk58a75"
                .to_string(),
            signature: "sig123".to_string(),
            message: "refund:esc_test".to_string(),
        };
        assert!(
            verify_refund_authorization(&escrow, &buyer_auth, &verifier, &test_pool().await)
                .await
                .is_ok()
        );

        // Seller cannot refund
        let seller_auth = AuthContext {
            address: "kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya"
                .to_string(),
            signature: "sig123".to_string(),
            message: "refund:esc_test".to_string(),
        };
        assert!(
            verify_refund_authorization(&escrow, &seller_auth, &verifier, &test_pool().await)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn verify_settle_with_replay_protected_message() {
        let verifier = MockVerifier::new();
        let escrow = test_escrow();
        let msg = generate_replay_protected_message("settle", "esc_test");
        let auth = AuthContext {
            address: "kaspatest:qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpk58a75"
                .to_string(),
            signature: "any_hex".to_string(),
            message: msg,
        };
        assert!(
            verify_settle_authorization(&escrow, &auth, &verifier, &test_pool().await)
                .await
                .is_ok()
        );
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
        let v = create_verifier("testnet-11", true);
        assert!(v.verify_signature("kaspa:test", "anything", "msg").is_ok());
    }

    #[test]
    fn script_hash_address_rejected() {
        let verifier = SchnorrVerifier::new();
        let result =
            verifier.verify_signature("kaspatest:pq99546ray", "ab".repeat(64).as_str(), "test");
        assert!(result.is_err());
    }

    #[test]
    fn generate_nonce_produces_20_bytes_hex() {
        let nonce = generate_nonce();
        assert_eq!(nonce.len(), 40); // 20 bytes = 40 hex chars
        assert!(hex::decode(&nonce).is_ok());
    }
}
