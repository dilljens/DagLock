//! Execution tests for DagLock Vault covenant — runs the withdraw path through
//! the Kaspa script engine (TxScriptEngine) with real Schnorr signatures.

use daglock_contracts::{compile_daglock_vault, compile_daglock_vault_multisig, entrypoints};
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

#[test]
fn vault_withdraw_succeeds_after_timeout() {
    let owner = random_keypair();
    let lock_duration: i64 = 500;
    let treasury = random_keypair();

    let compiled = compile_daglock_vault(
        &pubkey_bytes(&owner),
        lock_duration,
        &pubkey_bytes(&treasury),
        &[0u8; 32],
        0,
    );

    let input_value: u64 = 2_000_000;
    let fee_amount = input_value / 1000;
    let send_amount = input_value - fee_amount;
    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&owner))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([1u8; 32]), 0),
        vec![],
        lock_duration as u64,
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
    let sig_raw = owner.sign_schnorr(msg);
    let mut sig = Vec::with_capacity(65);
    sig.extend_from_slice(sig_raw.as_ref().as_slice());
    sig.push(SIG_HASH_ALL.to_u8());

    let sigscript = compiled
        .build_sig_script(
            entrypoints::WITHDRAW,
            vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(sig)],
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
        "withdraw after timeout failed: {}",
        result.unwrap_err()
    );
}

#[test]
fn vault_withdraw_fails_before_timeout() {
    let owner = random_keypair();
    let lock_duration: i64 = 3000;
    let treasury = random_keypair();

    let compiled = compile_daglock_vault(
        &pubkey_bytes(&owner),
        lock_duration,
        &pubkey_bytes(&treasury),
        &[0u8; 32],
        0,
    );

    let input_value: u64 = 2_000_000;
    let fee_amount = input_value / 1000;
    let send_amount = input_value - fee_amount;
    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&owner))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([2u8; 32]), 0),
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
    let sig_raw = owner.sign_schnorr(msg);
    let mut sig = Vec::with_capacity(65);
    sig.extend_from_slice(sig_raw.as_ref().as_slice());
    sig.push(SIG_HASH_ALL.to_u8());

    let sigscript = compiled
        .build_sig_script(
            entrypoints::WITHDRAW,
            vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(sig)],
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
    assert!(result.is_err(), "withdraw before timeout should fail");
}

#[test]
fn vault_withdraw_fails_wrong_signature() {
    let owner = random_keypair();
    let wrong_signer = random_keypair();
    let lock_duration: i64 = 500;
    let treasury = random_keypair();

    let compiled = compile_daglock_vault(
        &pubkey_bytes(&owner),
        lock_duration,
        &pubkey_bytes(&treasury),
        &[0u8; 32],
        0,
    );

    let input_value: u64 = 2_000_000;
    let fee_amount = input_value / 1000;
    let send_amount = input_value - fee_amount;
    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&owner))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([3u8; 32]), 0),
        vec![],
        lock_duration as u64,
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
    let sig_raw = wrong_signer.sign_schnorr(msg);
    let mut sig = Vec::with_capacity(65);
    sig.extend_from_slice(sig_raw.as_ref().as_slice());
    sig.push(SIG_HASH_ALL.to_u8());

    let sigscript = compiled
        .build_sig_script(
            entrypoints::WITHDRAW,
            vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(sig)],
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
    assert!(result.is_err(), "withdraw with wrong sig should fail");
}

/* ─── Softlock Vault Tests ───────────────────────────────────── */

use daglock_contracts::compile_daglock_vault_softlock;
use sha2::{Digest, Sha256};

fn sha256_full(password: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password);
    let result = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&result);
    arr
}

fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&result);
    arr
}

