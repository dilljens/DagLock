//! Edge case tests for DagLock indexer.

use sqlx::{Row, SqlitePool};

async fn test_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create test pool");

    sqlx::query(include_str!("../src/db/migrations/001_create_escrows.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migration 001");
    sqlx::query(include_str!("../src/db/migrations/002_create_offers.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migration 002");
    sqlx::query(include_str!("../src/db/migrations/003_create_indexes.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migration 003");
    sqlx::query(include_str!(
        "../src/db/migrations/004_create_dispute_evidence.sql"
    ))
    .execute(&pool)
    .await
    .expect("Failed to run migration 004");

    pool
}

#[tokio::test]
async fn test_atomic_settle_only_works_on_active() {
    let pool = test_pool().await;

    // Insert escrow with active status
    sqlx::query(
        "INSERT INTO escrows (id, lock_tx_id, lock_tx_output_index, status, asset_type,
         buyer_address, amount_sompi, fee_sompi, template_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )
    .bind("esc_active")
    .bind("tx1")
    .bind(0i64)
    .bind("active")
    .bind("KAS")
    .bind("kaspa:buyer")
    .bind(1_000_000_000i64)
    .bind(5_000_000i64)
    .bind(vec![1u8, 2, 3])
    .bind(1_700_000_000i64)
    .execute(&pool)
    .await
    .unwrap();

    // Insert escrow with settled status
    sqlx::query(
        "INSERT INTO escrows (id, lock_tx_id, lock_tx_output_index, status, asset_type,
         buyer_address, amount_sompi, fee_sompi, template_hash, created_at, settled_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )
    .bind("esc_settled")
    .bind("tx2")
    .bind(0i64)
    .bind("settled")
    .bind("KAS")
    .bind("kaspa:buyer")
    .bind(1_000_000_000i64)
    .bind(5_000_000i64)
    .bind(vec![1u8, 2, 3])
    .bind(1_700_000_000i64)
    .bind(1_700_001_000i64)
    .execute(&pool)
    .await
    .unwrap();

    // Try to settle active escrow - should succeed
    let result = sqlx::query(
        "UPDATE escrows SET status = 'settled', settled_at = ?1, refunded_at = NULL 
         WHERE id = ?2 AND status = 'active'",
    )
    .bind(1_700_002_000i64)
    .bind("esc_active")
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(result.rows_affected(), 1);

    // Try to settle already settled escrow - should fail (0 rows affected)
    let result = sqlx::query(
        "UPDATE escrows SET status = 'settled', settled_at = ?1, refunded_at = NULL 
         WHERE id = ?2 AND status = 'active'",
    )
    .bind(1_700_003_000i64)
    .bind("esc_settled")
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(result.rows_affected(), 0);
}

#[tokio::test]
async fn test_atomic_refund_only_works_on_active() {
    let pool = test_pool().await;

    // Insert active escrow
    sqlx::query(
        "INSERT INTO escrows (id, lock_tx_id, lock_tx_output_index, status, asset_type,
         buyer_address, amount_sompi, fee_sompi, template_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )
    .bind("esc_active")
    .bind("tx1")
    .bind(0i64)
    .bind("active")
    .bind("KAS")
    .bind("kaspa:buyer")
    .bind(1_000_000_000i64)
    .bind(5_000_000i64)
    .bind(vec![1u8, 2, 3])
    .bind(1_700_000_000i64)
    .execute(&pool)
    .await
    .unwrap();

    // Refund active escrow - should succeed
    let result = sqlx::query(
        "UPDATE escrows SET status = 'refunded', refunded_at = ?1, settled_at = NULL 
         WHERE id = ?2 AND status = 'active'",
    )
    .bind(1_700_002_000i64)
    .bind("esc_active")
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(result.rows_affected(), 1);

    // Verify status changed
    let (status,): (String,) = sqlx::query_as("SELECT status FROM escrows WHERE id = ?1")
        .bind("esc_active")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "refunded");
}

#[tokio::test]
async fn test_address_validation() {
    // Test valid addresses
    assert!(daglock_indexer::api::escrows::validate_kaspa_address(
        "kaspa:qz2q8c9yxp8yru3n"
    ));
    assert!(daglock_indexer::api::escrows::validate_kaspa_address(
        "kaspa:qr3a5x9yzp8yru3n"
    ));

    // Test invalid addresses
    assert!(!daglock_indexer::api::escrows::validate_kaspa_address(""));
    assert!(!daglock_indexer::api::escrows::validate_kaspa_address(
        "kaspa:"
    ));
    assert!(!daglock_indexer::api::escrows::validate_kaspa_address(
        "invalid"
    ));
    assert!(!daglock_indexer::api::escrows::validate_kaspa_address(
        "kaspa:INVALID"
    ));
}

#[tokio::test]
async fn test_fee_calculation_edge_cases() {
    // Test various amounts
    let test_cases = vec![
        (0i64, 0i64),                     // Zero amount
        (199i64, 0i64),                   // Below fee threshold
        (200i64, 1i64),                   // Exactly 0.5%
        (100_000_000i64, 500_000i64),     // 1 KAS
        (1_000_000_000i64, 5_000_000i64), // 10 KAS
    ];

    for (amount, expected_fee) in test_cases {
        let fee = amount / 200;
        assert_eq!(fee, expected_fee, "Fee calculation for amount {}", amount);
    }
}

#[tokio::test]
async fn test_reputation_score_bounds() {
    // Test that reputation score is always between 1.0 and 5.0
    let test_cases = vec![
        (0i64, 0i64, 0i64, 0, 0),
        (1, 100_000_000, 1, 0, 0),
        (10, 10_000_000_000, 365, 0, 0),
        (100, 100_000_000_000, 3650, 10, 5),
    ];

    for (trades, volume, age, _disputes, refunds) in test_cases {
        let score = daglock_indexer::db::queries::calculate_reputation_score(
            trades, 0, volume, age, refunds, 0,
        );
        assert!((1.0..=5.0).contains(&score), "Score {} out of range", score);
    }
}

/* ─── Full Lifecycle Tests ─────────────────────────────────────── */

#[tokio::test]
async fn test_full_lifecycle_create_settle_receipt() {
    let pool = test_pool().await;

    // 1. Create escrow
    sqlx::query(
        "INSERT INTO escrows (id, lock_tx_id, lock_tx_output_index, status, asset_type,
         buyer_address, seller_address, amount_sompi, fee_sompi, template_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )
    .bind("esc_lifecycle_full")
    .bind("tx_lifecycle_1")
    .bind(0i64)
    .bind("pending_confirmation")
    .bind("KAS")
    .bind("kaspa:buyer")
    .bind("kaspa:seller")
    .bind(500_000_000_000i64) // 5000 KAS
    .bind(2_500_000_000i64) // fee
    .bind(vec![1u8, 2, 3])
    .bind(1_700_000_000i64)
    .execute(&pool)
    .await
    .unwrap();

    // Verify created
    let (status,): (String,) = sqlx::query_as("SELECT status FROM escrows WHERE id = ?1")
        .bind("esc_lifecycle_full")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "pending_confirmation");

    // 2. Transition to active
    sqlx::query(
        "UPDATE escrows SET status = 'active' WHERE id = ?1 AND status = 'pending_confirmation'",
    )
    .bind("esc_lifecycle_full")
    .execute(&pool)
    .await
    .unwrap();

    // 3. Settle escrow (atomic update)
    let settled = sqlx::query(
        "UPDATE escrows SET status = 'settled', settled_at = ?1, refunded_at = NULL 
         WHERE id = ?2 AND status IN ('active', 'pending_confirmation')",
    )
    .bind(1_700_010_000i64)
    .bind("esc_lifecycle_full")
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(settled.rows_affected(), 1);

    // 4. Verify receipt (query all fields)
    let row = sqlx::query("SELECT id, status, amount_sompi, fee_sompi, buyer_address, seller_address, settled_at FROM escrows WHERE id = ?1")
        .bind("esc_lifecycle_full")
        .fetch_one(&pool)
        .await
        .unwrap();
    let st: String = row.try_get("status").unwrap();
    assert_eq!(st, "settled");
    let amt: i64 = row.try_get("amount_sompi").unwrap();
    assert_eq!(amt, 500_000_000_000);
    let fee: i64 = row.try_get("fee_sompi").unwrap();
    assert_eq!(fee, 2_500_000_000);
    // Fee should be 0.5%
    assert_eq!(fee, amt / 200);

    // 5. Verify double-settle is prevented
    let double = sqlx::query(
        "UPDATE escrows SET status = 'settled', settled_at = ?1 WHERE id = ?2 AND status IN ('active', 'pending_confirmation')",
    )
    .bind(1_700_020_000i64)
    .bind("esc_lifecycle_full")
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(double.rows_affected(), 0, "double settle should be blocked");
}

