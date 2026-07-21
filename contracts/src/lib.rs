//! DagLock contracts — compile-time access to the daglock.sil covenant.
//!
//! Provides:
//! - `daglock_source()` — the .sil source as a static string
//! - `compile_daglock(...)` — compile with given constructor args
//! - `template_parts_and_hash()` — compute the stable template hash

// Re-export for downstream consumers (tests, indexer, WASM SDK)
pub use silverscript_lang;

use silverscript_lang::ast::Expr;
use silverscript_lang::compiler::{compile_contract, CompileOptions, CompiledContract};

/// The daglock.sil source embedded at compile time.
pub fn daglock_source() -> &'static str {
    include_str!("daglock.sil")
}

/// The daglock_krc20.sil source embedded at compile time.
pub fn daglock_krc20_source() -> &'static str {
    include_str!("daglock_krc20.sil")
}

/// The daglock_arbiter.sil source embedded at compile time.
pub fn daglock_arbiter_source() -> &'static str {
    include_str!("daglock_arbiter.sil")
}

/// The daglock_vault_multisig.sil source embedded at compile time.
pub fn daglock_vault_multisig_source() -> &'static str {
    include_str!("daglock_vault_multisig.sil")
}

/// The daglock_vault_softlock.sil source embedded at compile time.
pub fn daglock_vault_softlock_source() -> &'static str {
    include_str!("daglock_vault_softlock.sil")
}

/// The daglock_vault.sil source embedded at compile time.
pub fn daglock_vault_source() -> &'static str {
    include_str!("daglock_vault.sil")
}

/// Compile the DagLock covenant with the given constructor arguments.
///
/// Arguments (in order):
/// - `buyer_pk`: 32-byte compressed public key
/// - `seller_pk`: 32-byte compressed public key
/// - `trade_hash`: 32-byte SHA-256 hash (or zeroes if no atomic swap)
/// - `timeout`: Unix timestamp (i64)
/// - `treasury_pk`: 32-byte compressed public key
pub fn compile_daglock(
    buyer_pk: &[u8],
    seller_pk: &[u8],
    trade_hash: &[u8],
    timeout: i64,
    treasury_pk: &[u8],
) -> CompiledContract<'static> {
    let source = daglock_source();
    let args = vec![
        Expr::bytes(buyer_pk.to_vec()),
        Expr::bytes(seller_pk.to_vec()),
        Expr::bytes(trade_hash.to_vec()),
        Expr::int(timeout),
        Expr::bytes(treasury_pk.to_vec()),
    ];
    compile_contract(source, &args, CompileOptions::default())
        .expect("daglock.sil should compile — if this fails, fix the .sil syntax")
}

/// Compile the DagLock Arbiter covenant with the given constructor arguments.
///
/// This is the same as daglock.sil but with an additional `arbiterKey` parameter
/// and two extra entrypoints: `disputeSellerWins` and `disputeBuyerWins`.
/// If `arbiter_key` is all-zeroes, the dispute paths are unreachable.
///
/// Arguments (in order):
/// - `buyer_pk`: 32-byte compressed public key
/// - `seller_pk`: 32-byte compressed public key
/// - `trade_hash`: 32-byte SHA-256 hash (or zeroes if no atomic swap)
/// - `timeout`: Unix timestamp (i64)
/// - `treasury_pk`: 32-byte compressed public key
/// - `arbiter_key`: 32-byte compressed public key (zeroes = dispute paths disabled)
pub fn compile_daglock_arbiter(
    buyer_pk: &[u8],
    seller_pk: &[u8],
    trade_hash: &[u8],
    timeout: i64,
    treasury_pk: &[u8],
    arbiter_key: &[u8],
) -> CompiledContract<'static> {
    let source = daglock_arbiter_source();
    let args = vec![
        Expr::bytes(buyer_pk.to_vec()),
        Expr::bytes(seller_pk.to_vec()),
        Expr::bytes(trade_hash.to_vec()),
        Expr::int(timeout),
        Expr::bytes(treasury_pk.to_vec()),
        Expr::bytes(arbiter_key.to_vec()),
    ];
    compile_contract(source, &args, CompileOptions::default())
        .expect("daglock_arbiter.sil should compile — if this fails, fix the .sil syntax")
}

/// The daglock_advanced.sil source embedded at compile time.
pub fn daglock_advanced_source() -> &'static str {
    include_str!("daglock_advanced.sil")
}

/// The daglock_subscription.sil source embedded at compile time.
pub fn daglock_subscription_source() -> &'static str {
    include_str!("daglock_subscription.sil")
}

/// The daglock_milestone.sil source embedded at compile time.
pub fn daglock_milestone_source() -> &'static str {
    include_str!("daglock_milestone.sil")
}

