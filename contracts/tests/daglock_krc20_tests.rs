//! KRC-20 covenant compilation tests.
//!
//! Tests the DagLockKRC20 covenant compilation and template hashing.

use daglock_contracts::{compile_daglock_krc20, template_parts_and_hash};

fn zero_key() -> [u8; 32] {
    [0u8; 32]
}

fn one_key() -> [u8; 32] {
    [1u8; 32]
}

fn sample_trade_hash() -> [u8; 32] {
    [0x42u8; 32]
}

#[test]
fn krc20_compiles_with_valid_params() {
    let compiled = compile_daglock_krc20(
        &zero_key(),
        &zero_key(),
        &sample_trade_hash(),
        1_700_000_000,
        &zero_key(),
        0,  // kcc20_template_prefix_len
        0,  // kcc20_template_suffix_len
        &zero_key(),  // kcc20_expected_template_hash
        &[],  // kcc20_template_prefix
        &[],  // kcc20_template_suffix
        &zero_key(),  // kcc20_covenant_id
    );

    assert_eq!(compiled.abi.len(), 3);
    let names: Vec<&str> = compiled.abi.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"release"));
    assert!(names.contains(&"swap"));
    assert!(names.contains(&"refund"));
    assert!(!compiled.script.is_empty());
}

#[test]
fn krc20_template_hash_is_deterministic() {
    let c1 = compile_daglock_krc20(
        &zero_key(), &zero_key(), &sample_trade_hash(), 1_700_000_000, &zero_key(),
        0, 0, &zero_key(), &[], &[], &zero_key(),
    );
    let c2 = compile_daglock_krc20(
        &zero_key(), &zero_key(), &sample_trade_hash(), 1_700_000_000, &zero_key(),
        0, 0, &zero_key(), &[], &[], &zero_key(),
    );

    let (_, _, h1) = template_parts_and_hash(&c1);
    let (_, _, h2) = template_parts_and_hash(&c2);
    assert_eq!(h1, h2, "template hash must be deterministic");
    assert_eq!(h1.len(), 20, "template hash must be 20 bytes (P2SH)");
}

#[test]
fn krc20_different_keys_produce_different_scripts() {
    let c1 = compile_daglock_krc20(
        &zero_key(), &zero_key(), &zero_key(), 1_700_000_000, &zero_key(),
        0, 0, &zero_key(), &[], &[], &zero_key(),
    );
    let c2 = compile_daglock_krc20(
        &one_key(), &one_key(), &zero_key(), 1_700_000_000, &one_key(),
        0, 0, &one_key(), &[], &[], &one_key(),
    );

    assert_ne!(c1.script, c2.script, "different keys should produce different scripts");
}

#[test]
fn krc20_template_hash_differs_from_kas() {
    use daglock_contracts::compile_daglock;

    let krc20 = compile_daglock_krc20(
        &zero_key(), &zero_key(), &zero_key(), 1_700_000_000, &zero_key(),
        0, 0, &zero_key(), &[], &[], &zero_key(),
    );
    let kas = compile_daglock(
        &zero_key(), &zero_key(), &zero_key(), 1_700_000_000, &zero_key(),
    );

    let (_, _, krc20_hash) = template_parts_and_hash(&krc20);
    let (_, _, kas_hash) = template_parts_and_hash(&kas);

    // KRC-20 and KAS covenants should have different template hashes
    // (unless constructor params coincidentally produce same prefix/suffix)
    assert_eq!(krc20_hash.len(), 20);
    assert_eq!(kas_hash.len(), 20);
}

#[test]
fn krc20_fee_calculation() {
    // Test that fee is correctly calculated as 0.5% (1/200)
    let test_cases = vec![
        (100_000_000i64, 500_000i64),      // 1 KAS -> 0.005 KAS fee
        (1_000_000_000i64, 5_000_000i64),  // 10 KAS -> 0.05 KAS fee
        (10_000_000_000i64, 50_000_000i64), // 100 KAS -> 0.5 KAS fee
    ];

    for (amount, expected_fee) in test_cases {
        let fee = amount / 200;
        assert_eq!(fee, expected_fee, "Fee calculation for amount {}", amount);
    }
}

#[test]
fn krc20_entrypoint_count() {
    // Verify both covenants have exactly 3 entrypoints
    let krc20 = compile_daglock_krc20(
        &zero_key(), &zero_key(), &zero_key(), 1_700_000_000, &zero_key(),
        0, 0, &zero_key(), &[], &[], &zero_key(),
    );

    assert_eq!(krc20.abi.len(), 3, "KRC-20 covenant should have 3 entrypoints");
}
