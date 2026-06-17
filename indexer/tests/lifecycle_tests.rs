//! Lifecycle integration tests for DagLock indexer.
//!
//! Tests full escrow, offer, vault, and dispute flows through the query layer.
//! Uses in-memory SQLite with all migrations applied.

use daglock_indexer::db::queries;
use daglock_indexer::types::*;
use sqlx::SqlitePool;

async fn test_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create test pool");
    daglock_indexer::db::schema::migrate(&pool)
        .await
        .expect("Failed to run migrations");
    pool
}

fn make_escrow(id: &str, status: EscrowStatus) -> Escrow {
    Escrow {
        id: id.to_string(),
        lock_tx_id: format!("tx_{id}"),
        lock_tx_output_index: 0,
        status,
        asset_type: "KAS".to_string(),
        buyer_address: "kaspa:buyer1".to_string(),
        seller_address: Some("kaspa:seller1".to_string()),
        amount_sompi: 1_000_000_000, // 10 KAS
        fee_sompi: 5_000_000,        // 0.5%
        template_hash: vec![0x30, 0x87, 0x6e],
        expiration_daa_score: Some(100_000),
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
    }
}

// ─── Create → Settle → Receipt ──────────────────────────────────

#[tokio::test]
async fn test_create_settle_receipt() {
    let pool = test_pool().await;
    let escrow = make_escrow("esc_001", EscrowStatus::PendingConfirmation);

    queries::insert_escrow(&pool, &escrow).await.unwrap();

    let got = queries::get_escrow(&pool, "esc_001").await.unwrap();
    assert!(got.is_some(), "escrow should exist after insert");
    let got = got.unwrap();
    assert_eq!(got.status, EscrowStatus::PendingConfirmation);

    let settled = queries::settle_escrow_atomic(&pool, "esc_001")
        .await
        .unwrap();
    assert!(settled, "settle should succeed");

    let got = queries::get_escrow(&pool, "esc_001")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.status, EscrowStatus::Settled);
    assert!(got.settled_at.is_some());

    let receipt = queries::receipt_from_escrow(&got);
    assert_eq!(receipt.status, "settled");
    assert!(receipt.verification.covenant_verified);
    assert!(receipt.verification.fee_compliant);
}

// ─── Create → Refund ────────────────────────────────────────────

#[tokio::test]
async fn test_create_refund() {
    let pool = test_pool().await;
    let escrow = make_escrow("esc_002", EscrowStatus::Active);

    queries::insert_escrow(&pool, &escrow).await.unwrap();

    let refunded = queries::refund_escrow_atomic(&pool, "esc_002")
        .await
        .unwrap();
    assert!(refunded, "refund should succeed");

    let got = queries::get_escrow(&pool, "esc_002")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.status, EscrowStatus::Refunded);
    assert!(got.refunded_at.is_some());
}

// ─── Atomic Swap ────────────────────────────────────────────────

#[tokio::test]
async fn test_atomic_swap_with_trade_hash() {
    let pool = test_pool().await;
    let mut escrow = make_escrow("esc_swap", EscrowStatus::Active);
    escrow.trade_hash = Some("a".repeat(64));

    queries::insert_escrow(&pool, &escrow).await.unwrap();

    let got = queries::get_escrow(&pool, "esc_swap")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.trade_hash, Some("a".repeat(64)));

    let settled = queries::settle_escrow_atomic(&pool, "esc_swap")
        .await
        .unwrap();
    assert!(settled);

    let got = queries::get_escrow(&pool, "esc_swap")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.status, EscrowStatus::Settled);
}

// ─── Dispute → Jury → Verdict ───────────────────────────────────