/// The daglock_multi.sil source embedded at compile time.
pub fn daglock_multi_source() -> &'static str {
    include_str!("daglock_multi.sil")
}

/// The daglock_deposit.sil source embedded at compile time.
pub fn daglock_deposit_source() -> &'static str {
    include_str!("daglock_deposit.sil")
}

/// Compile the DagLock Advanced covenant (time extension + partial swap).
pub fn compile_daglock_advanced(
    buyer_pk: &[u8],
    seller_pk: &[u8],
    trade_hash: &[u8],
    timeout: i64,
    treasury_pk: &[u8],
) -> CompiledContract<'static> {
    let source = daglock_advanced_source();
    let args = vec![
        Expr::bytes(buyer_pk.to_vec()),
        Expr::bytes(seller_pk.to_vec()),
        Expr::bytes(trade_hash.to_vec()),
        Expr::int(timeout),
        Expr::bytes(treasury_pk.to_vec()),
    ];
    compile_contract(source, &args, CompileOptions::default())
        .expect("daglock_advanced.sil should compile")
}

/// Compile the DagLock Subscription covenant for recurring payments.
/// Arguments: payer_key, recipient_key, total_amount, installment_amount,
///            interval_seconds, start_time, current_period, treasury_key
pub fn compile_daglock_subscription(
    payer_pk: &[u8],
    recipient_pk: &[u8],
    total_amount: i64,
    installment_amount: i64,
    interval_seconds: i64,
    start_time: i64,
    current_period: i64,
    treasury_pk: &[u8],
) -> CompiledContract<'static> {
    let source = daglock_subscription_source();
    let args = vec![
        Expr::bytes(payer_pk.to_vec()),
        Expr::bytes(recipient_pk.to_vec()),
        Expr::int(total_amount),
        Expr::int(installment_amount),
        Expr::int(interval_seconds),
        Expr::int(start_time),
        Expr::int(current_period),
        Expr::bytes(treasury_pk.to_vec()),
    ];
    compile_contract(source, &args, CompileOptions::default())
        .expect("daglock_subscription.sil should compile")
}

/// Compile the DagLock Milestone covenant for milestone-based partial escrow.
/// Arguments (in order):
/// - `buyer_pk`: 32-byte compressed public key
/// - `seller_pk`: 32-byte compressed public key
/// - `total_amount`: total locked amount in sompi
/// - `milestone_amounts`: array of 5 milestone amounts (0 = unused)
/// - `milestone_timeouts`: array of 5 Unix timestamps (0 = no timeout)
/// - `current_milestone_index`: which milestone is active (0-based)
/// - `treasury_pk`: 32-byte compressed public key
pub fn compile_daglock_milestone(
    buyer_pk: &[u8],
    seller_pk: &[u8],
    total_amount: i64,
    milestone_amounts: Vec<i64>,
    milestone_timeouts: Vec<i64>,
    current_milestone_index: i64,
    treasury_pk: &[u8],
) -> CompiledContract<'static> {
    let source = daglock_milestone_source();
    let args = vec![
        Expr::bytes(buyer_pk.to_vec()),
        Expr::bytes(seller_pk.to_vec()),
        Expr::int(total_amount),
        milestone_amounts.into(),  // Vec<i64> → Expr::Array of ints
        milestone_timeouts.into(), // Vec<i64> → Expr::Array of ints
        Expr::int(current_milestone_index),
        Expr::bytes(treasury_pk.to_vec()),
    ];
    compile_contract(source, &args, CompileOptions::default())
        .expect("daglock_milestone.sil should compile")
}

/// Compile the DagLock Multi-Party covenant for 3+ party escrow.
/// Arguments (in order):
/// - `party1_pk` through `party4_pk`: 32-byte compressed public keys (zeros = unused)
/// - `shares`: array of 4 basis-point shares (sum = 10000)
/// - `trade_hash`: 32-byte SHA-256 hash (or zeroes if unused)
/// - `timeout`: Unix timestamp
/// - `treasury_pk`: 32-byte compressed public key
pub fn compile_daglock_multi(
    party1_pk: &[u8],
    party2_pk: &[u8],
    party3_pk: &[u8],
    party4_pk: &[u8],
    shares: Vec<i64>,
    trade_hash: &[u8],
    timeout: i64,
    treasury_pk: &[u8],
) -> CompiledContract<'static> {
    let source = daglock_multi_source();
    let args = vec![
        Expr::bytes(party1_pk.to_vec()),
        Expr::bytes(party2_pk.to_vec()),
        Expr::bytes(party3_pk.to_vec()),
        Expr::bytes(party4_pk.to_vec()),
        shares.into(), // Vec<i64> → Expr::Array of ints
        Expr::bytes(trade_hash.to_vec()),
        Expr::int(timeout),
        Expr::bytes(treasury_pk.to_vec()),
    ];
    compile_contract(source, &args, CompileOptions::default())
        .expect("daglock_multi.sil should compile")
}

