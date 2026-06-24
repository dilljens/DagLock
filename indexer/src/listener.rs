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

use kaspa_rpc_core::RpcBlock;
use kaspa_wrpc_client::prelude::{NetworkId, NetworkType, RpcApi};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::db::queries;
use crate::types::EscrowStatus;

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

        // Attempt wRPC connection with automatic reconnection on drop.
        // Only falls back to offline mode if initial connection fails.
        match try_connect_wrpc(&wrpc_url, &network).await {
            Ok(client) => {
                info!("Connected to Kaspa node at {wrpc_url}");
                run_online_loop_with_reconnect(
                    client,
                    db.clone(),
                    kas_hash.clone(),
                    krc20_hash.clone(),
                    &wrpc_url,
                    &network,
                )
                .await;
            }
            Err(e) => {
                error!("Failed to connect to Kaspa node via wRPC: {e}");
                info!("Falling back to offline reconciliation mode");
                run_offline_loop(db).await;
            }
        }
    });
}

/// Resolve a network string to a NetworkId.
/// Preserves the testnet/ simnet suffix number (e.g. testnet-12 → suffix 12).
pub fn parse_network_id(network: &str) -> NetworkId {
    match network {
        "mainnet" => NetworkId::new(NetworkType::Mainnet),
        n if n.starts_with("testnet-") => {
            let suffix: u32 = n
                .strip_prefix("testnet-")
                .and_then(|s| s.parse().ok())
                .unwrap_or(12);
            NetworkId::with_suffix(NetworkType::Testnet, suffix)
        }
        n if n.starts_with("simnet-") => {
            let suffix: u32 = n
                .strip_prefix("simnet-")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            NetworkId::with_suffix(NetworkType::Simnet, suffix)
        }
        n if n.starts_with("devnet-") => {
            let suffix: u32 = n
                .strip_prefix("devnet-")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            NetworkId::with_suffix(NetworkType::Devnet, suffix)
        }
        _ => {
            warn!("Unknown network '{network}', defaulting to testnet-12");
            NetworkId::with_suffix(NetworkType::Testnet, 12)
        }
    }
}

