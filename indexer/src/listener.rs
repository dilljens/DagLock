//! Background reconciliation loop for the indexer.
//!
//! Currently runs reconciliation every 30 seconds using a simulated DAA score.
//! Full wRPC BlockAdded subscription will be implemented when the wRPC client
//! API is stabilized for the tn12 branch.

use std::time::Duration;

use sqlx::{Pool, Sqlite};
use tokio::time::interval;
use tracing::{info, warn};

use crate::db::queries;

/// Spawn the background listener/reconciliation loop.
///
/// # Current behavior
/// - Runs reconciliation every 30 seconds
/// - Uses a simulated DAA score (0) to avoid premature expiration
///
/// # TODO: Full wRPC implementation
/// - Connect to Kaspa node via wRPC
/// - Subscribe to BlockAdded notifications
/// - Scan transaction outputs for DagLock template hashes
/// - Insert detected escrows into database
/// - Use actual DAA score from block headers for reconciliation
pub fn spawn(
    wrpc_url: String,
    db: Pool<Sqlite>,
    network: String,
    _daglock_kas_template: Option<String>,
    _daglock_krc20_template: Option<String>,
) {
    tokio::spawn(async move {
        info!("Listener starting for {network} at {wrpc_url}");

        // TODO: Connect to wRPC node
        // TODO: Subscribe to BlockAdded notifications
        // TODO: Scan for DagLock template hashes
        // For now, just run reconciliation loop

        let mut ticker = interval(Duration::from_secs(30));
        loop {
            ticker.tick().await;

            // TODO: Fetch current DAA score from wRPC node
            // When wRPC is fully implemented: let current_daa = get_daa_score_from_node().await;
            // For now, pass 0 to only expire escrows that have no expiration set
            // (which won't match any real escrows since they all have expiration_daa_score)
            let current_daa_score: i64 = 0;

            match queries::reconcile_expired_escrows(&db, current_daa_score).await {
                Ok(0) => {}
                Ok(count) => info!("Reconciled {count} expired escrow(s)"),
                Err(err) => warn!("Escrow reconciliation failed: {err}"),
            }
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn listener_module_compiles() {
        // Basic smoke test - ensure the module compiles
        assert!(true);
    }
}