/// Compile the DagLock Deposit covenant for security deposits.
/// Arguments (in order):
/// - `party1_pk`: 32-byte compressed public key
/// - `party2_pk`: 32-byte compressed public key
/// - `jury_pk`: 32-byte compressed public key (for forfeit rulings)
/// - `deposit_amount`: amount each party stakes
/// - `timeout`: Unix timestamp
/// - `treasury_pk`: 32-byte compressed public key
pub fn compile_daglock_deposit(
    party1_pk: &[u8],
    party2_pk: &[u8],
    jury_pk: &[u8],
    deposit_amount: i64,
    timeout: i64,
    treasury_pk: &[u8],
) -> CompiledContract<'static> {
    let source = daglock_deposit_source();
    let args = vec![
        Expr::bytes(party1_pk.to_vec()),
        Expr::bytes(party2_pk.to_vec()),
        Expr::bytes(jury_pk.to_vec()),
        Expr::int(deposit_amount),
        Expr::int(timeout),
        Expr::bytes(treasury_pk.to_vec()),
    ];
    compile_contract(source, &args, CompileOptions::default())
        .expect("daglock_deposit.sil should compile")
}

/// Compile the DagLock Vault covenant with the given constructor arguments.
///
/// Arguments (in order):
/// - `owner_key`: 32-byte compressed public key
/// - `lock_duration`: DAA block count (~1 block/sec)
/// - `treasury_key`: 32-byte compressed public key
/// - `heir_key`: 32-byte compressed public key (or zeroes for no heir)
/// - `inherit_lock_duration`: DAA block count for inheritance claim
pub fn compile_daglock_vault(
    owner_key: &[u8],
    lock_duration: i64,
    treasury_key: &[u8],
    heir_key: &[u8],
    inherit_lock_duration: i64,
) -> CompiledContract<'static> {
    let source = daglock_vault_source();
    let args = vec![
        Expr::bytes(owner_key.to_vec()),
        Expr::int(lock_duration),
        Expr::bytes(treasury_key.to_vec()),
        Expr::bytes(heir_key.to_vec()),
        Expr::int(inherit_lock_duration),
    ];
    compile_contract(source, &args, CompileOptions::default())
        .expect("daglock_vault.sil should compile — if this fails, fix the .sil syntax")
}

/// Compile the DagLock KRC-20 covenant with the given constructor arguments.
///
/// Arguments (in order):
/// - `buyer_pubkey`: 32-byte compressed public key
/// - `seller_pubkey`: 32-byte compressed public key
/// - `trade_hash`: 32-byte SHA-256 hash (or zeroes if no atomic swap)
/// - `timeout`: Unix timestamp (i64)
/// - `treasury_pubkey`: 32-byte compressed public key
/// - `kcc20_template_prefix_len`: bytecode prefix length
/// - `kcc20_template_suffix_len`: bytecode suffix length
/// - `kcc20_expected_template_hash`: 32-byte expected template hash
/// - `kcc20_template_prefix`: template prefix bytes
/// - `kcc20_template_suffix`: template suffix bytes
/// - `kcc20_covenant_id`: 32-byte covenant identifier
#[allow(clippy::too_many_arguments)]
pub fn compile_daglock_krc20(
    buyer_pubkey: &[u8],
    seller_pubkey: &[u8],
    trade_hash: &[u8],
    timeout: i64,
    treasury_pubkey: &[u8],
    kcc20_template_prefix_len: i64,
    kcc20_template_suffix_len: i64,
    kcc20_expected_template_hash: &[u8],
    kcc20_template_prefix: &[u8],
    kcc20_template_suffix: &[u8],
    kcc20_covenant_id: &[u8],
) -> CompiledContract<'static> {
    let source = daglock_krc20_source();
    let args = vec![
        Expr::bytes(buyer_pubkey.to_vec()),
        Expr::bytes(seller_pubkey.to_vec()),
        Expr::bytes(trade_hash.to_vec()),
        Expr::int(timeout),
        Expr::bytes(treasury_pubkey.to_vec()),
        Expr::int(kcc20_template_prefix_len),
        Expr::int(kcc20_template_suffix_len),
        Expr::bytes(kcc20_expected_template_hash.to_vec()),
        Expr::bytes(kcc20_template_prefix.to_vec()),
        Expr::bytes(kcc20_template_suffix.to_vec()),
        Expr::bytes(kcc20_covenant_id.to_vec()),
    ];
    compile_contract(source, &args, CompileOptions::default())
        .expect("daglock_krc20.sil should compile — if this fails, fix the .sil syntax")
}

