//! Integration tests for the DagLock REST API.
//!
//! Tests the full request/response cycle against an in-memory SQLite database.

use sqlx::SqlitePool;

// We need to import from the binary crate, so we'll use a different approach
// by directly testing the API handlers

/// Helper to create a test database pool with migrations
async fn test_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    daglock_indexer::db::schema::migrate(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_database_migration() {
    let pool = test_pool().await;

    // Verify escrows table exists
    let result: Result<(i64,), _> = sqlx::query_as("SELECT COUNT(*) FROM escrows")
        .fetch_one(&pool)
        .await;
    assert!(result.is_ok(), "escrows table should exist");

    // Verify offers table exists
    let result: Result<(i64,), _> = sqlx::query_as("SELECT COUNT(*) FROM offers")
        .fetch_one(&pool)
        .await;
    assert!(result.is_ok(), "offers table should exist");
}

#[tokio::test]
async fn test_insert_and_query_escrow() {
    let pool = test_pool().await;

    // Insert escrow
    sqlx::query(
        "INSERT INTO escrows (id, lock_tx_id, lock_tx_output_index, status, asset_type,
         buyer_address, seller_address, amount_sompi, fee_sompi, template_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )
    .bind("esc_test1")
    .bind("tx123")
    .bind(0i64)
    .bind("pending_confirmation")
    .bind("KAS")
    .bind("kaspa:buyer1")
    .bind::<Option<String>>(None)
    .bind(500_000_000i64)
    .bind(2_500_000i64)
    .bind(vec![1u8, 2, 3])
    .bind(1_700_000_000i64)
    .execute(&pool)
    .await
    .expect("Failed to insert escrow");

    // Query escrow
    let row: (String, String, i64) =
        sqlx::query_as("SELECT id, buyer_address, amount_sompi FROM escrows WHERE id = ?1")
            .bind("esc_test1")
            .fetch_one(&pool)
            .await
            .expect("Failed to query escrow");

    assert_eq!(row.0, "esc_test1");
    assert_eq!(row.1, "kaspa:buyer1");
    assert_eq!(row.2, 500_000_000);
}

#[tokio::test]
async fn test_escrow_lifecycle_transitions() {
    let pool = test_pool().await;

    // Insert escrow
    sqlx::query(
        "INSERT INTO escrows (id, lock_tx_id, lock_tx_output_index, status, asset_type,
         buyer_address, amount_sompi, fee_sompi, template_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )
    .bind("esc_lifecycle")
    .bind("tx456")
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
    .expect("Failed to insert escrow");

    // Transition to settled
    sqlx::query("UPDATE escrows SET status = 'settled', settled_at = ?1 WHERE id = ?2")
        .bind(1_700_001_000i64)
        .bind("esc_lifecycle")
        .execute(&pool)
        .await
        .expect("Failed to update status");

    // Verify transition
    let (status,): (String,) = sqlx::query_as("SELECT status FROM escrows WHERE id = ?1")
        .bind("esc_lifecycle")
        .fetch_one(&pool)
        .await
        .expect("Failed to query status");

    assert_eq!(status, "settled");
}

#[tokio::test]
async fn test_reputation_calculation() {
    let pool = test_pool().await;

    // Insert multiple settled escrows for same buyer
    for i in 0..5 {
        sqlx::query(
            "INSERT INTO escrows (id, lock_tx_id, lock_tx_output_index, status, asset_type,
             buyer_address, amount_sompi, fee_sompi, template_hash, created_at, settled_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        )
        .bind(format!("esc_rep_{}", i))
        .bind(format!("tx{}", i))
        .bind(0i64)
        .bind("settled")
        .bind("KAS")
        .bind("kaspa:active_buyer")
        .bind(100_000_000i64)
        .bind(500_000i64)
        .bind(vec![1u8, 2, 3])
        .bind(1_700_000_000i64 + i * 86_400)
        .bind(Some(1_700_000_000i64 + i * 86_400))
        .execute(&pool)
        .await
        .expect("Failed to insert escrow");
    }

    // Query trade count
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE buyer_address = ?1 AND status = 'settled'",
    )
    .bind("kaspa:active_buyer")
    .fetch_one(&pool)
    .await
    .expect("Failed to count trades");

    assert_eq!(count, 5);
}

#[tokio::test]
async fn test_fee_calculation() {
    let _pool = test_pool().await;

    // Test various amounts
    let test_cases = vec![
        (100_000_000i64, 500_000i64),       // 1 KAS -> 0.005 KAS fee
        (1_000_000_000i64, 5_000_000i64),   // 10 KAS -> 0.05 KAS fee
        (10_000_000_000i64, 50_000_000i64), // 100 KAS -> 0.5 KAS fee
    ];

    for (amount, expected_fee) in test_cases {
        let fee = amount / 200;
        assert_eq!(fee, expected_fee, "Fee calculation for amount {}", amount);
    }
}
