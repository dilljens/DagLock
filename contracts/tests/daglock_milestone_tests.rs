//! Execution tests for DagLock Milestone covenant.
//!
//! Tests all five entrypoints:
//! - release_milestone: seller claims after timeout
//! - approve_milestone: buyer releases early
//! - dispute: buyer halts, reclaims everything
//! - refund_remaining: buyer reclaims after current milestone timeout
//! - complete: mutual full release

use daglock_contracts::{
    compile_daglock_milestone, entrypoints,
    silverscript_lang::ast::Expr,
};
use kaspa_consensus_core::hashing::sighash::{
    calc_schnorr_signature_hash, SigHashReusedValuesUnsync,
};
use kaspa_consensus_core::hashing::sighash_type::SIG_HASH_ALL;
use kaspa_consensus_core::tx::{
    MutableTransaction, ScriptPublicKey, Transaction, TransactionId, TransactionInput,
    TransactionOutpoint, TransactionOutput, UtxoEntry, VerifiableTransaction,
};
use kaspa_txscript::caches::Cache;
use kaspa_txscript::opcodes::codes::{OpCheckSig, OpData32};
use kaspa_txscript::script_builder::ScriptBuilder;
use kaspa_txscript::{EngineCtx, EngineFlags, TxScriptEngine};
use rand::RngCore;
use secp256k1::{Keypair, Secp256k1, SecretKey};

// ── Helpers ──────────────────────────────────────────────────────────

fn random_keypair() -> Keypair {
    let secp = Secp256k1::new();
    let mut rng = rand::thread_rng();
    let mut sk_bytes = [0u8; 32];
    loop {
        rng.fill_bytes(&mut sk_bytes);
        if let Ok(sk) = SecretKey::from_slice(&sk_bytes) {
            return Keypair::from_secret_key(&secp, &sk);
        }
    }
}

fn pubkey_bytes(kp: &Keypair) -> Vec<u8> {
    kp.x_only_public_key().0.serialize().to_vec()
}

fn p2pk_script(pubkey: &[u8]) -> ScriptPublicKey {
    let script = ScriptBuilder::new()
        .add_data(pubkey)
        .unwrap()
        .add_op(OpCheckSig)
        .unwrap()
        .drain();
    ScriptPublicKey::new(0, script.into())
}

/// Build a minimal P2PK relock script in the exact format that
/// `SpkEncoding::to_bytes()` produces for comparison against
/// `tx.outputs[i].scriptPubKey` inside the covenant.
///
/// Format: [version: u16 BE (0x0000)] [OpData32 (0x20)] [32-byte pubkey] [OpCheckSig (0xAC)]
/// Total: 36 bytes — well under the 520-byte OP_DATA limit.
///
/// The corresponding ScriptPublicKey for TransactionOutput uses the inner
/// 34 bytes (OpData32 + pubkey + OpCheckSig) as the script field:
/// `ScriptPublicKey::new(0, relock_inner_script(pubkey).into())`
fn relock_full_bytes(pubkey: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(36);
    bytes.extend_from_slice(&0u16.to_be_bytes()); // version
    bytes.push(OpData32);                          // push 32 bytes
    bytes.extend_from_slice(&pubkey[..32]);        // pubkey
    bytes.push(OpCheckSig);                        // OP_CHECKSIG
    bytes
}

/// The inner 34 bytes that go into the ScriptPublicKey's script field.
fn relock_inner_bytes(pubkey: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(34);
    bytes.push(OpData32);
    bytes.extend_from_slice(&pubkey[..32]);
    bytes.push(OpCheckSig);
    bytes
}

// ── Constants ────────────────────────────────────────────────────────

const TOTAL: i64 = 10_000_000_000;
const FEE_DENOM: u64 = 200;

fn make_amounts() -> Vec<i64> { vec![2_000_000_000, 3_000_000_000, 5_000_000_000, 0, 0] }
fn make_timeouts() -> Vec<i64> { vec![100_000, 200_000, 300_000, 0, 0] }

// ── Tests ────────────────────────────────────────────────────────────