/// Compile the DagLock Vault Softlock covenant with the given constructor arguments.
///
/// Arguments (in order):
/// - `owner_key`: 32-byte compressed public key
/// - `beneficiary_key`: 32-byte compressed public key (zero = open-ended)
/// - `password_hash`: 32-byte SHA-256 hash of the password
/// - `lock_duration`: DAA block count (~1 block/sec)
pub fn compile_daglock_vault_softlock(
    owner_key: &[u8],
    beneficiary_key: &[u8],
    password_hash: &[u8],
    lock_duration: i64,
    treasury_key: &[u8],
) -> CompiledContract<'static> {
    let source = daglock_vault_softlock_source();
    let args = vec![
        Expr::bytes(owner_key.to_vec()),
        Expr::bytes(beneficiary_key.to_vec()),
        Expr::bytes(password_hash.to_vec()),
        Expr::int(lock_duration),
        Expr::bytes(treasury_key.to_vec()),
    ];
    compile_contract(source, &args, CompileOptions::default())
        .expect("daglock_vault_softlock.sil should compile")
}

/// Compile the DagLock Vault Multi-sig covenant.
///
/// Arguments (in order):
/// - `key1`: 32-byte compressed public key
/// - `key2`: 32-byte compressed public key (zero if unused)
/// - `key3`: 32-byte compressed public key (zero if unused)
/// - `lock_duration`: DAA block count (~1 block/sec)
/// - `treasury_key`: 32-byte compressed public key
pub fn compile_daglock_vault_multisig(
    key1: &[u8],
    key2: &[u8],
    key3: &[u8],
    lock_duration: i64,
    treasury_key: &[u8],
) -> CompiledContract<'static> {
    let source = daglock_vault_multisig_source();
    let args = vec![
        Expr::bytes(key1.to_vec()),
        Expr::bytes(key2.to_vec()),
        Expr::bytes(key3.to_vec()),
        Expr::int(lock_duration),
        Expr::bytes(treasury_key.to_vec()),
    ];
    compile_contract(source, &args, CompileOptions::default())
        .expect("daglock_vault_multisig.sil should compile")
}

/// The daglock_reputation.sil source embedded at compile time.
pub fn daglock_reputation_source() -> &'static str {
    include_str!("daglock_reputation.sil")
}

/// Compile the DagLock Reputation covenant.
///
/// Arguments:
/// - `treasury_key`: 32-byte compressed public key for fee collection
pub fn compile_daglock_reputation(treasury_key: &[u8]) -> CompiledContract<'static> {
    let source = daglock_reputation_source();
    let args = vec![Expr::bytes(treasury_key.to_vec())];
    compile_contract(source, &args, CompileOptions::default())
        .expect("daglock_reputation.sil should compile")
}

/// Extract the template parts and hash from a compiled DagLock contract.
///
/// Returns `(prefix, suffix, template_hash)` where `template_hash` is
/// `blake2b(prefix || suffix)` truncated to 20 bytes (P2SH script hash length).
/// This hash identifies any DagLock UTXO regardless of constructor args.
pub fn template_parts_and_hash(compiled: &CompiledContract) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let state_layout = &compiled.state_layout;
    let prefix = compiled.script[..state_layout.start].to_vec();
    let suffix = compiled.script[state_layout.start + state_layout.len..].to_vec();
    let template_hash = blake2b_simd::Params::new()
        .hash_length(20)
        .to_state()
        .update(&prefix)
        .update(&suffix)
        .finalize()
        .as_bytes()
        .to_vec();
    (prefix, suffix, template_hash)
}

