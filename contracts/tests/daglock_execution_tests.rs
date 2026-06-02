//! Execution tests for DagLock covenant — runs each spending path through
//! the Kaspa script engine (TxScriptEngine) with real Schnorr signatures.
//!
//! These tests do NOT require a running node.

use daglock_contracts::{compile_daglock, entrypoints};
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
use sha2::{Digest, Sha256};

// ── Helpers ────────────────────────────────────────────────────────

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

/// Sign and execute a covenant spend in one step.

fn sha256_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

// ── Helper: build a release test and run it ────────────────────────

fn test_release(
    buyer_sig_valid: bool,
    seller_sig_valid: bool,
    wrong_fee: bool,
) -> Result<(), kaspa_txscript_errors::TxScriptError> {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let zero_hash = [0u8; 32];
    let timeout = 2_000_000_000i64;

    let compiled = compile_daglock(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &zero_hash,
        timeout,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 200_000;
    let correct_fee = input_value / 200;
    let fee_amount = if wrong_fee {
        correct_fee + 1
    } else {
        correct_fee
    };
    let send_amount = input_value - fee_amount;

    let recipient = random_keypair();
    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&recipient))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    // Build the transaction with empty sigscript for sighash computation
    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([1u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    let tx = Transaction::new(
        1,
        vec![input],
        outputs.clone(),
        0,
        Default::default(),
        0,
        vec![],
    );
    let utxo = UtxoEntry::new(
        input_value,
        ScriptPublicKey::new(0, compiled.script.clone().into()),
        0,
        false,
        None,
    );
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    // Compute sighash and sign
    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();

    let buyer_sig = if buyer_sig_valid {
        let sig = buyer.sign_schnorr(msg);
        let mut s = Vec::with_capacity(65);
        s.extend_from_slice(sig.as_ref().as_slice());
        s.push(SIG_HASH_ALL.to_u8());
        s
    } else {
        vec![0u8; 65]
    };

    let seller_sig = if seller_sig_valid {
        let sig = seller.sign_schnorr(msg);
        let mut s = Vec::with_capacity(65);
        s.extend_from_slice(sig.as_ref().as_slice());
        s.push(SIG_HASH_ALL.to_u8());
        s
    } else {
        vec![0u8; 65]
    };

    let sigscript = compiled
        .build_sig_script(
            entrypoints::RELEASE,
            vec![
                daglock_contracts::silverscript_lang::ast::Expr::bytes(buyer_sig),
                daglock_contracts::silverscript_lang::ast::Expr::bytes(seller_sig),
            ],
        )
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags {
        covenants_enabled: true,
        sigop_script_units: 0.into(),
    };

    let ver_tx = mtx.as_verifiable();
    let mut vm =
        TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    vm.execute()
}

// ── Tests ──────────────────────────────────────────────────────────

#[test]
fn release_path_succeeds_with_both_signatures() {
    let result = test_release(true, true, false);
    assert!(
        result.is_ok(),
        "release with both signatures failed: {}",
        result.unwrap_err()
    );
}

#[test]
fn release_fails_with_only_buyer_signature() {
    let result = test_release(true, false, false);
    assert!(result.is_err(), "release with only buyer sig should fail");
}

#[test]
fn swap_path_succeeds_with_correct_preimage() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let secret = b"daglock-atomic-swap-secret-v1";
    let trade_hash = sha256_hash(secret);

    let compiled = compile_daglock(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &trade_hash,
        2_000_000_000,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 200_000;
    let fee = input_value / 200;
    let send = input_value - fee;

    let recipient = random_keypair();
    let outputs = vec![
        TransactionOutput::new(send, p2pk_script(&pubkey_bytes(&recipient))),
        TransactionOutput::new(fee, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([3u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    let tx = Transaction::new(
        1,
        vec![input],
        outputs.clone(),
        0,
        Default::default(),
        0,
        vec![],
    );
    let utxo = UtxoEntry::new(
        input_value,
        ScriptPublicKey::new(0, compiled.script.clone().into()),
        0,
        false,
        None,
    );
    let mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    // Build sigscript with preimage (no signing needed for swap path)
    let sigscript = compiled
        .build_sig_script(
            entrypoints::SWAP,
            vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(
                secret.to_vec(),
            )],
        )
        .expect("build_sig_script");

    let reused = SigHashReusedValuesUnsync::new();
    let mut mtx = mtx;
    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags {
        covenants_enabled: true,
        sigop_script_units: 0.into(),
    };

    let ver_tx = mtx.as_verifiable();
    let mut vm =
        TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(
        result.is_ok(),
        "swap with correct preimage failed: {}",
        result.unwrap_err()
    );
}

#[test]
fn swap_fails_with_wrong_preimage() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let secret = b"correct-secret";
    let trade_hash = sha256_hash(secret);
    let wrong_secret = b"wrong-secret-xxxxx";

    let compiled = compile_daglock(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &trade_hash,
        2_000_000_000,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 200_000;
    let fee = input_value / 200;
    let send = input_value - fee;

    let recipient = random_keypair();
    let outputs = vec![
        TransactionOutput::new(send, p2pk_script(&pubkey_bytes(&recipient))),
        TransactionOutput::new(fee, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([3u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    let tx = Transaction::new(
        1,
        vec![input],
        outputs.clone(),
        0,
        Default::default(),
        0,
        vec![],
    );
    let utxo = UtxoEntry::new(
        input_value,
        ScriptPublicKey::new(0, compiled.script.clone().into()),
        0,
        false,
        None,
    );
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let sigscript = compiled
        .build_sig_script(
            entrypoints::SWAP,
            vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(
                wrong_secret.to_vec(),
            )],
        )
        .expect("build_sig_script");

    let reused = SigHashReusedValuesUnsync::new();
    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags {
        covenants_enabled: true,
        sigop_script_units: 0.into(),
    };

    let ver_tx = mtx.as_verifiable();
    let mut vm =
        TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_err(), "swap with wrong preimage should fail");
}

#[test]
fn refund_succeeds_after_timeout() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let timeout: i64 = 1_600_000_000;
    let zero_hash = [0u8; 32];

    let compiled = compile_daglock(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &zero_hash,
        timeout,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 200_000;
    let outputs = vec![TransactionOutput::new(
        input_value,
        p2pk_script(&pubkey_bytes(&buyer)),
    )];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([5u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    let tx = Transaction::new(
        1,
        vec![input],
        outputs.clone(),
        timeout as u64,
        Default::default(),
        0,
        vec![],
    );
    let utxo = UtxoEntry::new(
        input_value,
        ScriptPublicKey::new(0, compiled.script.clone().into()),
        0,
        false,
        None,
    );
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let sig_raw = buyer.sign_schnorr(msg);
    let mut buyer_sig = Vec::with_capacity(65);
    buyer_sig.extend_from_slice(sig_raw.as_ref().as_slice());
    buyer_sig.push(SIG_HASH_ALL.to_u8());

    let sigscript = compiled
        .build_sig_script(
            entrypoints::REFUND,
            vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(
                buyer_sig,
            )],
        )
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags {
        covenants_enabled: true,
        sigop_script_units: 0.into(),
    };

    let ver_tx = mtx.as_verifiable();
    let mut vm =
        TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(
        result.is_ok(),
        "refund after timeout failed: {}",
        result.unwrap_err()
    );
}

#[test]
fn refund_fails_before_timeout() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let timeout: i64 = 3_000_000_000;
    let zero_hash = [0u8; 32];

    let compiled = compile_daglock(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &zero_hash,
        timeout,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 200_000;
    let outputs = vec![TransactionOutput::new(
        input_value,
        p2pk_script(&pubkey_bytes(&buyer)),
    )];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([6u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    let tx = Transaction::new(
        1,
        vec![input],
        outputs.clone(),
        0,
        Default::default(),
        0,
        vec![],
    );
    let utxo = UtxoEntry::new(
        input_value,
        ScriptPublicKey::new(0, compiled.script.clone().into()),
        0,
        false,
        None,
    );
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let sig_raw = buyer.sign_schnorr(msg);
    let mut buyer_sig = Vec::with_capacity(65);
    buyer_sig.extend_from_slice(sig_raw.as_ref().as_slice());
    buyer_sig.push(SIG_HASH_ALL.to_u8());

    let sigscript = compiled
        .build_sig_script(
            entrypoints::REFUND,
            vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(
                buyer_sig,
            )],
        )
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags {
        covenants_enabled: true,
        sigop_script_units: 0.into(),
    };

    let ver_tx = mtx.as_verifiable();
    let mut vm =
        TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_err(), "refund before timeout should fail");
}

#[test]
fn release_fails_with_wrong_fee_amount() {
    let result = test_release(true, true, true);
    assert!(result.is_err(), "release with wrong fee should fail");
}