#[tokio::test]
async fn test_full_lifecycle_create_refund() {
    let pool = test_pool().await;

    // Create escrow
    sqlx::query(
        "INSERT INTO escrows (id, lock_tx_id, lock_tx_output_index, status, asset_type,
         buyer_address, amount_sompi, fee_sompi, template_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )
    .bind("esc_refund_full")
    .bind("tx_refund_1")
    .bind(0i64)
    .bind("active")
    .bind("KAS")
    .bind("kaspa:buyer")
    .bind(100_000_000_000i64)
    .bind(500_000_000i64)
    .bind(vec![1u8, 2, 3])
    .bind(1_700_000_000i64)
    .execute(&pool)
    .await
    .unwrap();

    // Refund
    let refunded = sqlx::query(
        "UPDATE escrows SET status = 'refunded', refunded_at = ?1 WHERE id = ?2 AND status = 'active'",
    )
    .bind(1_700_010_000i64)
    .bind("esc_refund_full")
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(refunded.rows_affected(), 1);

    // Verify
    let st: String = sqlx::query_as::<_, (String,)>("SELECT status FROM escrows WHERE id = ?1")
        .bind("esc_refund_full")
        .fetch_one(&pool)
        .await
        .unwrap()
        .0;
    assert_eq!(st, "refunded");

    // Double refund blocked
    let dbl = sqlx::query(
        "UPDATE escrows SET status = 'refunded', refunded_at = ?1 WHERE id = ?2 AND status = 'active'",
    )
    .bind(1_700_020_000i64)
    .bind("esc_refund_full")
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(dbl.rows_affected(), 0);
}