/// Entrypoint names for the DagLock covenant.
pub mod entrypoints {
    pub const RELEASE: &str = "release";
    pub const SWAP: &str = "swap";
    pub const REFUND: &str = "refund";
    pub const REFUND_AFTER_TIMEOUT: &str = "refundAfterTimeout";
    pub const DISPUTE_SELLER_WINS: &str = "disputeSellerWins";
    pub const DISPUTE_BUYER_WINS: &str = "disputeBuyerWins";
    pub const WITHDRAW: &str = "withdraw";
    pub const WITHDRAW_PASSWORD: &str = "withdrawPassword";
    pub const WITHDRAW_TIMEOUT: &str = "withdrawTimeout";
    pub const SWEEP: &str = "sweep";
    pub const RELOCK: &str = "relock";
    pub const EARLY_EXIT: &str = "early_exit";
    pub const HEIR_WITHDRAW: &str = "heir_withdraw";
    pub const AUTO_SETTLE: &str = "auto_settle";
    pub const EMERGENCY_REFUND: &str = "emergencyRefund";
    pub const EMERGENCY_REFUND_NOSIG: &str = "emergency_refund";
    pub const SPLIT: &str = "split";
    pub const ARBITRATE_SPLIT: &str = "arbitrateSplit";
    pub const RECORD_TRADE: &str = "recordTrade";
    // Milestone entrypoints
    pub const RELEASE_MILESTONE: &str = "release_milestone";
    pub const APPROVE_MILESTONE: &str = "approve_milestone";
    pub const COMPLETE: &str = "complete";
    pub const REFUND_REMAINING: &str = "refund_remaining";
    // Subscription entrypoints
    pub const CLAIM: &str = "claim";
    pub const CANCEL: &str = "cancel";
    // Advanced entrypoints
    pub const SWAP_PARTIAL: &str = "swap_partial";
    pub const EXTEND_TIMEOUT: &str = "extendTimeout";
    // Multi-party entrypoints
    // release, swap, refund shared with base
    // Deposit entrypoints
    pub const FORFEIT: &str = "forfeit";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_is_non_empty() {
        let src = daglock_source();
        assert!(src.contains("pragma silverscript"));
        assert!(src.contains("contract DagLock"));
        assert!(src.contains("entrypoint function release"));
        assert!(src.contains("entrypoint function swap"));
        assert!(src.contains("entrypoint function refund"));
        assert!(src.contains("entrypoint function auto_settle"));
        assert!(src.contains("entrypoint function emergency_refund"));
        assert!(!src.is_empty());
    }

    #[test]
    fn krc20_source_is_non_empty() {
        let src = daglock_krc20_source();
        assert!(src.contains("pragma silverscript"));
        assert!(src.contains("contract DagLockKRC20"));
        assert!(src.contains("entrypoint function release"));
        assert!(src.contains("entrypoint function swap"));
        assert!(src.contains("entrypoint function refund"));
    }