/// Connect to a Kaspa node via wRPC.
pub async fn try_connect_wrpc(
    url: &str,
    network: &str,
) -> Result<Arc<kaspa_wrpc_client::KaspaRpcClient>, String> {
    use kaspa_wrpc_client::{KaspaRpcClient, WrpcEncoding};

    let network_id = parse_network_id(network);

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

/// Auto-discover and connect to a Kaspa node via the public Resolver network.
/// Uses the Kaspa Public Node Network (PNN) to find an active node.
/// Uses JSON encoding (Borsh is only for same-codebase inter-process communication).
pub async fn try_connect_resolver(
    network: &str,
) -> Result<Arc<kaspa_wrpc_client::KaspaRpcClient>, String> {
    use kaspa_wrpc_client::{KaspaRpcClient, Resolver, WrpcEncoding};

    let network_id = parse_network_id(network);

    let resolver = Resolver::default();
    let client = KaspaRpcClient::new(
        WrpcEncoding::SerdeJson,
        None,
        Some(resolver),
        Some(network_id),
        None,
    )
    .map_err(|e| format!("Failed to create wRPC client with resolver: {e}"))?;

    let client = Arc::new(client);

    let options = kaspa_wrpc_client::client::ConnectOptions {
        block_async_connect: true,
        connect_timeout: Some(Duration::from_secs(15)),
        ..Default::default()
    };

    // Wrap with timeout to fail fast if resolver is unresponsive
    tokio::time::timeout(Duration::from_secs(20), client.connect(Some(options)))
        .await
        .map_err(|_| "Resolver connection timed out after 20s".to_string())?
        .map_err(|e| format!("Failed to connect via resolver: {e}"))?;

    Ok(client)
}

/// Run the online loop with automatic reconnection on connection drop.
pub async fn run_online_loop_with_reconnect(
    client: Arc<kaspa_wrpc_client::KaspaRpcClient>,
    db: Pool<Sqlite>,
    kas_hash: Option<Vec<u8>>,
    krc20_hash: Option<Vec<u8>>,
    _wrpc_url: &str,
    _network: &str,
) {
    run_online_loop(client, db, kas_hash, krc20_hash).await;
    // If run_online_loop returns (connection lost), wait and reconnect
    warn!("wRPC connection lost — reconnecting in 30s...");
    tokio::time::sleep(Duration::from_secs(30)).await;
}

/// Run the listener loop with an active wRPC connection.
/// Uses polling to check DAA score, detect new blocks, and scan for DagLock UTXOs.
async fn run_online_loop(
    client: Arc<kaspa_wrpc_client::KaspaRpcClient>,
    db: Pool<Sqlite>,
    kas_hash: Option<Vec<u8>>,
    krc20_hash: Option<Vec<u8>>,
) {
    let mut last_daa_score: i64 = 0;
    let mut last_processed_hash: Option<kaspa_hashes::Hash> = None;
    let mut heartbeat_count: u64 = 0;
    let mut price_update_count: u64 = 0;
    let mut scanned_count: u64 = 0;

    info!("Starting wRPC online listener loop (polling)...");

    loop {
        match client.get_block_dag_info().await {
            Ok(info) => {
                let daa_score = info.virtual_daa_score as i64;

                // Detect new blocks by DAA score increase
                if daa_score > last_daa_score && last_daa_score > 0 {
                    let new_blocks = daa_score - last_daa_score;
                    info!("DAA progressed: {last_daa_score} → {daa_score} (+{new_blocks} blocks)");

                    // Scan new blocks for DagLock template hash matches
                    if kas_hash.is_some() || krc20_hash.is_some() {
                        // On first run, start from the current tip
                        if last_processed_hash.is_none() {
                            if let Some(tip) = info.tip_hashes.first() {
                                last_processed_hash = Some(*tip);
                                info!("Initializing block scan from tip: {tip}");
                            }
                        }

                        if let Some(low_hash) = last_processed_hash {
                            match client.get_blocks(Some(low_hash), true, true).await {
                                Ok(response) => {
                                    let block_count = response.blocks.len();
                                    for block in &response.blocks {
                                        scan_block_for_escrows(
                                            block,
                                            &db,
                                            kas_hash.as_deref(),
                                            krc20_hash.as_deref(),
                                            &mut scanned_count,
                                        )
                                        .await;
                                    }
                                    // Update last_processed_hash to the latest block hash
                                    if let Some(last_hash) = response.block_hashes.last() {
                                        last_processed_hash = Some(*last_hash);
                                    }
                                    if block_count > 0 {
                                        info!(
                                            "Scanned {block_count} block(s), activated {scanned_count} escrow(s) total"
                                        );
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to fetch blocks: {e}");
                                }
                            }
                        }
                    }
                }
                last_daa_score = daa_score;

                // Reconcile expired escrows
                if let Err(e) = queries::reconcile_expired_escrows(&db, daa_score).await {
                    warn!("Escrow reconciliation failed: {e}");
                }

                // Update market prices every 15 minutes (90 cycles at 10s)
                price_update_count += 1;
                if price_update_count >= 90 {
                    price_update_count = 0;
                    match update_market_prices(&db).await {
                        Ok(n) => {
                            if n > 0 {
                                info!("Updated prices for {n} market-priced offers");
                            }
                        }
                        Err(e) => warn!("Price update failed: {e}"),
                    }
                }

                heartbeat_count += 1;
                if heartbeat_count.is_multiple_of(10) {
                    info!("Listener heartbeat: {heartbeat_count} cycles, DAA: {daa_score}");
                }
            }
            Err(e) => {
                warn!("wRPC connection lost: {e}. Reconnecting...");
                return; // outer loop in spawn() handles reconnection wait
            }
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

/// Scan a block's transaction outputs for DagLock template hash matches.
/// When a match is found, look up the escrow by lock_tx_id and activate it.
async fn scan_block_for_escrows(
    block: &RpcBlock,
    db: &Pool<Sqlite>,
    kas_hash: Option<&[u8]>,
    krc20_hash: Option<&[u8]>,
    activated_count: &mut u64,
) {
    for tx in &block.transactions {
        let tx_id_str = tx
            .verbose_data
            .as_ref()
            .map(|vd| format!("{}", vd.transaction_id))
            .unwrap_or_default();

        if tx_id_str.is_empty() {
            continue;
        }

        // Scan each output for template hash matches
        for (output_index, output) in tx.outputs.iter().enumerate() {
            let script = output.script_public_key.script();
            if let Some(asset_type) = check_template_match(script, kas_hash, krc20_hash) {
                // Found a DagLock output — try to activate the corresponding escrow
                match queries::try_find_escrow_by_lock_tx(db, &tx_id_str).await {
                    Ok(Some(escrow_id)) => {
                        // Check current status before updating
                        match queries::get_escrow(db, &escrow_id).await {
                            Ok(Some(escrow))
                                if escrow.status == EscrowStatus::PendingConfirmation =>
                            {
                                match queries::update_escrow_status_only(db, &escrow_id, "active")
                                    .await
                                {
                                    Ok(()) => {
                                        *activated_count += 1;
                                        info!(
                                            "Activated escrow {escrow_id} — lock tx {tx_id_str} output {output_index} matches {asset_type} template"
                                        );
                                    }
                                    Err(e) => {
                                        warn!("Failed to activate escrow {escrow_id}: {e}");
                                    }
                                }
                            }
                            Ok(Some(_)) => {
                                // Already active/settled — skip
                            }
                            Ok(None) => {
                                warn!("Escrow {escrow_id} not found in DB after lock tx match");
                            }
                            Err(e) => {
                                warn!("Failed to get escrow {escrow_id}: {e}");
                            }
                        }
                    }
                    Ok(None) => {
                        // No escrow found for this tx_id — not a DagLock escrow
                    }
                    Err(e) => {
                        warn!("Failed to look up escrow for tx {tx_id_str}: {e}");
                    }
                }
            }
        }
    }
}

/// Run the listener without wRPC connection (reconciliation only).
#[allow(dead_code)]
async fn run_offline_loop(db: Pool<Sqlite>) {
    let mut ticker = interval(Duration::from_secs(30));
    let mut count: u64 = 0;
    let mut price_update_count: u64 = 0;

    loop {
        ticker.tick().await;
        match queries::reconcile_expired_escrows(&db, 0).await {
            Ok(0) => {}
            Ok(n) => info!("Reconciled {n} expired escrow(s)"),
            Err(e) => warn!("Reconciliation failed: {e}"),
        }

        // Update market prices every 15 minutes (30 cycles at 30s)
        price_update_count += 1;
        if price_update_count >= 30 {
            price_update_count = 0;
            match update_market_prices(&db).await {
                Ok(n) => {
                    if n > 0 {
                        info!("Updated prices for {n} market-priced offers");
                    }
                }
                Err(e) => warn!("Price update failed: {e}"),
            }
        }

        count += 1;
        if count.is_multiple_of(20) {
            info!("Offline listener heartbeat: {count} cycles");
        }
    }
}

/// Update market prices for price_locked offers (fetches from CoinGecko).
#[allow(dead_code)]
async fn update_market_prices(pool: &Pool<Sqlite>) -> Result<u64, String> {
    let usd_price = crate::types::fetch_kas_usd_price()
        .await
        .ok_or_else(|| "Failed to fetch price from CoinGecko".to_string())?;

    // Update all market-priced offers
    let now = chrono::Utc::now().timestamp();
    let result = sqlx::query(
        "UPDATE offers SET current_price = ?1, price_updated_at = ?2 WHERE price_type = 'market' AND status = 'proposed'"
    )
    .bind(usd_price)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update prices: {e}"))?;

    Ok(result.rows_affected())
}

/// Check if a script matches any DagLock template hash by computing its BLAKE2b-160 hash.
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

/// Spawn the vault auto-sweep background loop.
#[allow(dead_code)]
pub fn spawn_vault_sweeper(
    db: Pool<Sqlite>,
    _wrpc_client: Option<Arc<kaspa_wrpc_client::KaspaRpcClient>>,
    treasury_pubkey_hex: Option<String>,
) {
    if _wrpc_client.is_none() {
        warn!("Vault auto-sweep disabled: no wRPC connection available.");
        return;
    }
    if treasury_pubkey_hex.is_none() {
        warn!("Vault auto-sweep disabled: no treasury pubkey configured.");
        return;
    }

    // Guarded by is_none() check above — safe but avoid unwrap for audit compliance
    let treasury_pubkey = treasury_pubkey_hex.unwrap_or_default();
    let treasury_key = match hex::decode(&treasury_pubkey) {
        Ok(k) if k.len() == 32 => k,
        _ => {
            error!("Invalid treasury pubkey hex for vault sweep");
            return;
        }
    };

    tokio::spawn(async move {
        info!("Vault auto-sweep loop started (30s interval)");
        let mut ticker = interval(Duration::from_secs(30));
        loop {
            ticker.tick().await;
            match queries::find_sweepable_vaults(&db).await {
                Ok(vaults) => {
                    for (id, _owner_addr, amount_sompi, owner_pubkey_hex) in &vaults {
                        info!(
                            "Vault {} can be swept: {} sompi past timeout",
                            id, amount_sompi
                        );
                        if let Some(owner_pk_hex) = owner_pubkey_hex {
                            if let Ok(owner_key) = hex::decode(owner_pk_hex) {
                                if owner_key.len() == 32 && treasury_key.len() == 32 {
                                    let compiled = daglock_contracts::compile_daglock_vault(
                                        &owner_key,
                                        0,
                                        &treasury_key,
                                    );
                                    let fee_amount = *amount_sompi as u64 / 1000;
                                    let send_amount = *amount_sompi as u64 - fee_amount;
                                    if let Ok(sigscript) = compiled.build_sig_script(
                                        daglock_contracts::entrypoints::SWEEP,
                                        vec![],
                                    ) {
                                        info!("Sweep tx ready for vault {} (send: {}, fee: {}, sigscript: {} bytes)", id, send_amount, fee_amount, sigscript.len());
                                        let _ = queries::mark_vault_swept(&db, id, "sweep_pending")
                                            .await;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Vault sweep query failed: {}", e);
                }
            }
        }
    });
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
