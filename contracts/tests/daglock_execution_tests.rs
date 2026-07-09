//! Execution tests for DagLock covenant — runs each spending path through
//! the Kaspa script engine (TxScriptEngine) with real Schnorr signatures.
//!
//! These tests do NOT require a running node.

use daglock_contracts::{
    compile_daglock, compile_daglock_advanced, compile_daglock_deposit, compile_daglock_multi,
    entrypoints, silverscript_lang::ast::Expr,
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

// ── Auto-settle tests ───────────────────────────────────────────────

#[test]
fn auto_settle_succeeds_after_timeout() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let timeout = 1_700_000_000i64;
    let now: u64 = (timeout + 1) as u64;
    let input_value: u64 = 200_000;
    let fee = input_value / 200;
    let send = input_value - fee;

    let compiled = compile_daglock(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &[0u8; 32], // no trade hash
        timeout,
        &pubkey_bytes(&treasury),
    );

    let outputs = vec![
        TransactionOutput::new(send, p2pk_script(&pubkey_bytes(&seller))),
        TransactionOutput::new(fee, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([10u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs.clone(), now, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let sigscript = compiled
        .build_sig_script(entrypoints::AUTO_SETTLE, vec![])
        .expect("build_sig_script");

    let reused = SigHashReusedValuesUnsync::new();
    let mut mtx = mtx;
    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };

    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_ok(), "auto_settle after timeout should succeed: {:?}", result.err());
}

#[test]
fn auto_settle_fails_before_timeout() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let timeout = 1_700_000_000i64;
    let now: u64 = (timeout - 1) as u64;
    let input_value: u64 = 200_000;
    let fee = input_value / 200;
    let send = input_value - fee;

    let compiled = compile_daglock(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &[0u8; 32],
        timeout,
        &pubkey_bytes(&treasury),
    );

    let outputs = vec![
        TransactionOutput::new(send, p2pk_script(&pubkey_bytes(&seller))),
        TransactionOutput::new(fee, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([11u8; 32]), 0),
        vec![], 0, 0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs.clone(), now, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let sigscript = compiled.build_sig_script(entrypoints::AUTO_SETTLE, vec![]).expect("build_sig_script");
    let reused = SigHashReusedValuesUnsync::new();
    let mut mtx = mtx;
    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };

    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_err(), "auto_settle before timeout should fail");
}

// ── Emergency refund tests ──────────────────────────────────────────

#[test]
fn emergency_refund_succeeds_after_30d() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let timeout = 1_700_000_000i64;
    let now: u64 = (timeout + 2592000 + 1) as u64;
    let input_value: u64 = 200_000;

    let compiled = compile_daglock(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &[0u8; 32],
        timeout,
        &pubkey_bytes(&treasury),
    );

    let outputs = vec![
        TransactionOutput::new(input_value, p2pk_script(&pubkey_bytes(&buyer))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([12u8; 32]), 0),
        vec![], 0, 0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs.clone(), now, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let sigscript = compiled
        .build_sig_script(entrypoints::EMERGENCY_REFUND_NOSIG, vec![])
        .expect("build_sig_script");

    let reused = SigHashReusedValuesUnsync::new();
    let mut mtx = mtx;
    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };

    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_ok(), "emergency_refund after 30d should succeed: {:?}", result.err());
}

// ── Deposit release test ────────────────────────────────────────────