/// Shared helper for release_milestone / approve_milestone tests.
/// Uses a minimal P2PK script as the re-lock output — this is safe because
/// the covenant only checks `tx.outputs[2].scriptPubKey == byte[](newCovenantScript)`.
/// The P2PK bytes are constructed to match SpkEncoding::to_bytes() format exactly.
fn test_milestone_release(
    use_buyer_sig: bool,
) -> Result<(), kaspa_txscript_errors::TxScriptError> {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();

    let covenant = compile_daglock_milestone(
        &pubkey_bytes(&buyer), &pubkey_bytes(&seller),
        TOTAL, make_amounts(), make_timeouts(), 0,
        &pubkey_bytes(&treasury),
    );

    let relock = relock_full_bytes(&pubkey_bytes(&buyer));
    let relock_inner = relock_inner_bytes(&pubkey_bytes(&buyer));

    let input_value: u64 = TOTAL as u64;
    let milestone_amount: u64 = 2_000_000_000;
    let fee = milestone_amount / FEE_DENOM;
    let net = milestone_amount - fee;
    let remaining = input_value - milestone_amount;

    let outputs = vec![
        TransactionOutput::new(net, p2pk_script(&pubkey_bytes(&seller))),
        TransactionOutput::new(fee, p2pk_script(&pubkey_bytes(&treasury))),
        TransactionOutput::new(remaining, ScriptPublicKey::new(0, relock_inner.into())),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([10u8; 32]), 0),
        vec![], 0, 0u8,
    );
    // Set lock_time past the milestone timeout so `tx.time >= milestoneTimeouts[0]` passes
    let now: u64 = 100_001;
    let tx = Transaction::new(1, vec![input], outputs, now, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, covenant.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();

    let sig = {
        let kp = if use_buyer_sig { &buyer } else { &seller };
        let s = kp.sign_schnorr(msg);
        let mut buf = Vec::with_capacity(65);
        buf.extend_from_slice(s.as_ref().as_slice());
        buf.push(SIG_HASH_ALL.to_u8());
        buf
    };

    let entrypoint = if use_buyer_sig { entrypoints::APPROVE_MILESTONE } else { entrypoints::RELEASE_MILESTONE };
    let sigscript = covenant
        .build_sig_script(entrypoint, vec![
            Expr::bytes(sig),
            Expr::int(0),
            Expr::bytes(relock),  // full version+script bytes matching to_bytes()
        ])
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    vm.execute()
}

#[test]
fn milestone_release_after_timeout_succeeds() {
    let result = test_milestone_release(false);
    assert!(result.is_ok(), "release milestone 0: {:?}", result.err());
}

#[test]
fn milestone_approve_by_buyer_succeeds() {
    let result = test_milestone_release(true);
    assert!(result.is_ok(), "approve milestone 0: {:?}", result.err());
}

#[test]
fn milestone_release_before_timeout_fails() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let future_timeouts = vec![9_999_999_999i64, 9_999_999_999, 9_999_999_999, 0, 0];

    let covenant = compile_daglock_milestone(
        &pubkey_bytes(&buyer), &pubkey_bytes(&seller),
        TOTAL, make_amounts(), future_timeouts, 0,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = TOTAL as u64;
    let fee = 2_000_000_000u64 / FEE_DENOM;
    let net = 2_000_000_000u64 - fee;
    let remaining = input_value - 2_000_000_000u64;

    let outputs = vec![
        TransactionOutput::new(net, p2pk_script(&pubkey_bytes(&seller))),
        TransactionOutput::new(fee, p2pk_script(&pubkey_bytes(&treasury))),
        TransactionOutput::new(remaining, p2pk_script(&pubkey_bytes(&buyer))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([11u8; 32]), 0),
        vec![], 0, 0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, covenant.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let seller_sig = {
        let sig = seller.sign_schnorr(msg);
        let mut s = Vec::with_capacity(65);
        s.extend_from_slice(sig.as_ref().as_slice());
        s.push(SIG_HASH_ALL.to_u8());
        s
    };

    let sigscript = covenant
        .build_sig_script(entrypoints::RELEASE_MILESTONE, vec![
            Expr::bytes(seller_sig),
            Expr::int(0),
            Expr::bytes(vec![]),
        ])
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_err(), "release before timeout should fail");
}

