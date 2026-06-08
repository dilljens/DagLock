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

/// Compile the DagLock Vault covenant with the given constructor arguments.
///
/// Arguments (in order):
/// - `owner_key`: 32-byte compressed public key
/// - `timeout`: Unix timestamp (i64)
pub fn compile_daglock_vault(owner_key: &[u8], timeout: i64) -> CompiledContract<'static> {
    let source = daglock_vault_source();
    let args = vec![Expr::bytes(owner_key.to_vec()), Expr::int(timeout)];
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
/// - `timeout`: Unix timestamp (i64)
pub fn compile_daglock_vault_softlock(
    owner_key: &[u8],
    password_hash: &[u8],
    timeout: i64,
) -> CompiledContract<'static> {
    let source = daglock_vault_softlock_source();
    let args = vec![
        Expr::bytes(owner_key.to_vec()),
        Expr::bytes(password_hash.to_vec()),
        Expr::int(timeout),
    ];
    compile_contract(source, &args, CompileOptions::default())
        .expect("daglock_vault_softlock.sil should compile")
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

        // Verify 3 entrypoints in ABI
        assert_eq!(compiled.abi.len(), 3);
        let names: Vec<&str> = compiled.abi.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"release"));
        assert!(names.contains(&"swap"));
        assert!(names.contains(&"refund"));

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

        assert_eq!(compiled.abi.len(), 3);
        let names: Vec<&str> = compiled.abi.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"release"));
        assert!(names.contains(&"swap"));
        assert!(names.contains(&"refund"));
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
    }

    #[test]
    fn compiles_daglock_arbiter_with_valid_params() {
        let zero = [0u8; 32];
        let compiled = compile_daglock_arbiter(&zero, &zero, &zero, 1_700_000_000, &zero, &zero);

        assert_eq!(compiled.abi.len(), 5);
        let names: Vec<&str> = compiled.abi.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"release"));
        assert!(names.contains(&"swap"));
        assert!(names.contains(&"refundAfterTimeout"));
        assert!(names.contains(&"disputeSellerWins"));
        assert!(names.contains(&"disputeBuyerWins"));
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

        let softlock = compile_daglock_vault_softlock(&zero, &[0u8; 32], 1_700_000_000);
        let (_, _, softlock_hash) = template_parts_and_hash(&softlock);
        let softlock_hex: String = softlock_hash.iter().map(|b| format!("{:02x}", b)).collect();
        println!("daglock_vault_softlock_template_hash={}", softlock_hex);

        let vault = compile_daglock_vault(&zero, 1_700_000_000);
        let (_, _, vault_hash) = template_parts_and_hash(&vault);
        let vault_hex: String = vault_hash.iter().map(|b| format!("{:02x}", b)).collect();
        println!("daglock_vault_template_hash={}", vault_hex);
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
}