#[tokio::test]
async fn test_dispute_jury_verdict() {
    let pool = test_pool().await;

    let escrow = make_escrow("esc_dispute", EscrowStatus::Active);
    queries::insert_escrow(&pool, &escrow).await.unwrap();

    queries::mark_escrow_disputed(&pool, "esc_dispute", "item not as described")
        .await
        .unwrap();

    let got = queries::get_escrow(&pool, "esc_dispute")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.status, EscrowStatus::Disputed);
    assert_eq!(
        got.dispute_reason,
        Some("item not as described".to_string())
    );

    // Register 4 jurors
    queries::register_juror(&pool, "kaspa:juror1")
        .await
        .unwrap();
    queries::register_juror(&pool, "kaspa:juror2")
        .await
        .unwrap();
    queries::register_juror(&pool, "kaspa:juror3")
        .await
        .unwrap();
    queries::register_juror(&pool, "kaspa:juror4")
        .await
        .unwrap();

    let juror_addrs: Vec<String> = vec![
        "kaspa:juror1".into(),
        "kaspa:juror2".into(),
        "kaspa:juror3".into(),
        "kaspa:juror4".into(),
    ];

    // Create jury case with threshold 3
    let case_id = queries::create_jury_case(&pool, "esc_dispute", 4, 3, &juror_addrs)
        .await
        .unwrap();

    // Cast votes: 2 for seller, 1 for buyer
    queries::cast_jury_vote(&pool, &case_id, "kaspa:juror1", "seller_wins", None)
        .await
        .unwrap();
    queries::cast_jury_vote(&pool, &case_id, "kaspa:juror2", "seller_wins", None)
        .await
        .unwrap();
    queries::cast_jury_vote(&pool, &case_id, "kaspa:juror3", "buyer_wins", None)
        .await
        .unwrap();

    // Threshold is 3, only 2 votes for seller — no verdict yet
    let verdict = queries::check_jury_verdict(&pool, &case_id).await.unwrap();
    assert!(verdict.is_none(), "no verdict yet — need 3 votes");

    // Cast another vote for seller
    queries::cast_jury_vote(&pool, &case_id, "kaspa:juror4", "seller_wins", None)
        .await
        .unwrap();

    // Now seller_wins
    let verdict = queries::check_jury_verdict(&pool, &case_id).await.unwrap();
    assert_eq!(verdict, Some("seller_wins".to_string()));
}

// ─── Double Settle Rejected ─────────────────────────────────────

#[tokio::test]
async fn test_double_settle_rejected() {
    let pool = test_pool().await;
    let escrow = make_escrow("esc_double", EscrowStatus::Active);

    queries::insert_escrow(&pool, &escrow).await.unwrap();

    let first = queries::settle_escrow_atomic(&pool, "esc_double")
        .await
        .unwrap();
    assert!(first);

    let second = queries::settle_escrow_atomic(&pool, "esc_double")
        .await
        .unwrap();
    assert!(!second, "double settle should return false");

    let got = queries::get_escrow(&pool, "esc_double")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.status, EscrowStatus::Settled);
}

// ─── Cancel Escrow ──────────────────────────────────────────────

#[tokio::test]
async fn test_cancel_escrow() {
    let pool = test_pool().await;
    let escrow = make_escrow("esc_cancel", EscrowStatus::PendingConfirmation);

    queries::insert_escrow(&pool, &escrow).await.unwrap();

    queries::mark_escrow_cancelled(&pool, "esc_cancel")
        .await
        .unwrap();

    let got = queries::get_escrow(&pool, "esc_cancel")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.status, EscrowStatus::Cancelled);
    assert!(got.cancelled_at.is_some());
}

// ─── Offer Lifecycle ────────────────────────────────────────────

#[tokio::test]
async fn test_offer_create_accept() {
    let pool = test_pool().await;

    let offer = Offer {
        id: "off_001".to_string(),
        creator_address: "kaspa:maker".to_string(),
        side: "buy".to_string(),
        base_asset: "KAS".to_string(),
        quote_asset: "USD".to_string(),
        amount_sompi: 500_000_000,
        counterparty_address: None,
        status: "open".to_string(),
        expires_at: None,
        created_at: 1_700_000_000,
        price_type: "fixed".to_string(),
        price_offset: None,
        min_price: None,
        max_price: None,
        current_price: None,
        price_currency: "USD".to_string(),
        price_updated_at: None,
    };

    queries::insert_offer(&pool, &offer).await.unwrap();

    let got = queries::get_offer(&pool, "off_001").await.unwrap();
    assert!(got.is_some());
    assert_eq!(got.unwrap().status, "open");

    queries::accept_offer(&pool, "off_001", "kaspa:taker")
        .await
        .unwrap();

    let got = queries::get_offer(&pool, "off_001").await.unwrap().unwrap();
    assert_eq!(got.status, "accepted");
    assert_eq!(got.counterparty_address.as_deref(), Some("kaspa:taker"));
}

// ─── Vault Create + Update ──────────────────────────────────────