#[test]
fn milestone_dispute_returns_all_to_buyer() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();

    let covenant = compile_daglock_milestone(
        &pubkey_bytes(&buyer), &pubkey_bytes(&seller),
        TOTAL, make_amounts(), make_timeouts(), 1,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = TOTAL as u64;
    let outputs = vec![
        TransactionOutput::new(input_value, p2pk_script(&pubkey_bytes(&buyer))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([13u8; 32]), 0),
        vec![], 0, 0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, covenant.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let buyer_sig = {
        let sig = buyer.sign_schnorr(msg);
        let mut s = Vec::with_capacity(65);
        s.extend_from_slice(sig.as_ref().as_slice());
        s.push(SIG_HASH_ALL.to_u8());
        s
    };

    let sigscript = covenant
        .build_sig_script("dispute", vec![Expr::bytes(buyer_sig)])
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_ok(), "dispute: {:?}", result.err());
}

#[test]
fn milestone_refund_remaining_after_timeout_succeeds() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();

    let covenant = compile_daglock_milestone(
        &pubkey_bytes(&buyer), &pubkey_bytes(&seller),
        TOTAL, make_amounts(), make_timeouts(), 0,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = TOTAL as u64;
    let outputs = vec![
        TransactionOutput::new(input_value, p2pk_script(&pubkey_bytes(&buyer))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([14u8; 32]), 0),
        vec![], 0, 0u8,
    );
    let now: u64 = 100_001;
    let tx = Transaction::new(1, vec![input], outputs, now, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, covenant.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let buyer_sig = {
        let sig = buyer.sign_schnorr(msg);
        let mut s = Vec::with_capacity(65);
        s.extend_from_slice(sig.as_ref().as_slice());
        s.push(SIG_HASH_ALL.to_u8());
        s
    };

    let sigscript = covenant
        .build_sig_script(entrypoints::REFUND_REMAINING, vec![Expr::bytes(buyer_sig)])
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_ok(), "refund_remaining: {:?}", result.err());
}

#[test]
fn milestone_complete_mutual_release_succeeds() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();

    let covenant = compile_daglock_milestone(
        &pubkey_bytes(&buyer), &pubkey_bytes(&seller),
        TOTAL, make_amounts(), make_timeouts(), 2,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = TOTAL as u64;
    let fee = input_value / FEE_DENOM;
    let send_amount = input_value - fee;

    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&seller))),
        TransactionOutput::new(fee, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([15u8; 32]), 0),
        vec![], 0, 0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, covenant.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let buyer_sig = {
        let sig = buyer.sign_schnorr(msg);
        let mut s = Vec::with_capacity(65);
        s.extend_from_slice(sig.as_ref().as_slice());
        s.push(SIG_HASH_ALL.to_u8());
        s
    };
    let seller_sig = {
        let sig = seller.sign_schnorr(msg);
        let mut s = Vec::with_capacity(65);
        s.extend_from_slice(sig.as_ref().as_slice());
        s.push(SIG_HASH_ALL.to_u8());
        s
    };

    let sigscript = covenant
        .build_sig_script(entrypoints::COMPLETE, vec![
            Expr::bytes(buyer_sig),
            Expr::bytes(seller_sig),
        ])
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_ok(), "complete: {:?}", result.err());
}

#[test]
fn milestone_abi_has_correct_entrypoints() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();

    let covenant = compile_daglock_milestone(
        &pubkey_bytes(&buyer), &pubkey_bytes(&seller),
        TOTAL, make_amounts(), make_timeouts(), 0,
        &pubkey_bytes(&treasury),
    );

    let names: Vec<&str> = covenant.abi.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"release_milestone"));
    assert!(names.contains(&"approve_milestone"));
    assert!(names.contains(&"dispute"));
    assert!(names.contains(&"refund_remaining"));
    assert!(names.contains(&"complete"));
    assert_eq!(covenant.abi.len(), 5);
    assert!(!covenant.script.is_empty());
}
