//! On-chain verification module for DagLock escrows.
//!
//! Provides async verification of UTXO existence via:
//! - wRPC connection to a Kaspa node (`WrpcVerifier`)
//! - Kaspa community REST API (`RestVerifier`) — no local node needed
//! - Mock verifier for testing (`MockVerifier`)
use std::sync::Arc;

use async_trait::async_trait;
use tracing::{info, warn};

use crate::types::Escrow;

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
/// - `WrpcVerifier`: Uses wRPC client to verify on-chain via `get_utxos_by_addresses()`
#[async_trait]
pub trait EscrowVerifier: Send + Sync {
    /// Verify that the escrow UTXO exists on-chain.
    async fn verify_utxo_exists(&self, escrow: &Escrow) -> VerificationResult<bool>;
}

/// Mock verifier that always returns success.
///
/// Use for testing or when on-chain verification is not available.
/// WARNING: Do not use in production — provides no actual security.
pub struct MockVerifier;

#[async_trait]
impl EscrowVerifier for MockVerifier {
    async fn verify_utxo_exists(&self, _escrow: &Escrow) -> VerificationResult<bool> {
        warn!("Using MockVerifier — no actual UTXO verification performed");
        Ok(true)
    }
}

/// Verify escrow is in an active state and its UTXO exists on-chain.
///
/// Checks:
/// 1. Escrow is in an active state (active or pending_confirmation)
/// 2. UTXO exists on-chain (via verifier)
pub async fn verify_escrow_active(
    escrow: &Escrow,
    verifier: &dyn EscrowVerifier,
) -> VerificationResult<()> {
    // Check status
    if !matches!(
        escrow.status,
        crate::types::EscrowStatus::Active | crate::types::EscrowStatus::PendingConfirmation
    ) {
        return Err(VerificationError::Other(format!(
            "Escrow is not in active state: {:?}",
            escrow.status
        )));
    }

    // Verify UTXO exists on-chain
    if !verifier.verify_utxo_exists(escrow).await? {
        return Err(VerificationError::UtxoNotFound {
            tx_id: escrow.lock_tx_id.clone(),
            output_index: escrow.lock_tx_output_index,
        });
    }

    Ok(())
}

// ── wRPC-based Verifier ──────────────────────────────────────────────

/// Real verifier that checks UTXO existence via wRPC connection to a Kaspa node.
///
/// Uses `get_utxos_by_addresses()` to check if the escrow's lock transaction
/// UTXO is still unspent on-chain. The Kaspa node must have `--utxoindex` enabled.
pub struct WrpcVerifier {
    client: Option<Arc<kaspa_wrpc_client::KaspaRpcClient>>,
}