#[tokio::test]
async fn test_vault_create_and_update() {
    let pool = test_pool().await;

    let vault = Vault {
        id: "vault_001".to_string(),
        owner_address: "kaspa:owner".to_string(),
        beneficiary_address: None,
        vault_type: VaultType::Time,
        status: VaultStatus::Locked,
        amount_sompi: 2_000_000_000,
        timeout: 50_000,
        lock_tx_id: None,
        lock_tx_output_index: None,
        created_at: 1_700_000_000,
        unlocked_at: None,
        expires_at: None,
        owner_pubkey_hex: None,
        sweep_tx_id: None,
    };

    queries::insert_vault(&pool, &vault).await.unwrap();

    let got = queries::get_vault(&pool, "vault_001").await.unwrap();
    assert!(got.is_some());
    assert_eq!(got.unwrap().status, VaultStatus::Locked);

    let count = queries::count_vaults_by_owner(&pool, "kaspa:owner")
        .await
        .unwrap();
    assert_eq!(count, 1);

    // update_vault_status stores the raw string; row_to_vault deserializes via serde_json
    // so we need to pass the JSON-serialized form: "unlocked" → "\"unlocked\""
    let status_json = serde_json::to_string(&VaultStatus::Unlocked).unwrap();
    queries::update_vault_status(&pool, "vault_001", &status_json)
        .await
        .unwrap();

    let got = queries::get_vault(&pool, "vault_001")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.status, VaultStatus::Unlocked);
}

// ─── Auth Nonce Replay Protection ───────────────────────────────

#[tokio::test]
async fn test_auth_nonce_replay_protection() {
    let pool = test_pool().await;

    let nonce_bytes = b"abcdef1234567890abcdef12"; // 20 bytes

    queries::store_auth_nonce(
        &pool,
        nonce_bytes,
        "settle",
        "esc_001",
        "kaspa:buyer1",
        1_700_000_000,
    )
    .await
    .unwrap();

    let exists = queries::check_auth_nonce_exists(&pool, nonce_bytes)
        .await
        .unwrap();
    assert!(exists, "nonce should exist after storage");

    // Cleanup with cutoff AFTER nonce creation — should remove it (created_at < cutoff)
    let cleaned = queries::cleanup_expired_auth_nonces(&pool, 1_700_000_001)
        .await
        .unwrap();
    assert_eq!(cleaned, 1, "nonce should be cleaned up");

    let exists = queries::check_auth_nonce_exists(&pool, nonce_bytes)
        .await
        .unwrap();
    assert!(!exists, "nonce should be gone after cleanup");
}

// ─── Reputation Score ───────────────────────────────────────────

#[tokio::test]
async fn test_reputation_score_calculation() {
    let score_clean = queries::calculate_reputation_score(10, 10, 10_000_000_000, 180, 0, 0);
    let score_disputed = queries::calculate_reputation_score(10, 10, 10_000_000_000, 180, 3, 3);

    assert!(score_clean > score_disputed, "disputes should lower score");
    assert!(score_clean >= 1.0 && score_clean <= 5.0);
    assert!(score_disputed >= 1.0 && score_disputed <= 5.0);
}

// ─── List by Address ────────────────────────────────────────────

#[tokio::test]
async fn test_list_escrows_by_address() {
    let pool = test_pool().await;

    let e1 = make_escrow("esc_a1", EscrowStatus::Active);
    let mut e2 = make_escrow("esc_a2", EscrowStatus::Settled);
    e2.buyer_address = "kaspa:buyer1".to_string();
    let mut e3 = make_escrow("esc_a3", EscrowStatus::Active);
    e3.buyer_address = "kaspa:other".to_string();

    queries::insert_escrow(&pool, &e1).await.unwrap();
    queries::insert_escrow(&pool, &e2).await.unwrap();
    queries::insert_escrow(&pool, &e3).await.unwrap();

    // Verify inserts
    assert!(queries::get_escrow(&pool, "esc_a1")
        .await
        .unwrap()
        .is_some());

    // Test list_escrows_by_address with the fixed SQL params
    let (results, count) =
        queries::list_escrows_by_address(&pool, "kaspa:buyer1", None, None, 10, 0)
            .await
            .unwrap();
    assert_eq!(count, 2, "buyer1 should have 2 escrows (count)");
    assert_eq!(results.len(), 2, "buyer1 should have 2 escrows (results)");

    let (results, count) =
        queries::list_escrows_by_address(&pool, "kaspa:other", None, None, 10, 0)
            .await
            .unwrap();
    assert_eq!(count, 1, "other should have 1 escrow (count)");
    assert_eq!(results.len(), 1, "other should have 1 escrow (results)");

    // Test with status filter
    let (results, count) =
        queries::list_escrows_by_address(&pool, "kaspa:buyer1", None, Some("active"), 10, 0)
            .await
            .unwrap();
    assert_eq!(count, 1, "buyer1 should have 1 active escrow");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].status, EscrowStatus::Active);
}