#[test]
fn deposit_release_compiles_and_executes() {
    let party1 = random_keypair();
    let party2 = random_keypair();
    let jury = random_keypair();
    let treasury = random_keypair();
    let deposit: u64 = 1_000_000;
    let total = deposit * 2;

    let compiled = compile_daglock_deposit(
        &pubkey_bytes(&party1), &pubkey_bytes(&party2),
        &pubkey_bytes(&jury), deposit as i64, 1_800_000_000,
        &pubkey_bytes(&treasury),
    );

    let outputs = vec![
        TransactionOutput::new(deposit, p2pk_script(&pubkey_bytes(&party1))),
        TransactionOutput::new(deposit, p2pk_script(&pubkey_bytes(&party2))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([13u8; 32]), 0),
        vec![], 0, 0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs.clone(), 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(total, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let sigscript = compiled
        .build_sig_script(entrypoints::RELEASE, vec![
            daglock_contracts::silverscript_lang::ast::Expr::bytes(vec![0u8; 65]),
            daglock_contracts::silverscript_lang::ast::Expr::bytes(vec![0u8; 65]),
        ])
        .expect("build_sig_script");

    let reused = SigHashReusedValuesUnsync::new();
    let mut mtx = mtx;
    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };

    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    // Deposit release requires valid signatures — we're using dummy sigs so it fails
    // But the important thing is the covenant compiles and runs without crashing
    assert!(result.is_err(), "deposit release with dummy sigs should fail (expected)");
}

// ── Multi-party structural test ─────────────────────────────────────

#[test]
fn multi_escrow_compiles_and_has_correct_entrypoints() {
    let party1 = random_keypair();
    let party2 = random_keypair();
    let party3 = random_keypair();
    let party4 = random_keypair();
    let treasury = random_keypair();
    let shares = vec![5_000i64, 3_000, 2_000, 0];

    let compiled = compile_daglock_multi(
        &pubkey_bytes(&party1), &pubkey_bytes(&party2),
        &pubkey_bytes(&party3), &pubkey_bytes(&party4),
        shares, &[0u8; 32], 1_700_000_000,
        &pubkey_bytes(&treasury),
    );

    // Verify 3 entrypoints: release, swap, refund
    assert_eq!(compiled.abi.len(), 3);
    let names: Vec<&str> = compiled.abi.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"release"));
    assert!(names.contains(&"swap"));
    assert!(names.contains(&"refund"));
    assert!(!compiled.script.is_empty());
}

// ── Advanced Covenant Tests ──────────────────────────────────────────

fn test_advanced_release(
    buyer_sig_valid: bool,
    seller_sig_valid: bool,
) -> Result<(), kaspa_txscript_errors::TxScriptError> {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let zero_hash = [0u8; 32];
    let timeout = 2_000_000_000i64;

    let compiled = compile_daglock_advanced(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &zero_hash,
        timeout,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 200_000;
    let fee_amount = input_value / 200;
    let send_amount = input_value - fee_amount;

    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&seller))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([15u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs.clone(), 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

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
                Expr::bytes(buyer_sig),
                Expr::bytes(seller_sig),
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
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    vm.execute()
}

#[test]
fn advanced_release_path_succeeds_with_both_signatures() {
    let result = test_advanced_release(true, true);
    assert!(result.is_ok(), "advanced release with both sigs: {:?}", result.err());
}

#[test]
fn advanced_release_path_fails_with_single_signature() {
    let result = test_advanced_release(true, false);
    assert!(result.is_err(), "advanced release with single sig should fail");
}

#[test]
fn advanced_swap_with_correct_secret_succeeds() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let secret = b"my-atomic-swap-secret-123!";
    let trade_hash = sha256_hash(secret);
    let timeout = 2_000_000_000i64;

    let compiled = compile_daglock_advanced(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &trade_hash,
        timeout,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 500_000;
    let fee_amount = input_value / 200;
    let send_amount = input_value - fee_amount;

    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&seller))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([16u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let sigscript = compiled
        .build_sig_script(entrypoints::SWAP, vec![Expr::bytes(secret.to_vec())])
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let reused = SigHashReusedValuesUnsync::new();
    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags {
        covenants_enabled: true,
        sigop_script_units: 0.into(),
    };

    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_ok(), "advanced swap with correct secret: {:?}", result.err());
}

#[test]
fn advanced_swap_with_wrong_secret_fails() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let secret = b"correct-secret";
    let trade_hash = sha256_hash(secret);
    let wrong_secret = b"wrong-secret";
    let timeout = 2_000_000_000i64;

    let compiled = compile_daglock_advanced(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &trade_hash,
        timeout,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 500_000;
    let fee_amount = input_value / 200;
    let send_amount = input_value - fee_amount;

    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&seller))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([17u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let sigscript = compiled
        .build_sig_script(entrypoints::SWAP, vec![Expr::bytes(wrong_secret.to_vec())])
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let reused = SigHashReusedValuesUnsync::new();
    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags {
        covenants_enabled: true,
        sigop_script_units: 0.into(),
    };

    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_err(), "advanced swap with wrong secret should fail");
}