#[test]
fn softlock_password_withdraw_succeeds_correct_password() {
    let owner = random_keypair();
    let beneficiary = random_keypair();
    let password = b"my-secure-password-123!";
    let password_hash = sha256_full(password);
    let lock_duration: i64 = 500;
    let treasury = random_keypair();

    let compiled = compile_daglock_vault_softlock(
        &pubkey_bytes(&owner),
        &pubkey_bytes(&beneficiary),
        &password_hash,
        lock_duration,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 2_000_000;
    let fee_amount = input_value / 1000;
    let send_amount = input_value - fee_amount;
    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&beneficiary))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

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
        0, // no timelock needed — password path doesn't check time
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
            entrypoints::WITHDRAW_PASSWORD,
            vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(
                password.to_vec(),
            )],
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
    let mut vm =
        TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(
        result.is_ok(),
        "password withdraw with correct password failed: {}",
        result.unwrap_err()
    );
}

#[test]
fn softlock_password_withdraw_fails_wrong_password() {
    let owner = random_keypair();
    let beneficiary = random_keypair();
    let password = b"correct-password";
    let password_hash = sha256_full(password);
    let wrong_password = b"wrong-password";
    let lock_duration: i64 = 500;
    let treasury = random_keypair();

    let compiled = compile_daglock_vault_softlock(
        &pubkey_bytes(&owner),
        &pubkey_bytes(&beneficiary),
        &password_hash,
        lock_duration,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 2_000_000;
    let fee_amount = input_value / 1000;
    let send_amount = input_value - fee_amount;
    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&beneficiary))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([7u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, 0, Default::default(), 0, vec![]);
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
            entrypoints::WITHDRAW_PASSWORD,
            vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(
                wrong_password.to_vec(),
            )],
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
    let mut vm =
        TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(
        result.is_err(),
        "password withdraw with wrong password should fail"
    );
}

#[test]
fn softlock_timeout_withdraw_succeeds_after_timeout() {
    let owner = random_keypair();
    let beneficiary = random_keypair();
    let password_hash = sha256_full(b"any-password");
    let lock_duration: i64 = 500;
    let treasury = random_keypair();

    let compiled = compile_daglock_vault_softlock(
        &pubkey_bytes(&owner),
        &pubkey_bytes(&owner),
        &password_hash,
        lock_duration,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 2_000_000;
    let fee_amount = input_value / 1000;
    let send_amount = input_value - fee_amount;
    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&owner))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([8u8; 32]), 0),
        vec![],
        lock_duration as u64,
        0u8,
    );
    let tx = Transaction::new(
        1,
        vec![input],
        outputs,
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
    let sig_raw = owner.sign_schnorr(msg);
    let mut sig = Vec::with_capacity(65);
    sig.extend_from_slice(sig_raw.as_ref().as_slice());
    sig.push(SIG_HASH_ALL.to_u8());

    let sigscript = compiled
        .build_sig_script(
            entrypoints::WITHDRAW_TIMEOUT,
            vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(sig)],
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
        "softlock timeout withdraw failed: {}",
        result.unwrap_err()
    );
}

#[test]
fn softlock_timeout_withdraw_fails_before_timeout() {
    let owner = random_keypair();
    let beneficiary = random_keypair();
    let password_hash = sha256_full(b"any");
    let lock_duration: i64 = 3000;
    let treasury = random_keypair();

    let compiled = compile_daglock_vault_softlock(
        &pubkey_bytes(&owner),
        &pubkey_bytes(&owner),
        &password_hash,
        lock_duration,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 2_000_000;
    let fee_amount = input_value / 1000;
    let send_amount = input_value - fee_amount;
    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&owner))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([9u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, 0, Default::default(), 0, vec![]);
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
    let sig_raw = owner.sign_schnorr(msg);
    let mut sig = Vec::with_capacity(65);
    sig.extend_from_slice(sig_raw.as_ref().as_slice());
    sig.push(SIG_HASH_ALL.to_u8());

    let sigscript = compiled
        .build_sig_script(
            entrypoints::WITHDRAW_TIMEOUT,
            vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(sig)],
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
    let mut vm =
        TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(
        result.is_err(),
        "softlock timeout withdraw before timeout should fail"
    );
}