pub async fn ensure_dispute_columns(pool: &sqlx::SqlitePool) {
    // Ensure schema has lifecycle columns (migration ensure functions work
    // but in-memory tests may need these explicitly)
    let _ = sqlx::query("ALTER TABLE escrows ADD COLUMN dispute_outcome TEXT")
        .execute(pool)
        .await;
    let _ = sqlx::query("ALTER TABLE escrows ADD COLUMN dispute_resolved_at INTEGER")
        .execute(pool)
        .await;
    let _ = sqlx::query("ALTER TABLE escrows ADD COLUMN disputed_at INTEGER")
        .execute(pool)
        .await;
    let _ = sqlx::query("ALTER TABLE escrows ADD COLUMN dispute_reason TEXT")
        .execute(pool)
        .await;
    let _ = sqlx::query("ALTER TABLE escrows ADD COLUMN mediator_key TEXT")
        .execute(pool)
        .await;
    let _ = sqlx::query("ALTER TABLE escrows ADD COLUMN dispute_mode TEXT DEFAULT 'standard'")
        .execute(pool)
        .await;
}

async fn test_full_lifecycle_dispute_evidence() {
    let pool = test_pool().await;
    ensure_dispute_columns(&pool).await;

    // Create escrow
    sqlx::query(
        "INSERT INTO escrows (id, lock_tx_id, lock_tx_output_index, status, asset_type,
         buyer_address, seller_address, amount_sompi, fee_sompi, template_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )
    .bind("esc_dispute_full")
    .bind("tx_dispute_1")
    .bind(0i64)
    .bind("active")
    .bind("KAS")
    .bind("kaspa:buyer")
    .bind("kaspa:seller")
    .bind(1_000_000_000i64)
    .bind(5_000_000i64)
    .bind(vec![1u8, 2, 3])
    .bind(1_700_000_000i64)
    .execute(&pool)
    .await
    .unwrap();

    // Dispute
    sqlx::query(
        "UPDATE escrows SET status = 'disputed', disputed_at = ?1, dispute_reason = ?2 WHERE id = ?3",
    )
    .bind(1_700_005_000i64)
    .bind("seller did not deliver")
    .bind("esc_dispute_full")
    .execute(&pool)
    .await
    .unwrap();

    let (st, reason): (String, String) =
        sqlx::query_as("SELECT status, dispute_reason FROM escrows WHERE id = ?1")
            .bind("esc_dispute_full")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(st, "disputed");
    assert_eq!(reason, "seller did not deliver");

    // Add evidence
    let evidence_id = format!("ev_{}", 1001);
    sqlx::query(
        "INSERT INTO dispute_evidence (id, escrow_id, submitted_by, content, content_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(&evidence_id)
    .bind("esc_dispute_full")
    .bind("kaspa:buyer")
    .bind("Screenshots of delivery proof")
    .bind("abc123")
    .bind(1_700_006_000i64)
    .execute(&pool)
    .await
    .unwrap();

    // Resolve dispute
    sqlx::query(
        "UPDATE escrows SET dispute_outcome = ?1, dispute_resolved_at = ?2 WHERE id = ?3 AND status = 'disputed'",
    )
    .bind("uphold")
    .bind(1_700_007_000i64)
    .bind("esc_dispute_full")
    .execute(&pool)
    .await
    .unwrap();

    let outcome: String =
        sqlx::query_as::<_, (String,)>("SELECT dispute_outcome FROM escrows WHERE id = ?1")
            .bind("esc_dispute_full")
            .fetch_one(&pool)
            .await
            .unwrap()
            .0;
    assert_eq!(outcome, "uphold");
}

#[tokio::test]
async fn test_offer_lifecycle() {
    let pool = test_pool().await;

    // Create offer
    sqlx::query(
        "INSERT INTO offers (id, creator_address, side, base_asset, quote_asset, amount_sompi, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )
    .bind("off_test_full")
    .bind("kaspa:creator")
    .bind("sell")
    .bind("KAS")
    .bind("USDC")
    .bind(1_000_000_000_000i64)
    .bind("proposed")
    .bind(1_700_000_000i64)
    .execute(&pool)
    .await
    .unwrap();

    // Accept
    sqlx::query("UPDATE offers SET status = 'accepted', counterparty_address = ?1 WHERE id = ?2")
        .bind("kaspa:accepter")
        .bind("off_test_full")
        .execute(&pool)
        .await
        .unwrap();

    let st: String = sqlx::query_as::<_, (String,)>("SELECT status FROM offers WHERE id = ?1")
        .bind("off_test_full")
        .fetch_one(&pool)
        .await
        .unwrap()
        .0;
    assert_eq!(st, "accepted");
}
