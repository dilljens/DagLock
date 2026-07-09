//! Execution tests for DagLock Subscription covenant.
//!
//! Tests all three entrypoints:
//! - claim: recipient claims an installment, remaining is re-locked
//! - cancel: payer cancels, remaining minus fee returned to payer
//! - release: mutual release, all remaining to recipient minus fee

use daglock_contracts::{
    compile_daglock_subscription, entrypoints,
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
use kaspa_txscript::opcodes::codes::OpCheckSig;
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

/// Prepend the ScriptPublicKey version bytes (u16 big-endian) to a script.
/// This matches the format produced by SpkEncoding::to_bytes() so that
/// comparisons against tx.outputs[i].scriptPubKey in the covenant work.
fn with_version(script: Vec<u8>) -> Vec<u8> {
    let version: u16 = 0;
    version.to_be_bytes().into_iter().chain(script.into_iter()).collect()
}

// ── Constants ────────────────────────────────────────────────────────

const TOTAL: i64 = 1_000_000_000;
const INSTALLMENT: i64 = 200_000_000;
const INTERVAL: i64 = 86400;
const START_TIME: i64 = 1_000_000_000;
const FEE_DENOM: i64 = 200;

// ── Tests ────────────────────────────────────────────────────────────

#[test]
fn subscription_claim_first_installment_succeeds() {
    let payer = random_keypair();
    let recipient = random_keypair();
    let treasury = random_keypair();

    let covenant = compile_daglock_subscription(
        &pubkey_bytes(&payer),
        &pubkey_bytes(&recipient),
        TOTAL, INSTALLMENT, INTERVAL, START_TIME, 0,
        &pubkey_bytes(&treasury),
    );

    let next = compile_daglock_subscription(
        &pubkey_bytes(&payer),
        &pubkey_bytes(&recipient),
        TOTAL, INSTALLMENT, INTERVAL, START_TIME, 1,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = TOTAL as u64;
    let fee = INSTALLMENT / FEE_DENOM;
    let net = INSTALLMENT - fee;
    let remaining = input_value - INSTALLMENT as u64;

    let outputs = vec![
        TransactionOutput::new(net as u64, p2pk_script(&pubkey_bytes(&recipient))),
        TransactionOutput::new(fee as u64, p2pk_script(&pubkey_bytes(&treasury))),
        TransactionOutput::new(remaining, ScriptPublicKey::new(0, next.script.clone().into())),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([1u8; 32]), 0),
        vec![], 0, 0u8,
    );
    let now = START_TIME as u64 + 1; // satisfy tx.time >= startTime + (currentPeriod * intervalSeconds)
    let tx = Transaction::new(1, vec![input], outputs, now, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, covenant.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let recipient_sig = {
        let sig = recipient.sign_schnorr(msg);
        let mut s = Vec::with_capacity(65);
        s.extend_from_slice(sig.as_ref().as_slice());
        s.push(SIG_HASH_ALL.to_u8());
        s
    };

    let sigscript = covenant
        .build_sig_script(entrypoints::CLAIM, vec![
            Expr::bytes(recipient_sig),
            Expr::bytes(with_version(next.script.clone())),
        ])
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_ok(), "claim first installment: {:?}", result.err());
}

#[test]
fn subscription_claim_before_timeout_fails() {
    let payer = random_keypair();
    let recipient = random_keypair();
    let treasury = random_keypair();
    let future_start = 9_999_999_999i64;

    let covenant = compile_daglock_subscription(
        &pubkey_bytes(&payer), &pubkey_bytes(&recipient),
        TOTAL, INSTALLMENT, INTERVAL, future_start, 0,
        &pubkey_bytes(&treasury),
    );

    let next = compile_daglock_subscription(
        &pubkey_bytes(&payer), &pubkey_bytes(&recipient),
        TOTAL, INSTALLMENT, INTERVAL, future_start, 1,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = TOTAL as u64;
    let fee = INSTALLMENT / FEE_DENOM;
    let net = INSTALLMENT - fee;
    let remaining = input_value - INSTALLMENT as u64;

    let outputs = vec![
        TransactionOutput::new(net as u64, p2pk_script(&pubkey_bytes(&recipient))),
        TransactionOutput::new(fee as u64, p2pk_script(&pubkey_bytes(&treasury))),
        TransactionOutput::new(remaining, ScriptPublicKey::new(0, next.script.clone().into())),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([2u8; 32]), 0),
        vec![], 0, 0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, covenant.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let recipient_sig = {
        let sig = recipient.sign_schnorr(msg);
        let mut s = Vec::with_capacity(65);
        s.extend_from_slice(sig.as_ref().as_slice());
        s.push(SIG_HASH_ALL.to_u8());
        s
    };

    let sigscript = covenant
        .build_sig_script(entrypoints::CLAIM, vec![
            Expr::bytes(recipient_sig),
            Expr::bytes(with_version(next.script.clone())),
        ])
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_err(), "claim before start_time should fail");
}

#[test]
fn subscription_claim_last_installment_no_remainder() {
    let payer = random_keypair();
    let recipient = random_keypair();
    let treasury = random_keypair();
    let single = INSTALLMENT;

    let covenant = compile_daglock_subscription(
        &pubkey_bytes(&payer), &pubkey_bytes(&recipient),
        single, single, INTERVAL, START_TIME, 0,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = single as u64;
    let fee = single / FEE_DENOM;
    let net = single - fee;

    let outputs = vec![
        TransactionOutput::new(net as u64, p2pk_script(&pubkey_bytes(&recipient))),
        TransactionOutput::new(fee as u64, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([3u8; 32]), 0),
        vec![], 0, 0u8,
    );
    let now = START_TIME as u64 + 1;
    let tx = Transaction::new(1, vec![input], outputs, now, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, covenant.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let recipient_sig = {
        let sig = recipient.sign_schnorr(msg);
        let mut s = Vec::with_capacity(65);
        s.extend_from_slice(sig.as_ref().as_slice());
        s.push(SIG_HASH_ALL.to_u8());
        s
    };

    let sigscript = covenant
        .build_sig_script(entrypoints::CLAIM, vec![
            Expr::bytes(recipient_sig),
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
    assert!(result.is_ok(), "claim last installment: {:?}", result.err());
}

#[test]
fn subscription_cancel_succeeds() {
    let payer = random_keypair();
    let recipient = random_keypair();
    let treasury = random_keypair();

    let covenant = compile_daglock_subscription(
        &pubkey_bytes(&payer), &pubkey_bytes(&recipient),
        TOTAL, INSTALLMENT, INTERVAL, START_TIME, 0,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = TOTAL as u64;
    let fee = input_value / FEE_DENOM as u64;
    let return_amount = input_value - fee;

    let outputs = vec![
        TransactionOutput::new(return_amount, p2pk_script(&pubkey_bytes(&payer))),
        TransactionOutput::new(fee, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([4u8; 32]), 0),
        vec![], 0, 0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, covenant.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let payer_sig = {
        let sig = payer.sign_schnorr(msg);
        let mut s = Vec::with_capacity(65);
        s.extend_from_slice(sig.as_ref().as_slice());
        s.push(SIG_HASH_ALL.to_u8());
        s
    };

    let sigscript = covenant
        .build_sig_script(entrypoints::CANCEL, vec![Expr::bytes(payer_sig)])
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_ok(), "cancel: {:?}", result.err());
}

#[test]
fn subscription_release_succeeds() {
    let payer = random_keypair();
    let recipient = random_keypair();
    let treasury = random_keypair();

    let covenant = compile_daglock_subscription(
        &pubkey_bytes(&payer), &pubkey_bytes(&recipient),
        TOTAL, INSTALLMENT, INTERVAL, START_TIME, 0,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = TOTAL as u64;
    let fee = input_value / FEE_DENOM as u64;
    let send_amount = input_value - fee;

    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&recipient))),
        TransactionOutput::new(fee, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([5u8; 32]), 0),
        vec![], 0, 0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, covenant.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let payer_sig = {
        let sig = payer.sign_schnorr(msg);
        let mut s = Vec::with_capacity(65);
        s.extend_from_slice(sig.as_ref().as_slice());
        s.push(SIG_HASH_ALL.to_u8());
        s
    };
    let recipient_sig = {
        let sig = recipient.sign_schnorr(msg);
        let mut s = Vec::with_capacity(65);
        s.extend_from_slice(sig.as_ref().as_slice());
        s.push(SIG_HASH_ALL.to_u8());
        s
    };

    let sigscript = covenant
        .build_sig_script(entrypoints::RELEASE, vec![
            Expr::bytes(payer_sig),
            Expr::bytes(recipient_sig),
        ])
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_ok(), "release: {:?}", result.err());
}

#[test]
fn subscription_abi_has_correct_entrypoints() {
    let payer = random_keypair();
    let recipient = random_keypair();
    let treasury = random_keypair();

    let covenant = compile_daglock_subscription(
        &pubkey_bytes(&payer), &pubkey_bytes(&recipient),
        TOTAL, INSTALLMENT, INTERVAL, START_TIME, 0,
        &pubkey_bytes(&treasury),
    );

    let names: Vec<&str> = covenant.abi.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"claim"));
    assert!(names.contains(&"cancel"));
    assert!(names.contains(&"release"));
    assert_eq!(covenant.abi.len(), 3);
    assert!(!covenant.script.is_empty());
}