impl WrpcVerifier {
    pub fn new(client: Option<Arc<kaspa_wrpc_client::KaspaRpcClient>>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl EscrowVerifier for WrpcVerifier {
    async fn verify_utxo_exists(&self, escrow: &Escrow) -> VerificationResult<bool> {
        match &self.client {
            Some(client) => {
                info!(
                    "WrpcVerifier: checking UTXO for escrow {} (tx: {}, output: {})",
                    escrow.id, escrow.lock_tx_id, escrow.lock_tx_output_index
                );

                // Build the outpoint from the lock transaction

                use kaspa_wrpc_client::prelude::RpcApi;

                let tx_id_hex = &escrow.lock_tx_id;
                let output_index = escrow.lock_tx_output_index;

                // Decode the tx id from hex
                let tx_id_bytes = match hex::decode(tx_id_hex) {
                    Ok(bytes) if bytes.len() == 32 => {
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(&bytes);
                        arr
                    }
                    _ => {
                        warn!(
                            "WrpcVerifier: invalid tx_id hex for escrow {}: {}",
                            escrow.id, tx_id_hex
                        );
                        return Ok(false);
                    }
                };

                // Build the outpoint request using TransactionId
                let tx_id = kaspa_hashes::Hash::from_bytes(tx_id_bytes);

                // Use get_utxos_by_addresses to find the UTXO.
                // We need the address from the escrow's buyer address to query.
                // Parse the address
                let address: kaspa_addresses::Address =
                    match escrow.buyer_address.as_str().try_into() {
                        Ok(addr) => addr,
                        Err(_) => {
                            warn!(
                                "WrpcVerifier: invalid buyer address for escrow {}: {}",
                                escrow.id, escrow.buyer_address
                            );
                            return Ok(false);
                        }
                    };

                // Query UTXOs for the buyer's address
                match client.get_utxos_by_addresses(vec![address]).await {
                    Ok(utxos) => {
                        // Look for the specific UTXO by matching outpoint
                        for utxo in &utxos {
                            // Check if this UTXO's outpoint matches our lock tx
                            let outpoint = &utxo.outpoint;
                            let is_match =
                                outpoint.transaction_id == tx_id && outpoint.index == output_index;

                            if is_match {
                                let amount = utxo.utxo_entry.amount;
                                info!(
                                    "WrpcVerifier: UTXO found for escrow {} — amount: {}",
                                    escrow.id, amount
                                );
                                return Ok(true);
                            }
                        }

                        warn!(
                            "WrpcVerifier: UTXO NOT found for escrow {} — tx:{}:{} not in address UTXO set",
                            escrow.id, tx_id_hex, output_index
                        );
                        Ok(false)
                    }
                    Err(e) => {
                        // If the node doesn't have UTXO index, try a different approach
                        warn!(
                            "WrpcVerifier: get_utxos_by_addresses failed for escrow {}: {}. \
                             The Kaspa node may need --utxoindex enabled.",
                            escrow.id, e
                        );
                        Err(VerificationError::Other(format!(
                            "UTXO query failed: {e}. Ensure Kaspa node has --utxoindex enabled."
                        )))
                    }
                }
            }
            None => {
                warn!("WrpcVerifier: no wRPC client available — failing UTXO check");
                Err(VerificationError::Other(
                    "No wRPC client connected. Cannot verify UTXO existence.".to_string(),
                ))
            }
        }
    }
}

// ── REST API-based Verifier ──────────────────────────────────────────

/// Verifier that uses the Kaspa community REST API to check UTXO existence.
///
/// Queries `{api_url}/addresses/{address}/utxos` (the Kaspa public REST API)
/// and checks if the escrow's lock transaction UTXO is still unspent.
///
/// This is the recommended verifier for setups without a local Kaspa node.
/// The Kaspa community API is best-effort (no SLA) but works well for
/// testnet and moderate mainnet usage.
///
/// # Examples
/// ```ignore
/// // Mainnet (default):
/// let verifier = RestVerifier::new("https://api.kaspa.org");
///
/// // Testnet-11:
/// let verifier = RestVerifier::new("https://api-tn11.kaspa.org");
/// ```
pub struct RestVerifier {
    api_url: String,
    client: reqwest::Client,
}

impl RestVerifier {
    pub fn new(api_url: impl Into<String>) -> Self {
        Self {
            api_url: api_url.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl EscrowVerifier for RestVerifier {
    async fn verify_utxo_exists(&self, escrow: &Escrow) -> VerificationResult<bool> {
        info!(
            "RestVerifier: checking UTXO for escrow {} (tx: {}, output: {})",
            escrow.id, escrow.lock_tx_id, escrow.lock_tx_output_index
        );

        // Determine which address to query. Try buyer_address first, then seller.
        let address = if !escrow.buyer_address.is_empty() {
            &escrow.buyer_address
        } else if let Some(ref seller) = escrow.seller_address {
            seller
        } else {
            warn!("RestVerifier: no address available for escrow {}", escrow.id);
            return Ok(false);
        };

        // Build the API URL
        let url = format!("{}/addresses/{}/utxos", self.api_url, address);

        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        // The response is an array of UTXOs
                        // Each UTXO has: { "outpoint": { "transactionId": "...", "index": N }, ... }
                        let utxos = match data.as_array() {
                            Some(arr) => arr,
                            None => {
                                warn!("RestVerifier: unexpected response format for escrow {}", escrow.id);
                                return Ok(false);
                            }
                        };

                        let tx_id = &escrow.lock_tx_id;
                        let output_index = escrow.lock_tx_output_index;

                        for utxo in utxos {
                            if let Some(outpoint) = utxo.get("outpoint") {
                                let utxo_tx_id = outpoint
                                    .get("transactionId")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let utxo_index = outpoint
                                    .get("index")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(u64::MAX) as u32;

                                // Compare case-insensitively (API returns lowercase hex)
                                if utxo_tx_id.eq_ignore_ascii_case(tx_id) && utxo_index == output_index
                                {
                                    let amount = utxo
                                        .get("utxoEntry")
                                        .and_then(|e| e.get("amount"))
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0);
                                    info!(
                                        "RestVerifier: UTXO found for escrow {} — amount: {}",
                                        escrow.id, amount
                                    );
                                    return Ok(true);
                                }
                            }
                        }

                        warn!(
                            "RestVerifier: UTXO NOT found for escrow {} — tx:{}:{} not in address UTXO set",
                            escrow.id, tx_id, output_index
                        );
                        Ok(false)
                    }
                    Err(e) => {
                        warn!("RestVerifier: failed to parse response for escrow {}: {}", escrow.id, e);
                        Err(VerificationError::Other(format!(
                            "REST API response parse failed: {e}"
                        )))
                    }
                }
            }
            Ok(response) => {
                warn!(
                    "RestVerifier: API returned {} for escrow {}",
                    response.status(),
                    escrow.id
                );
                Err(VerificationError::Other(format!(
                    "Kaspa API returned {}",
                    response.status()
                )))
            }
            Err(e) => {
                warn!("RestVerifier: request failed for escrow {}: {}", escrow.id, e);
                Err(VerificationError::Other(format!(
                    "REST API request failed: {e}"
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn test_escrow(status: EscrowStatus) -> Escrow {
        Escrow {
            id: "esc_test".to_string(),
            lock_tx_id: "ab".repeat(32),
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

    #[tokio::test]
    async fn mock_verifier_always_succeeds() {
        let verifier = MockVerifier;
        let escrow = test_escrow(EscrowStatus::Active);
        assert!(verifier.verify_utxo_exists(&escrow).await.unwrap());
    }

    #[tokio::test]
    async fn verify_escrow_active_with_active_escrow() {
        let verifier = MockVerifier;
        let escrow = test_escrow(EscrowStatus::Active);
        assert!(verify_escrow_active(&escrow, &verifier).await.is_ok());
    }

    #[tokio::test]
    async fn verify_escrow_active_fails_for_settled() {
        let verifier = MockVerifier;
        let escrow = test_escrow(EscrowStatus::Settled);
        assert!(verify_escrow_active(&escrow, &verifier).await.is_err());
    }

    #[tokio::test]
    async fn verify_escrow_active_fails_for_expired() {
        let verifier = MockVerifier;
        let escrow = test_escrow(EscrowStatus::Expired);
        assert!(verify_escrow_active(&escrow, &verifier).await.is_err());
    }
}