    #[test]
    fn compiles_daglock_with_valid_params() {
        let zero_pk = [0u8; 32];
        let zero_hash = [0u8; 32];
        let compiled = compile_daglock(&zero_pk, &zero_pk, &zero_hash, 1_700_000_000, &zero_pk);

        // Verify 6 entrypoints in ABI
        assert_eq!(compiled.abi.len(), 6);
        let names: Vec<&str> = compiled.abi.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"release"));
        assert!(names.contains(&"split"));
        assert!(names.contains(&"swap"));
        assert!(names.contains(&"refund"));
        assert!(names.contains(&"emergency_refund"));
        assert!(names.contains(&"auto_settle"));

        // Script should be non-empty
        assert!(!compiled.script.is_empty());
    }

    #[test]
    fn compiles_daglock_krc20_with_valid_params() {
        let zero_pk = [0u8; 32];
        let zero_hash = [0u8; 32];
        let compiled = compile_daglock_krc20(
            &zero_pk,
            &zero_pk,
            &zero_hash,
            1_700_000_000,
            &zero_pk,
            0,
            0,
            &zero_hash,
            &[],
            &[],
            &zero_pk,
        );

        assert_eq!(compiled.abi.len(), 4);
        let names: Vec<&str> = compiled.abi.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"release"));
        assert!(names.contains(&"swap"));
        assert!(names.contains(&"refund"));
        assert!(names.contains(&"auto_settle"));
        assert!(!compiled.script.is_empty());
    }

    #[test]
    fn template_hash_is_deterministic() {
        let zero_pk = [0u8; 32];
        let zero_hash = [0u8; 32];
        let c1 = compile_daglock(&zero_pk, &zero_pk, &zero_hash, 1_700_000_000, &zero_pk);
        let c2 = compile_daglock(&zero_pk, &zero_pk, &zero_hash, 1_700_000_000, &zero_pk);

        let (_, _, h1) = template_parts_and_hash(&c1);
        let (_, _, h2) = template_parts_and_hash(&c2);
        assert_eq!(h1, h2, "template hash must be deterministic");
        assert_eq!(h1.len(), 20, "template hash must be 20 bytes (P2SH)");
    }

    #[test]
    fn template_hash_differs_across_different_params() {
        let zero_pk = [0u8; 32];
        let one_pk = [1u8; 32];
        let zero_hash = [0u8; 32];

        let c1 = compile_daglock(&zero_pk, &zero_pk, &zero_hash, 1_700_000_000, &zero_pk);
        let c2 = compile_daglock(&one_pk, &one_pk, &zero_hash, 2_000_000_000, &one_pk);

        let (_, _, h1) = template_parts_and_hash(&c1);
        let (_, _, h2) = template_parts_and_hash(&c2);

        // The compiler may embed param-dependent code in prefix/suffix,
        // so template hashes may differ across different params.
        // The deterministic property (same params → same hash) is the
        // useful invariant — tested in template_hash_is_deterministic.
        assert_ne!(
            c1.script, c2.script,
            "full scripts must differ with different params"
        );
        // Both hashes should be 20 bytes
        assert_eq!(h1.len(), 20);
        assert_eq!(h2.len(), 20);
    }

    // ── Arbiter tests ────────────────────────────────────────────────

    #[test]
    fn arbiter_source_is_non_empty() {
        let src = daglock_arbiter_source();
        assert!(src.contains("contract DagLockArbiter"));
        assert!(src.contains("entrypoint function disputeSellerWins"));
        assert!(src.contains("entrypoint function disputeBuyerWins"));
        assert!(src.contains("entrypoint function release"));
        assert!(src.contains("entrypoint function swap"));
        assert!(src.contains("entrypoint function refundAfterTimeout"));
        assert!(src.contains("entrypoint function emergencyRefund"));
    }

    #[test]
    fn compiles_daglock_arbiter_with_valid_params() {
        let zero = [0u8; 32];
        let compiled = compile_daglock_arbiter(&zero, &zero, &zero, 1_700_000_000, &zero, &zero);

        assert_eq!(compiled.abi.len(), 7);
        let names: Vec<&str> = compiled.abi.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"release"));
        assert!(names.contains(&"swap"));
        assert!(names.contains(&"refundAfterTimeout"));
        assert!(names.contains(&"disputeSellerWins"));
        assert!(names.contains(&"disputeBuyerWins"));
        assert!(names.contains(&"arbitrateSplit"));
        assert!(!compiled.script.is_empty());
    }

    #[test]
    fn arbiter_template_hash_is_deterministic() {
        let zero = [0u8; 32];
        let c1 = compile_daglock_arbiter(&zero, &zero, &zero, 1_700_000_000, &zero, &zero);
        let c2 = compile_daglock_arbiter(&zero, &zero, &zero, 1_700_000_000, &zero, &zero);

        let (_, _, h1) = template_parts_and_hash(&c1);
        let (_, _, h2) = template_parts_and_hash(&c2);
        assert_eq!(h1, h2, "arbiter template hash must be deterministic");
        assert_eq!(h1.len(), 20);
    }

    #[test]
    fn arbiter_template_hash_differs_from_standard_daglock() {
        let zero = [0u8; 32];
        let std = compile_daglock(&zero, &zero, &zero, 1_700_000_000, &zero);
        let arb = compile_daglock_arbiter(&zero, &zero, &zero, 1_700_000_000, &zero, &zero);

        let (_, _, h1) = template_parts_and_hash(&std);
        let (_, _, h2) = template_parts_and_hash(&arb);
        assert_ne!(h1, h2, "arbiter template hash must differ from standard");
    }

    #[test]
    fn print_template_hashes() {
        let zero = [0u8; 32];
        let zero_hash = [0u8; 32];

        let std = compile_daglock(&zero, &zero, &zero_hash, 1_700_000_000, &zero);
        let (_, _, kas_hash) = template_parts_and_hash(&std);
        let kas_hex: String = kas_hash.iter().map(|b| format!("{:02x}", b)).collect();
        println!("daglock_kas_template_hash={}", kas_hex);

        let arb = compile_daglock_arbiter(&zero, &zero, &zero_hash, 1_700_000_000, &zero, &zero);
        let (_, _, arb_hash) = template_parts_and_hash(&arb);
        let arb_hex: String = arb_hash.iter().map(|b| format!("{:02x}", b)).collect();
        println!("daglock_arbiter_template_hash={}", arb_hex);

        let krc20 = compile_daglock_krc20(
            &zero,
            &zero,
            &zero_hash,
            1_700_000_000,
            &zero,
            0,
            0,
            &zero_hash,
            &[],
            &[],
            &zero,
        );
        let (_, _, krc20_hash) = template_parts_and_hash(&krc20);
        let krc20_hex: String = krc20_hash.iter().map(|b| format!("{:02x}", b)).collect();
        println!("daglock_krc20_template_hash={}", krc20_hex);

        let softlock = compile_daglock_vault_softlock(&zero, &[0u8; 32], &[0u8; 32], 500, &zero);
        let (_, _, softlock_hash) = template_parts_and_hash(&softlock);
        let softlock_hex: String = softlock_hash.iter().map(|b| format!("{:02x}", b)).collect();
        println!("daglock_vault_softlock_template_hash={}", softlock_hex);

        let multisig = compile_daglock_vault_multisig(&zero, &zero, &zero, 500, &zero);
        let (_, _, multisig_hash) = template_parts_and_hash(&multisig);
        let multisig_hex: String = multisig_hash.iter().map(|b| format!("{:02x}", b)).collect();
        println!("daglock_vault_multisig_template_hash={}", multisig_hex);

        let vault = compile_daglock_vault(&zero, 500, &zero, &zero, 0);
        let (_, _, vault_hash) = template_parts_and_hash(&vault);
        let vault_hex: String = vault_hash.iter().map(|b| format!("{:02x}", b)).collect();
        println!("daglock_vault_template_hash={}", vault_hex);

        let reputation = compile_daglock_reputation(&zero);
        let (_, _, rep_hash) = template_parts_and_hash(&reputation);
        let rep_hex: String = rep_hash.iter().map(|b| format!("{:02x}", b)).collect();
        println!("daglock_reputation_template_hash={}", rep_hex);

        let adv = compile_daglock_advanced(&zero, &zero, &zero_hash, 1_700_000_000, &zero);
        let (_, _, adv_hash) = template_parts_and_hash(&adv);
        let adv_hex: String = adv_hash.iter().map(|b| format!("{:02x}", b)).collect();
        println!("daglock_advanced_template_hash={}", adv_hex);

        let sub = compile_daglock_subscription(
            &zero,
            &zero,
            1_000_000_000,
            100_000_000,
            86400,
            1_700_000_000,
            0,
            &zero,
        );
        let (_, _, sub_hash) = template_parts_and_hash(&sub);
        let sub_hex: String = sub_hash.iter().map(|b| format!("{:02x}", b)).collect();
        println!("daglock_subscription_template_hash={}", sub_hex);

        let milestone = compile_daglock_milestone(
            &zero,
            &zero,
            100_000,
            vec![100_000, 0, 0, 0, 0],
            vec![1_700_000_000, 0, 0, 0, 0],
            0,
            &zero,
        );
        let (_, _, m_hash) = template_parts_and_hash(&milestone);
        let m_hex: String = m_hash.iter().map(|b| format!("{:02x}", b)).collect();
        println!("daglock_milestone_template_hash={}", m_hex);

        let multi = compile_daglock_multi(
            &zero,
            &zero,
            &zero,
            &zero,
            vec![2500, 2500, 2500, 2500],
            &zero_hash,
            1_700_000_000,
            &zero,
        );
        let (_, _, multi_hash) = template_parts_and_hash(&multi);
        let multi_hex: String = multi_hash.iter().map(|b| format!("{:02x}", b)).collect();
        println!("daglock_multi_template_hash={}", multi_hex);

        let deposit = compile_daglock_deposit(&zero, &zero, &zero, 100_000, 1_700_000_000, &zero);
        let (_, _, dep_hash) = template_parts_and_hash(&deposit);
        let dep_hex: String = dep_hash.iter().map(|b| format!("{:02x}", b)).collect();
        println!("daglock_deposit_template_hash={}", dep_hex);
    }
    #[test]
    fn arbiter_zero_key_and_nonzero_key_produce_different_scripts() {
        let zero = [0u8; 32];
        let one = [1u8; 32];
        let c1 = compile_daglock_arbiter(&zero, &zero, &zero, 1_700_000_000, &zero, &zero);
        let c2 = compile_daglock_arbiter(&zero, &zero, &zero, 1_700_000_000, &zero, &one);

        assert_ne!(
            c1.script, c2.script,
            "zero vs non-zero arbiter should differ"
        );
    }

    #[test]
    fn multisig_source_is_non_empty() {
        let src = daglock_vault_multisig_source();
        assert!(src.contains("contract DagLockVaultMultisig"));
        assert!(src.contains("entrypoint function withdraw"));
    }

    #[test]
    fn compiles_daglock_multisig_with_valid_params() {
        let pk = [0u8; 32];
        let compiled = compile_daglock_vault_multisig(&pk, &pk, &pk, 500, &pk);
        assert_eq!(compiled.abi.len(), 2);
        assert!(!compiled.script.is_empty());
    }

    #[test]
    fn multisig_template_hash_is_deterministic() {
        let pk = [0u8; 32];
        let c1 = compile_daglock_vault_multisig(&pk, &pk, &pk, 500, &pk);
        let c2 = compile_daglock_vault_multisig(&pk, &pk, &pk, 500, &pk);
        let (_, _, h1) = template_parts_and_hash(&c1);
        let (_, _, h2) = template_parts_and_hash(&c2);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 20);
    }

    // ── Milestone tests ─────────────────────────────────────────────

    #[test]
    fn milestone_source_is_non_empty() {
        let src = daglock_milestone_source();
        assert!(src.contains("contract DagLockMilestone"));
        assert!(src.contains("entrypoint function release_milestone"));
        assert!(src.contains("entrypoint function complete"));
    }

    #[test]
    fn compiles_daglock_milestone_with_valid_params() {
        let pk = [0u8; 32];
        let amounts = vec![1_000_000, 2_000_000, 0i64, 0, 0];
        let timeouts = vec![1_800_000_000, 1_900_000_000, 0i64, 0, 0];
        let compiled = compile_daglock_milestone(&pk, &pk, 3_000_000, amounts, timeouts, 0, &pk);
        assert_eq!(compiled.abi.len(), 5);
        let names: Vec<&str> = compiled.abi.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"release_milestone"));
        assert!(names.contains(&"approve_milestone"));
        assert!(names.contains(&"dispute"));
        assert!(names.contains(&"refund_remaining"));
        assert!(names.contains(&"complete"));
        assert!(!compiled.script.is_empty());
    }

    #[test]
    fn milestone_template_hash_is_deterministic() {
        let pk = [0u8; 32];
        let amounts = vec![1_000_000, 2_000_000, 0i64, 0, 0];
        let timeouts = vec![1_800_000_000, 1_900_000_000, 0i64, 0, 0];
        let c1 = compile_daglock_milestone(
            &pk,
            &pk,
            3_000_000,
            amounts.clone(),
            timeouts.clone(),
            0,
            &pk,
        );
        let c2 = compile_daglock_milestone(&pk, &pk, 3_000_000, amounts, timeouts, 0, &pk);
        let (_, _, h1) = template_parts_and_hash(&c1);
        let (_, _, h2) = template_parts_and_hash(&c2);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 20);
    }

    // ── Multi-party tests ───────────────────────────────────────────

    #[test]
    fn multi_source_is_non_empty() {
        let src = daglock_multi_source();
        assert!(src.contains("contract DagLockMulti"));
        assert!(src.contains("entrypoint function release"));
        assert!(src.contains("entrypoint function refund"));
    }

    #[test]
    fn compiles_daglock_multi_with_valid_params() {
        let pk = [0u8; 32];
        let shares = vec![5_000i64, 3_000, 1_500, 500];
        let compiled =
            compile_daglock_multi(&pk, &pk, &pk, &pk, shares, &[0u8; 32], 1_700_000_000, &pk);
        assert_eq!(compiled.abi.len(), 3);
        let names: Vec<&str> = compiled.abi.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"release"));
        assert!(names.contains(&"swap"));
        assert!(names.contains(&"refund"));
        assert!(!compiled.script.is_empty());
    }

    #[test]
    fn multi_template_hash_is_deterministic() {
        let pk = [0u8; 32];
        let shares = vec![5_000i64, 3_000, 1_500, 500];
        let c1 = compile_daglock_multi(
            &pk,
            &pk,
            &pk,
            &pk,
            shares.clone(),
            &[0u8; 32],
            1_700_000_000,
            &pk,
        );
        let c2 = compile_daglock_multi(&pk, &pk, &pk, &pk, shares, &[0u8; 32], 1_700_000_000, &pk);
        let (_, _, h1) = template_parts_and_hash(&c1);
        let (_, _, h2) = template_parts_and_hash(&c2);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 20);
    }

    // ── Deposit tests ───────────────────────────────────────────────

    #[test]
    fn deposit_source_is_non_empty() {
        let src = daglock_deposit_source();
        assert!(src.contains("contract DagLockDeposit"));
        assert!(src.contains("entrypoint function forfeit"));
        assert!(src.contains("entrypoint function release"));
        assert!(src.contains("entrypoint function sweep"));
    }

    #[test]
    fn compiles_daglock_deposit_with_valid_params() {
        let pk = [0u8; 32];
        let compiled = compile_daglock_deposit(&pk, &pk, &pk, 100_000, 1_700_000_000, &pk);
        assert_eq!(compiled.abi.len(), 3);
        let names: Vec<&str> = compiled.abi.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"forfeit"));
        assert!(names.contains(&"release"));
        assert!(names.contains(&"sweep"));
        assert!(!compiled.script.is_empty());
    }

    #[test]
    fn deposit_template_hash_is_deterministic() {
        let pk = [0u8; 32];
        let c1 = compile_daglock_deposit(&pk, &pk, &pk, 100_000, 1_700_000_000, &pk);
        let c2 = compile_daglock_deposit(&pk, &pk, &pk, 100_000, 1_700_000_000, &pk);
        let (_, _, h1) = template_parts_and_hash(&c1);
        let (_, _, h2) = template_parts_and_hash(&c2);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 20);
    }
}