#[test]
fn vault_sweep_succeeds_after_timeout() {
    let owner = random_keypair();
    let lock_duration: i64 = 500;
    let treasury = random_keypair();

    let compiled = compile_daglock_vault(
        &pubkey_bytes(&owner),
        lock_duration,
        &pubkey_bytes(&treasury),
        &[0u8; 32],
        0,
    );

    let input_value: u64 = 2_000_000;
    let fee_amount = input_value / 1000;
    let send_amount = input_value - fee_amount;
    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&owner))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([10u8; 32]), 0),
        vec![],
        lock_duration as u64,
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
        .build_sig_script(entrypoints::SWEEP, vec![])
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
    let mut vm =
        TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(
        result.is_ok(),
        "sweep after timeout should succeed: {:?}",
        result.err()
    );
}

#[test]
fn vault_sweep_fails_before_timeout() {
    let owner = random_keypair();
    let lock_duration: i64 = 3000;
    let treasury = random_keypair();
    let compiled = compile_daglock_vault(
        &pubkey_bytes(&owner),
        lock_duration,
        &pubkey_bytes(&treasury),
        &[0u8; 32],
        0,
    );

    let input_value: u64 = 2_000_000;
    let fee_amount = input_value / 1000;
    let send_amount = input_value - fee_amount;
    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&owner))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([11u8; 32]), 0),
        vec![],
        0,
        0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(
        input_value,
        ScriptPublicKey::new(0, compiled.script.clone().into()),
        0,
        false,
        None,
    );
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let sigscript = compiled
        .build_sig_script(entrypoints::SWEEP, vec![])
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
    let mut vm =
        TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_err(), "sweep before timeout should fail");
}

// ── Multi-sig Sweep Tests ────────────────────────────────────────

#[test]
fn multisig_sweep_succeeds_with_single_key() {
    let kp1 = random_keypair();
    let kp2 = random_keypair();
    let kp3 = random_keypair();
    let treasury = random_keypair();
    let lock_duration: i64 = 500;

    let compiled = compile_daglock_vault_multisig(
        &pubkey_bytes(&kp1),
        &pubkey_bytes(&kp2),
        &pubkey_bytes(&kp3),
        lock_duration,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 2_000_000;
    let fee_amount = input_value / 1000;
    let send_amount = input_value - fee_amount;
    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&kp1))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([30u8; 32]), 0),
        vec![],
        lock_duration as u64,
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

    // Sign with key1 only (key1 can sweep alone after timeout)
    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let sig_raw = kp1.sign_schnorr(msg);
    let mut sig = Vec::with_capacity(65);
    sig.extend_from_slice(sig_raw.as_ref().as_slice());
    sig.push(SIG_HASH_ALL.to_u8());

    use daglock_contracts::silverscript_lang::ast::Expr;
    let sigscript = compiled
        .build_sig_script(entrypoints::SWEEP, vec![Expr::bytes(sig)])
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
    let mut vm =
        TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(
        result.is_ok(),
        "multisig sweep with single key should succeed: {:?}",
        result.err()
    );
}

#[test]
fn multisig_sweep_fails_with_wrong_key() {
    let kp1 = random_keypair();
    let kp2 = random_keypair();
    let treasury = random_keypair();
    let impostor = random_keypair(); // not configured as a signer
    let lock_duration: i64 = 500;

    let compiled = compile_daglock_vault_multisig(
        &pubkey_bytes(&kp1),
        &pubkey_bytes(&kp2),
        &[0u8; 32],
        lock_duration,
        &pubkey_bytes(&treasury),
    );

    let input_value: u64 = 2_000_000;
    let fee_amount = input_value / 1000;
    let send_amount = input_value - fee_amount;
    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&kp1))),
        TransactionOutput::new(fee_amount, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([31u8; 32]), 0),
        vec![],
        lock_duration as u64,
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
    let sig_raw = impostor.sign_schnorr(msg);
    let mut sig = Vec::with_capacity(65);
    sig.extend_from_slice(sig_raw.as_ref().as_slice());
    sig.push(SIG_HASH_ALL.to_u8());

    use daglock_contracts::silverscript_lang::ast::Expr;
    let sigscript = compiled
        .build_sig_script(entrypoints::SWEEP, vec![Expr::bytes(sig)])
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
    let mut vm =
        TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    let result = vm.execute();
    assert!(result.is_err(), "multisig sweep with wrong key should fail");
}
