//! wRPC block listener for DagLock UTXO detection.
//!
//! Subscribes to BlockAdded notifications from the Kaspa node,
//! scans transaction outputs for DagLock template hashes (KAS + KRC-20),
//! and inserts detected escrows into the database.

use sqlx::{Pool, Sqlite};
use tokio::time::interval;
use std::time::Duration;
use tracing::{info, warn};

use crate::db::queries;


/// Spawn the wRPC block listener background task.
///
/// Connects to the Kaspa node, subscribes to BlockAdded notifications,
/// and scans for DagLock UTXOs by matching template hashes.
pub fn spawn(
    wrpc_url: String,
    db: Pool<Sqlite>,
    network: String,
    daglock_kas_template: Option<String>,
    daglock_krc20_template: Option<String>,
) {
    tokio::spawn(async move {
        info!("wRPC listener starting for {network} at {wrpc_url}");

        // Parse template hashes from hex strings
        let kas_hash = daglock_kas_template.as_ref().and_then(|h| hex::decode(h).ok());
        let krc20_hash = daglock_krc20_template.as_ref().and_then(|h| hex::decode(h).ok());

        if kas_hash.is_none() && krc20_hash.is_none() {
            warn!("No template hashes configured — listener will only run reconciliation");
        }

        // TODO: Full wRPC implementation
        // The rusty-kaspa wRPC client has a complex API with:
        // - KaspaRpcClient::new(encoding, url, resolver, network_id)
        // - client.connect(options).await
        // - client.notification_channel_receiver() -> Receiver<Notification>
        //
        // For now, run reconciliation loop with periodic DAA score fetch.
        // When wRPC is fully integrated:
        // 1. Connect to node
        // 2. Subscribe to BlockAdded
        // 3. For each block, scan outputs for template hashes
        // 4. Insert detected escrows
        // 5. Use block's DAA score for reconciliation

        let mut ticker = interval(Duration::from_secs(30));
        let mut block_count: u64 = 0;

        loop {
            ticker.tick().await;

            // TODO: Replace with actual wRPC connection
            // For now, simulate block processing with reconciliation
            //
            // When wRPC is ready, the flow will be:
            // while let Some(notification) = receiver.recv().await {
            //     match notification {
            //         Notification::BlockAdded(block) => {
            //             let daa_score = block.header.daa_score as i64;
            //             for tx in &block.block.transactions {
            //                 for (idx, output) in tx.outputs.iter().enumerate() {
            //                     if let Some(asset) = check_template(&output.script_publickey.script(), &kas_hash, &krc20_hash) {
            //                         insert_escrow(&db, tx, idx, asset).await;
            //                     }
            //                 }
            //             }
            //             reconcile_expired_escrows(&db, daa_score).await;
            //         }
            //         _ => {}
            //     }
            // }

            // Placeholder: reconcile with DAA score 0 (only expires escrows without expiration set)
            match queries::reconcile_expired_escrows(&db, 0).await {
                Ok(0) => {}
                Ok(count) => info!("Reconciled {count} expired escrow(s)"),
                Err(err) => warn!("Escrow reconciliation failed: {err}"),
            }

            block_count += 1;
            if block_count % 10 == 0 {
                info!("Listener heartbeat: {block_count} cycles completed");
            }
        }
    });
}

/// Check if a script matches any DagLock template hash.
///
/// Returns the asset type ("KAS" or "KRC20") if matched, None otherwise.
#[allow(dead_code)]
fn check_template_match(
    script: &[u8],
    kas_hash: Option<&[u8]>,
    krc20_hash: Option<&[u8]>,
) -> Option<String> {
    // Compute BLAKE2b-160 hash of the script (same as template_parts_and_hash)
    let hash = blake2b_simd::Params::new()
        .hash_length(20)
        .hash(script)
        .as_bytes()
        .to_vec();

    if let Some(kas) = kas_hash {
        if hash == kas {
            return Some("KAS".to_string());
        }
    }

    if let Some(krc20) = krc20_hash {
        if hash == krc20 {
            return Some("KRC20".to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_template_match_returns_none_for_unknown_script() {
        let script = vec![0x01, 0x02, 0x03];
        let result = check_template_match(&script, None, None);
        assert!(result.is_none());
    }

    #[test]
    fn check_template_match_detects_kas_hash() {
        // Create a fake script and compute its hash
        let script = vec![0xaa, 0xbb, 0xcc];
        let hash = blake2b_simd::Params::new()
            .hash_length(20)
            .hash(&script)
            .as_bytes()
            .to_vec();

        let result = check_template_match(&script, Some(&hash), None);
        assert_eq!(result, Some("KAS".to_string()));
    }

    #[test]
    fn check_template_match_detects_krc20_hash() {
        let script = vec![0xdd, 0xee, 0xff];
        let hash = blake2b_simd::Params::new()
            .hash_length(20)
            .hash(&script)
            .as_bytes()
            .to_vec();

        let result = check_template_match(&script, None, Some(&hash));
        assert_eq!(result, Some("KRC20".to_string()));
    }
}
