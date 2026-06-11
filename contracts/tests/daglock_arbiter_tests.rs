//! Execution tests for DagLock Arbiter covenant — tests all 5 entrypoints
//! through the Kaspa script engine (TxScriptEngine) with real Schnorr signatures.
//!
//! These tests do NOT require a running node.

use daglock_contracts::{compile_daglock_arbiter, entrypoints};
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

fn sha256_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

fn sign_message(kp: &Keypair, msg: &[u8]) -> Vec<u8> {
    let msg = secp256k1::Message::from_digest_slice(msg).unwrap();
    let sig = kp.sign_schnorr(msg);
    let mut s = Vec::with_capacity(65);
    s.extend_from_slice(sig.as_ref().as_slice());
    s.push(SIG_HASH_ALL.to_u8());
    s
}

/// Build a full transaction and run it through TxScriptEngine for an arbiter dispute path.
fn test_dispute_path(
    arbiter_sig_valid: bool,
    party_sig_buyer: bool,
    party_sig_valid: bool,
    mediator_fee: u64,
    _wrong_output_fee: bool,
) -> Result<(), kaspa_txscript_errors::TxScriptError> {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let arbiter = random_keypair();
    let zero_hash = [0u8; 32];
    let timeout = 2_000_000_000i64;

    let compiled = compile_daglock_arbiter(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &zero_hash,
        timeout,
        &pubkey_bytes(&treasury),
        &pubkey_bytes(&arbiter),
    );

    let input_value: u64 = 200_000;
    let treasury_fee = input_value / 200;
    let mediator_fee_value = mediator_fee;
    let send_amount = input_value - treasury_fee - mediator_fee_value;

    let recipient = if party_sig_buyer { buyer } else { seller };
    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&recipient))),
        TransactionOutput::new(treasury_fee, p2pk_script(&pubkey_bytes(&treasury))),
        TransactionOutput::new(mediator_fee_value, p2pk_script(&pubkey_bytes(&arbiter))),
    ];

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

    let arbiter_sig = if arbiter_sig_valid {
        sign_message(&arbiter, sighash.as_bytes().as_slice())
    } else {
        vec![0u8; 65]
    };

    let party_sig = if party_sig_valid {
        let party = if party_sig_buyer { &buyer } else { &seller };
        sign_message(party, sighash.as_bytes().as_slice())
    } else {
        vec![0u8; 65]
    };

    let entrypoint = if party_sig_buyer {
        entrypoints::DISPUTE_BUYER_WINS
    } else {
        entrypoints::DISPUTE_SELLER_WINS
    };

    let sigscript = compiled
        .build_sig_script(
            entrypoint,
            vec![
                daglock_contracts::silverscript_lang::ast::Expr::bytes(arbiter_sig),
                daglock_contracts::silverscript_lang::ast::Expr::bytes(party_sig),
                daglock_contracts::silverscript_lang::ast::Expr::int(mediator_fee_value as i64),
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

/// Test that disputeSellerWins succeeds with valid arbiter + seller sigs and correct mediator fee.
#[test]
fn dispute_seller_wins_succeeds() {
    let result = test_dispute_path(true, false, true, 10_000, false);
    assert!(
        result.is_ok(),
        "disputeSellerWins with valid sigs failed: {}",
        result.unwrap_err()
    );
}

/// Test that disputeBuyerWins succeeds with valid arbiter + buyer sigs and correct mediator fee.
#[test]
fn dispute_buyer_wins_succeeds() {
    let result = test_dispute_path(true, true, true, 10_000, false);
    assert!(
        result.is_ok(),
        "disputeBuyerWins with valid sigs failed: {}",
        result.unwrap_err()
    );
}

/// Test that disputeSellerWins succeeds with zero mediator fee (no fee to mediator).
#[test]
fn dispute_seller_wins_zero_mediator_fee() {
    let result = test_dispute_path(true, false, true, 0, false);
    assert!(
        result.is_ok(),
        "disputeSellerWins with zero mediator fee failed: {}",
        result.unwrap_err()
    );
}

/// Test that disputeBuyerWins fails when the arbiter signature is invalid.
#[test]
fn dispute_buyer_wins_fails_wrong_arbiter_sig() {
    let result = test_dispute_path(false, true, true, 10_000, false);
    assert!(
        result.is_err(),
        "disputeBuyerWins with invalid arbiter sig should fail"
    );
}

/// Test that disputeSellerWins fails when the seller signature is invalid.
#[test]
fn dispute_seller_wins_fails_wrong_seller_sig() {
    let result = test_dispute_path(true, false, false, 10_000, false);
    assert!(
        result.is_err(),
        "disputeSellerWins with invalid seller sig should fail"
    );
}

/// Test that disputeBuyerWins fails without buyer sig (arbiter alone — can't steal).
#[test]
fn dispute_arbiter_alone_cannot_steal() {
    // Arbiter sig valid but no party sig
    let result = test_dispute_path(true, false, false, 10_000, false);
    assert!(
        result.is_err(),
        "arbiter alone without party sig should fail"
    );
}

/// Test that dispute paths fail when sendAmount <= 0 (mediator fee too large).
#[test]
fn dispute_mediator_fee_too_large_fails() {
    // mediator_fee = input_value - treasury_fee = 199_000 → sendAmount = 0
    // Covenant: require(sendAmount > 0) → fails
    let result = test_dispute_path(true, true, true, 199_000, false);
    assert!(
        result.is_err(),
        "dispute with mediator fee making sendAmount=0 should fail"
    );
}

// ── Standard path tests (should work identically to daglock.sil) ───

/// Helper: build and execute the release path on the arbiter covenant.
fn test_arbiter_release(
    buyer_sig_valid: bool,
    seller_sig_valid: bool,
    wrong_fee: bool,
) -> Result<(), kaspa_txscript_errors::TxScriptError> {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let arbiter = random_keypair();
    let zero_hash = [0u8; 32];
    let timeout = 2_000_000_000i64;

    let compiled = compile_daglock_arbiter(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &zero_hash,
        timeout,
        &pubkey_bytes(&treasury),
        &pubkey_bytes(&arbiter),
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

#[test]
fn arbiter_release_succeeds_with_both_signatures() {
    let result = test_arbiter_release(true, true, false);
    assert!(
        result.is_ok(),
        "arbiter release with both sigs failed: {}",
        result.unwrap_err()
    );
}

#[test]
fn arbiter_release_fails_with_only_buyer() {
    let result = test_arbiter_release(true, false, false);
    assert!(
        result.is_err(),
        "arbiter release with only buyer sig should fail"
    );
}

#[test]
fn arbiter_release_fails_with_wrong_fee() {
    let result = test_arbiter_release(true, true, true);
    assert!(
        result.is_err(),
        "arbiter release with wrong fee should fail"
    );
}

#[test]
fn arbiter_swap_succeeds_with_correct_preimage() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let arbiter = random_keypair();
    let secret = b"arbiter-atomic-swap-secret";
    let trade_hash = sha256_hash(secret);

    let compiled = compile_daglock_arbiter(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &trade_hash,
        2_000_000_000,
        &pubkey_bytes(&treasury),
        &pubkey_bytes(&arbiter),
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
                secret.to_vec(),
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
    assert!(
        result.is_ok(),
        "arbiter swap with correct preimage failed: {}",
        result.unwrap_err()
    );
}

#[test]
fn arbiter_swap_fails_with_wrong_preimage() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let arbiter = random_keypair();
    let secret = b"correct-secret";
    let trade_hash = sha256_hash(secret);
    let wrong_secret = b"wrong-secret-for-arbiter";

    let compiled = compile_daglock_arbiter(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &trade_hash,
        2_000_000_000,
        &pubkey_bytes(&treasury),
        &pubkey_bytes(&arbiter),
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
    assert!(
        result.is_err(),
        "arbiter swap with wrong preimage should fail"
    );
}

#[test]
fn arbiter_refund_succeeds_after_timeout() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let arbiter = random_keypair();
    let timeout: i64 = 1_600_000_000;
    let zero_hash = [0u8; 32];

    let compiled = compile_daglock_arbiter(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &zero_hash,
        timeout,
        &pubkey_bytes(&treasury),
        &pubkey_bytes(&arbiter),
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

    let arbiter_sig_raw = arbiter.sign_schnorr(msg);
    let mut arbiter_sig = Vec::with_capacity(65);
    arbiter_sig.extend_from_slice(arbiter_sig_raw.as_ref().as_slice());
    arbiter_sig.push(SIG_HASH_ALL.to_u8());

    let buyer_sig_raw = buyer.sign_schnorr(msg);
    let mut buyer_sig = Vec::with_capacity(65);
    buyer_sig.extend_from_slice(buyer_sig_raw.as_ref().as_slice());
    buyer_sig.push(SIG_HASH_ALL.to_u8());

    let sigscript = compiled
        .build_sig_script(
            entrypoints::REFUND_AFTER_TIMEOUT,
            vec![
                daglock_contracts::silverscript_lang::ast::Expr::bytes(arbiter_sig),
                daglock_contracts::silverscript_lang::ast::Expr::bytes(buyer_sig),
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
    let result = vm.execute();
    assert!(
        result.is_ok(),
        "arbiter refund after timeout failed: {}",
        result.unwrap_err()
    );
}

#[test]
fn arbiter_refund_fails_before_timeout() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let arbiter = random_keypair();
    let timeout: i64 = 3_000_000_000;
    let zero_hash = [0u8; 32];

    let compiled = compile_daglock_arbiter(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &zero_hash,
        timeout,
        &pubkey_bytes(&treasury),
        &pubkey_bytes(&arbiter),
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

    let arbiter_sig_raw = arbiter.sign_schnorr(msg);
    let mut arbiter_sig = Vec::with_capacity(65);
    arbiter_sig.extend_from_slice(arbiter_sig_raw.as_ref().as_slice());
    arbiter_sig.push(SIG_HASH_ALL.to_u8());

    let buyer_sig_raw = buyer.sign_schnorr(msg);
    let mut buyer_sig = Vec::with_capacity(65);
    buyer_sig.extend_from_slice(buyer_sig_raw.as_ref().as_slice());
    buyer_sig.push(SIG_HASH_ALL.to_u8());

    let sigscript = compiled
        .build_sig_script(
            entrypoints::REFUND_AFTER_TIMEOUT,
            vec![
                daglock_contracts::silverscript_lang::ast::Expr::bytes(arbiter_sig),
                daglock_contracts::silverscript_lang::ast::Expr::bytes(buyer_sig),
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
    let result = vm.execute();
    assert!(result.is_err(), "arbiter refund before timeout should fail");
}
#[test]
fn arbiter_refund_fails_without_arbiter_sig() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let arbiter = random_keypair();
    let timeout: i64 = 1_600_000_000;
    let zero_hash = [0u8; 32];

    let compiled = compile_daglock_arbiter(
        &pubkey_bytes(&buyer),
        &pubkey_bytes(&seller),
        &zero_hash,
        timeout,
        &pubkey_bytes(&treasury),
        &pubkey_bytes(&arbiter),
    );

    let input_value: u64 = 200_000;
    let outputs = vec![TransactionOutput::new(
        input_value,
        p2pk_script(&pubkey_bytes(&buyer)),
    )];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([7u8; 32]), 0),
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

    let buyer_sig_raw = buyer.sign_schnorr(msg);
    let mut buyer_sig = Vec::with_capacity(65);
    buyer_sig.extend_from_slice(buyer_sig_raw.as_ref().as_slice());
    buyer_sig.push(SIG_HASH_ALL.to_u8());

    let sigscript = compiled
        .build_sig_script(
            entrypoints::REFUND_AFTER_TIMEOUT,
            vec![
                daglock_contracts::silverscript_lang::ast::Expr::bytes(vec![0u8; 65]),
                daglock_contracts::silverscript_lang::ast::Expr::bytes(buyer_sig),
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
    let result = vm.execute();
    assert!(
        result.is_err(),
        "refund without arbiter sig should fail even after timeout"
    );
}

#[test]
fn emergency_refund_succeeds_after_grace_period() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let arbiter = random_keypair();
    let treasury = random_keypair();
    let timeout: i64 = 1_600_000_000; // past timestamp
    let grace_expiry = timeout + 2_592_000; // timeout + 30 days, also past

    let compiled = compile_daglock_arbiter(
        &pubkey_bytes(&buyer), &pubkey_bytes(&seller), &[0u8; 32],
        timeout, &pubkey_bytes(&treasury), &pubkey_bytes(&arbiter),
    );

    let input_value: u64 = 1_000_000;
    let outputs = vec![TransactionOutput::new(input_value, p2pk_script(&pubkey_bytes(&buyer)))];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([20u8; 32]), 0), vec![], 0, 0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, grace_expiry as u64, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let sig_raw = buyer.sign_schnorr(msg);
    let mut sig = Vec::with_capacity(65);
    sig.extend_from_slice(sig_raw.as_ref().as_slice());
    sig.push(SIG_HASH_ALL.to_u8());

    let sigscript = compiled.build_sig_script(
        entrypoints::EMERGENCY_REFUND,
        vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(sig)],
    ).expect("build_sig_script");
    mtx.tx.inputs[0].signature_script = sigscript;

    let reused = SigHashReusedValuesUnsync::new();
    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_ok(), "emergency refund after grace should succeed: {:?}", result.err());
}

#[test]
fn emergency_refund_fails_during_grace_period() {
    let buyer = random_keypair();
    let seller = random_keypair();
    let arbiter = random_keypair();
    let treasury = random_keypair();
    let timeout: i64 = 3_000_000_000; // far future
    let still_in_grace: i64 = timeout + 1; // 1 second after timeout, still well within 30-day grace

    let compiled = compile_daglock_arbiter(
        &pubkey_bytes(&buyer), &pubkey_bytes(&seller), &[0u8; 32],
        timeout, &pubkey_bytes(&treasury), &pubkey_bytes(&arbiter),
    );

    let input_value: u64 = 1_000_000;
    let outputs = vec![TransactionOutput::new(input_value, p2pk_script(&pubkey_bytes(&buyer)))];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([21u8; 32]), 0), vec![], 0, 0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, still_in_grace as u64, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let sig_raw = buyer.sign_schnorr(msg);
    let mut sig = Vec::with_capacity(65);
    sig.extend_from_slice(sig_raw.as_ref().as_slice());
    sig.push(SIG_HASH_ALL.to_u8());

    let sigscript = compiled.build_sig_script(
        entrypoints::EMERGENCY_REFUND,
        vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(sig)],
    ).expect("build_sig_script");
    mtx.tx.inputs[0].signature_script = sigscript;

    let reused = SigHashReusedValuesUnsync::new();
    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_err(), "emergency refund during 30-day grace should fail");
}