// ─── Stats ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_stats_after_lifecycle() {
    let pool = test_pool().await;

    let mut e1 = make_escrow("esc_s1", EscrowStatus::Active);
    e1.amount_sompi = 1_000_000_000;
    let mut e2 = make_escrow("esc_s2", EscrowStatus::Active);
    e2.amount_sompi = 2_000_000_000;
    let mut e3 = make_escrow("esc_s3", EscrowStatus::Active);
    e3.amount_sompi = 3_000_000_000;

    queries::insert_escrow(&pool, &e1).await.unwrap();
    queries::insert_escrow(&pool, &e2).await.unwrap();
    queries::insert_escrow(&pool, &e3).await.unwrap();

    queries::settle_escrow_atomic(&pool, "esc_s1")
        .await
        .unwrap();
    queries::refund_escrow_atomic(&pool, "esc_s2")
        .await
        .unwrap();

    let stats = queries::get_stats(&pool).await.unwrap();
    assert_eq!(stats.total_escrows, 3);
    assert_eq!(stats.settled_escrows, 1);
    assert_eq!(stats.refunded_escrows, 1);
    assert_eq!(stats.active_escrows, 1);
    assert!(!stats.total_volume_kas.is_empty());
}

// ─── Vouch System ───────────────────────────────────────────────

#[tokio::test]
async fn test_vouch_lifecycle() {
    let pool = test_pool().await;

    // expires_at must be in the future for count_vouches_for_subject to find it
    let future_ts = chrono::Utc::now().timestamp() + 86_400 * 365; // 1 year from now

    let vouch = Vouch {
        id: "vouch_001".to_string(),
        voucher_address: "kaspa:voucher".to_string(),
        subject_address: "kaspa:subject".to_string(),
        escrow_id: Some("esc_vouch".to_string()),
        note: Some("trusted trader".to_string()),
        created_at: 1_700_000_000,
        expires_at: future_ts,
    };

    queries::insert_vouch(&pool, &vouch).await.unwrap();

    let count = queries::count_vouches_for_subject(&pool, "kaspa:subject")
        .await
        .unwrap();
    assert_eq!(count, 1);

    let vouches = queries::list_vouches_for_subject(&pool, "kaspa:subject")
        .await
        .unwrap();
    assert_eq!(vouches.len(), 1);

    queries::delete_vouch(&pool, "vouch_001", "kaspa:voucher")
        .await
        .unwrap();

    let count = queries::count_vouches_for_subject(&pool, "kaspa:subject")
        .await
        .unwrap();
    assert_eq!(count, 0);
}

// ─── Jury Registration ──────────────────────────────────────────

#[tokio::test]
async fn test_jury_registration() {
    let pool = test_pool().await;

    queries::register_juror(&pool, "kaspa:juror_a")
        .await
        .unwrap();
    queries::register_juror(&pool, "kaspa:juror_b")
        .await
        .unwrap();

    let jurors = queries::list_eligible_jurors_simple(&pool).await.unwrap();
    assert!(jurors.len() >= 2);

    let juror = queries::get_juror(&pool, "kaspa:juror_a").await.unwrap();
    assert!(juror.is_some());

    let unregistered = queries::unregister_juror(&pool, "kaspa:juror_a")
        .await
        .unwrap();
    assert!(unregistered);

    let juror = queries::get_juror(&pool, "kaspa:juror_a").await.unwrap();
    assert!(juror.is_none());
}

// ─── Messages ───────────────────────────────────────────────────

#[tokio::test]
async fn test_message_insert_and_list() {
    let pool = test_pool().await;

    // Messages require a parent escrow (FK constraint)
    let escrow = make_escrow("esc_msg", EscrowStatus::Active);
    queries::insert_escrow(&pool, &escrow).await.unwrap();

    let msg = EscrowMessage {
        id: "msg_001".to_string(),
        escrow_id: "esc_msg".to_string(),
        sender_address: "kaspa:sender".to_string(),
        content: "plaintext".to_string(),
        created_at: 1_700_000_000,
    };

    queries::insert_message(&pool, &msg, "encrypted_content", "nonce_hex")
        .await
        .unwrap();

    let messages = queries::list_messages_raw(&pool, "esc_msg").await.unwrap();
    assert_eq!(messages.len(), 1);

    let count = queries::count_messages(&pool, "esc_msg").await.unwrap();
    assert_eq!(count, 1);
}
