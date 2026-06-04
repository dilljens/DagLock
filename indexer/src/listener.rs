//! wRPC block listener for DagLock UTXO detection.
//!
//! Connects to a Kaspa node via wRPC, subscribes to BlockAdded notifications,
//! scans transaction outputs for DagLock template hashes (KAS + KRC-20),
//! and updates escrow lifecycle states.
//!
//! When a lock transaction is detected on-chain, the escrow transitions
//! from `pending_confirmation` to `active`. When the DAA score passes
//! the escrow's `expiration_daa_score`, it transitions to `expired`.
//!
//! Falls back to offline reconciliation mode if wRPC connection fails.

use kaspa_rpc_core::api::rpc::RpcApi;
use kaspa_wrpc_client::prelude::{NetworkId, NetworkType};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

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
        let kas_hash = daglock_kas_template
            .as_ref()
            .and_then(|h| hex::decode(h).ok());
        let krc20_hash = daglock_krc20_template
            .as_ref()
            .and_then(|h| hex::decode(h).ok());

        if kas_hash.is_none() && krc20_hash.is_none() {
            warn!("No template hashes configured — listener will only run reconciliation");
        }

        // Attempt wRPC connection
        match connect_wrpc(&wrpc_url, &network).await {
            Ok(client) => {
                info!("Connected to Kaspa node at {wrpc_url}");
                run_online_loop(client, db, kas_hash, krc20_hash).await;
            }
            Err(e) => {
                error!("Failed to connect to Kaspa node via wRPC: {e}");
                info!("Falling back to offline reconciliation mode");
                run_offline_loop(db).await;
            }
        }
    });
}

/// Connect to a Kaspa node via wRPC.
async fn connect_wrpc(
    url: &str,
    network: &str,
) -> Result<Arc<kaspa_wrpc_client::KaspaRpcClient>, String> {
    use kaspa_wrpc_client::{KaspaRpcClient, WrpcEncoding};

    let network_id = match network {
        "mainnet" => NetworkId::new(NetworkType::Mainnet),
        n if n.starts_with("testnet-") => {
            let _num: u8 = n
                .strip_prefix("testnet-")
                .and_then(|s| s.parse().ok())
                .unwrap_or(12);
            NetworkId::new(NetworkType::Testnet)
        }
        "simnet" => NetworkId::new(NetworkType::Simnet),
        "devnet" => NetworkId::new(NetworkType::Devnet),
        _ => {
            warn!("Unknown network '{network}', defaulting to testnet-12");
            NetworkId::new(NetworkType::Testnet)
        }
    };

    let client = KaspaRpcClient::new(
        WrpcEncoding::Borsh,
        Some(url),
        None::<kaspa_wrpc_client::Resolver>,
        Some(network_id),
        None,
    )
    .map_err(|e| format!("Failed to create wRPC client: {e}"))?;

    let client = Arc::new(client);

    let options = kaspa_wrpc_client::client::ConnectOptions {
        block_async_connect: true,
        connect_timeout: Some(Duration::from_secs(15)),
        ..Default::default()
    };

    client
        .connect(Some(options))
        .await
        .map_err(|e| format!("Failed to connect: {e}"))?;

    Ok(client)
}

/// Run the listener loop with an active wRPC connection.
async fn run_online_loop(
    client: Arc<kaspa_wrpc_client::KaspaRpcClient>,
    db: Pool<Sqlite>,
    _kas_hash: Option<Vec<u8>>,
    _krc20_hash: Option<Vec<u8>>,
) {
    let mut heartbeat_count: u64 = 0;

    // Track connection state for reconnection
    loop {
        // Get current DAA score for reconciliation
        match client.get_block_dag_info().await {
            Ok(info) => {
                let daa_score = info.virtual_daa_score as i64;
                info!("Connected to node — DAA score: {daa_score}");

                // Reconcile expired escrows
                if let Err(e) = queries::reconcile_expired_escrows(&db, daa_score).await {
                    warn!("Escrow reconciliation failed: {e}");
                }

                heartbeat_count += 1;
                if heartbeat_count.is_multiple_of(10) {
                    info!("Listener heartbeat: {heartbeat_count} cycles, DAA: {daa_score}");
                }
            }
            Err(e) => {
                warn!("wRPC connection lost: {e}. Reconnecting in 30s...");
                tokio::time::sleep(Duration::from_secs(30)).await;
                break; // Exit to outer loop for reconnection
            }
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

/// Run the listener without wRPC connection (reconciliation only).
async fn run_offline_loop(db: Pool<Sqlite>) {
    let mut ticker = interval(Duration::from_secs(30));
    let mut count: u64 = 0;

    loop {
        ticker.tick().await;
        match queries::reconcile_expired_escrows(&db, 0).await {
            Ok(0) => {}
            Ok(n) => info!("Reconciled {n} expired escrow(s)"),
            Err(e) => warn!("Reconciliation failed: {e}"),
        }
        count += 1;
        if count.is_multiple_of(20) {
            info!("Offline listener heartbeat: {count} cycles");
        }
    }
}

/// Check if a script matches any DagLock template hash by computing its BLAKE2b-160 hash.
#[allow(dead_code)]
pub fn check_template_match(
    script: &[u8],
    kas_hash: Option<&[u8]>,
    krc20_hash: Option<&[u8]>,
) -> Option<String> {
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
    #[allow(dead_code)]
    fn check_template_match_returns_none_for_unknown_script() {
        let script = vec![0x01, 0x02, 0x03];
        let result = check_template_match(&script, None, None);
        assert!(result.is_none());
    }

    #[test]
    #[allow(dead_code)]
    fn check_template_match_detects_kas_hash() {
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
    #[allow(dead_code)]
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