#[test]
fn advanced_swap_partial_succeeds() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let secret = b"my-partial-swap-secret";
    let trade_hash = sha256_hash(secret);
    let timeout = 2_000_000_000i64;

    let compiled = compile_daglock_advanced(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &trade_hash,
        timeout,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 1_000_000;
    let amount_to_seller: u64 = 300_000;
    let fee_amount = amount_to_seller / 200;
    let net_to_seller = amount_to_seller - fee_amount;
    let return_to_buyer = input_value - amount_to_seller + fee_amount;

    let outputs = vec![
        TransactionOutput::new(net_to_seller, p2pk_script(&pubkey_bytes(&seller))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
        TransactionOutput::new(return_to_buyer, p2pk_script(&pubkey_bytes(&buyer))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([18u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let sigscript = compiled
        .build_sig_script(
            entrypoints::SWAP_PARTIAL,
            vec![
                Expr::bytes(secret.to_vec()),
                Expr::int(amount_to_seller as i64),
            ],
        )
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let reused = SigHashReusedValuesUnsync::new();
    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags {
        covenants_enabled: true,
        sigop_script_units: 0.into(),
    };

    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_ok(), "advanced swap_partial: {:?}", result.err());
}

#[test]
fn advanced_split_equal_succeeds() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let zero_hash = [0u8; 32];
    let timeout = 2_000_000_000i64;

    let compiled = compile_daglock_advanced(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &zero_hash,
        timeout,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 2_000_000;
    let fee_amount = input_value / 200;
    let distributable = input_value - fee_amount;
    let buyer_share = distributable / 2;  // 50/50 split
    let seller_share = distributable - buyer_share;

    let outputs = vec![
        TransactionOutput::new(buyer_share, p2pk_script(&pubkey_bytes(&buyer))),
        TransactionOutput::new(seller_share, p2pk_script(&pubkey_bytes(&seller))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([19u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
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

    let sigscript = compiled
        .build_sig_script(
            entrypoints::SPLIT,
            vec![
                Expr::bytes(buyer_sig),
                Expr::bytes(seller_sig),
                Expr::int(5000),  // 50/50 in basis points
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
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_ok(), "advanced split 50/50: {:?}", result.err());
}

#[test]
fn advanced_auto_settle_after_timeout_succeeds() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let zero_hash = [0u8; 32];
    let timeout: i64 = 100_000;  // Past timeout

    let compiled = compile_daglock_advanced(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &zero_hash,
        timeout,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 500_000;
    let fee_amount = input_value / 200;
    let send_amount = input_value - fee_amount;

    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&seller))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([20u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, timeout as u64 + 1, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let sigscript = compiled
        .build_sig_script(entrypoints::AUTO_SETTLE, vec![])
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let reused = SigHashReusedValuesUnsync::new();
    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags {
        covenants_enabled: true,
        sigop_script_units: 0.into(),
    };

    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_ok(), "advanced auto_settle: {:?}", result.err());
}

#[test]
fn advanced_emergency_refund_after_30d_succeeds() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let zero_hash = [0u8; 32];
    // timeout + 30 days must be in the past for emergency_refund
    let timeout: i64 = 100_000;
    // tx.time needs to be >= timeout + 2592000, but we also need to make sure
    // auto_settle doesn't trigger instead. Since we're calling emergency_refund
    // directly, the covenant checks tx.time >= timeout + 2592000.
    // For the test engine, tx.time is whatever is set on the transaction.
    // The default tx time is 0, so we need to set it appropriately.

    let compiled = compile_daglock_advanced(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &zero_hash,
        timeout,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 500_000;

    let outputs = vec![
        TransactionOutput::new(input_value, p2pk_script(&pubkey_bytes(&buyer))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([21u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    // Set tx lock_time to satisfy timeout + 2592000
    let lock_time = (timeout as u64) + 2592000 + 1;
    let tx = Transaction::new(1, vec![input], outputs, lock_time, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let sigscript = compiled
        .build_sig_script(entrypoints::EMERGENCY_REFUND_NOSIG, vec![])
        .expect("build_sig_script");

    mtx.tx.inputs[0].signature_script = sigscript;

    let reused = SigHashReusedValuesUnsync::new();
    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags {
        covenants_enabled: true,
        sigop_script_units: 0.into(),
    };

    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_ok(), "advanced emergency_refund: {:?}", result.err());
}

#[test]
fn advanced_abi_has_correct_entrypoints() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let zero_hash = [0u8; 32];

    let compiled = compile_daglock_advanced(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &zero_hash,
        1_700_000_000,
        &pubkey_bytes(&treasury),
    );

    let names: Vec<&str> = compiled.abi.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"release"));
    assert!(names.contains(&"swap"));
    assert!(names.contains(&"swap_partial"));
    assert!(names.contains(&"extendTimeout"));
    assert!(names.contains(&"refund"));
    assert!(names.contains(&"auto_settle"));
    assert!(names.contains(&"split"));
    assert!(names.contains(&"emergency_refund"));
    assert_eq!(compiled.abi.len(), 8, "should have exactly 8 entrypoints");
    assert!(!compiled.script.is_empty());
}
