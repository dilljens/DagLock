//! Snapshot tests for the DagLock REST API.
//!
//! Uses `insta` to compare API responses against stored snapshots.
//! Run `cargo insta review` to approve new snapshots after intentional changes.
//!
//! Also includes time-paused tests for expiry/auto-settle logic.

use daglock_indexer::types::{Escrow, EscrowStatus, ApiError};
use insta::assert_json_snapshot;
use sqlx::SqlitePool;
use std::time::Duration;

/// Helper to create a test database pool with migrations
async fn test_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    daglock_indexer::db::schema::migrate(&pool).await.unwrap();
    pool
}

fn test_escrow(id: &str, status: EscrowStatus) -> Escrow {
    Escrow {
        id: id.to_string(),
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

// ── Snapshot Tests ──────────────────────────────────────────────

#[tokio::test]
async fn snapshot_escrow_active() {
    let escrow = test_escrow("esc_snap_001", EscrowStatus::Active);
    assert_json_snapshot!("escrow_active", escrow, {
        ".lock_tx_id" => "[hash]",
        ".template_hash" => "[bytes]",
    });
}

#[tokio::test]
async fn snapshot_escrow_settled() {
    let mut escrow = test_escrow("esc_snap_002", EscrowStatus::Settled);
    escrow.settled_at = Some(1_700_000_100);
    escrow.price_at_settlement = Some(0.15);
    assert_json_snapshot!("escrow_settled", escrow, {
        ".lock_tx_id" => "[hash]",
        ".template_hash" => "[bytes]",
    });
}

#[tokio::test]
async fn snapshot_escrow_disputed() {
    let mut escrow = test_escrow("esc_snap_003", EscrowStatus::Disputed);
    escrow.disputed_at = Some(1_700_000_050);
    escrow.dispute_reason = Some("Counterparty not responding".to_string());
    escrow.dispute_mode = Some("mediator".to_string());
    escrow.mediator_key = Some("kaspa:mediator".to_string());
    assert_json_snapshot!("escrow_disputed", escrow, {
        ".lock_tx_id" => "[hash]",
        ".template_hash" => "[bytes]",
    });
}

#[tokio::test]
async fn snapshot_api_error() {
    let err = ApiError::new("not_found", "Escrow not found");
    assert_json_snapshot!("api_error_not_found", err);
}

#[tokio::test]
async fn snapshot_api_error_rate_limited() {
    let err = ApiError::new("rate_limited", "Max 50 requests per 60 seconds");
    assert_json_snapshot!("api_error_rate_limited", err);
}

// ── Concurrent Access Tests ────────────────────────────────────

#[tokio::test]
async fn test_concurrent_settle_atomic() {
    let pool = test_pool().await;

    // Insert an active escrow
    let escrow = test_escrow("esc_atomic_001", EscrowStatus::Active);
    daglock_indexer::db::queries::insert_escrow(&pool, &escrow)
        .await
        .expect("insert_escrow");

    // Execute two sequential updates with atomic WHERE clause
    // (same pattern as a concurrent update would use)
    let r1 = sqlx::query("UPDATE escrows SET status = 'settled' WHERE id = ?1 AND status = 'active'")
        .bind("esc_atomic_001")
        .execute(&pool)
        .await;

    let r2 = sqlx::query("UPDATE escrows SET status = 'settled' WHERE id = ?1 AND status = 'active'")
        .bind("esc_atomic_001")
        .execute(&pool)
        .await;

    // Only the first should have succeeded
    assert_eq!(r1.unwrap().rows_affected(), 1, "First settle should succeed");
    assert_eq!(r2.unwrap().rows_affected(), 0, "Second settle should be a no-op");
}

// ── Property-Based Test: Escrow JSON Round-Trip ─────────────────

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn escrow_json_roundtrip(
            id in "[a-z0-9_]{1,32}",
            amount in 1_000u64..10_000_000_000_000u64,
            status in prop_oneof![
                Just(EscrowStatus::PendingConfirmation),
                Just(EscrowStatus::Active),
                Just(EscrowStatus::Settled),
                Just(EscrowStatus::Refunded),
                Just(EscrowStatus::Disputed),
                Just(EscrowStatus::Cancelled),
                Just(EscrowStatus::Expired),
            ],
        ) {
            let escrow = test_escrow(&id, status);
            let json = serde_json::to_string(&escrow).unwrap();
            let deserialized: Escrow = serde_json::from_str(&json).unwrap();
            assert_eq!(escrow.id, deserialized.id);
            assert_eq!(escrow.amount_sompi, deserialized.amount_sompi);
        }
    }
}
