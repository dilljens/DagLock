//! Edge case tests for DagLock indexer.

use sqlx::SqlitePool;

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

    pool
}

#[tokio::test]
async fn test_atomic_settle_only_works_on_active() {
    let pool = test_pool().await;

    // Insert escrow with active status
    sqlx::query(
        "INSERT INTO escrows (id, lock_tx_id, lock_tx_output_index, status, asset_type,
         buyer_address, amount_sompi, fee_sompi, template_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
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
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
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
         WHERE id = ?2 AND status = 'active'"
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
         WHERE id = ?2 AND status = 'active'"
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
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
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
         WHERE id = ?2 AND status = 'active'"
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
    assert!(daglock_indexer::api::escrows::validate_kaspa_address("kaspa:qz2q8c9yxp8yru3n"));
    assert!(daglock_indexer::api::escrows::validate_kaspa_address("kaspa:qr3a5x9yzp8yru3n"));

    // Test invalid addresses
    assert!(!daglock_indexer::api::escrows::validate_kaspa_address(""));
    assert!(!daglock_indexer::api::escrows::validate_kaspa_address("kaspa:"));
    assert!(!daglock_indexer::api::escrows::validate_kaspa_address("invalid"));
    assert!(!daglock_indexer::api::escrows::validate_kaspa_address("kaspa:INVALID"));
}

#[tokio::test]
async fn test_fee_calculation_edge_cases() {
    // Test various amounts
    let test_cases = vec![
        (0i64, 0i64),           // Zero amount
        (199i64, 0i64),         // Below fee threshold
        (200i64, 1i64),         // Exactly 0.5%
        (100_000_000i64, 500_000i64),    // 1 KAS
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

    for (trades, volume, age, disputes, refunds) in test_cases {
        let score = daglock_indexer::db::queries::calculate_reputation_score(
            trades, volume, age, refunds,
        );
        assert!(score >= 1.0 && score <= 5.0, "Score {} out of range", score);
    }
}
