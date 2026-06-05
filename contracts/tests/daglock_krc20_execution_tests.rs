//! Execution tests for DagLock KRC-20 covenant — tests release, swap, and refund paths
//! through the Kaspa script engine (TxScriptEngine) with real Schnorr signatures.

use daglock_contracts::{compile_daglock_krc20, entrypoints};
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

fn compile_krc20(
    buyer: &Keypair, seller: &Keypair, trade_hash: &[u8],
    timeout: i64, treasury: &Keypair,
) -> daglock_contracts::silverscript_lang::compiler::CompiledContract<'static> {
    compile_daglock_krc20(
        &pubkey_bytes(buyer), &pubkey_bytes(seller), trade_hash, timeout,
        &pubkey_bytes(treasury),
        0, 0, &[0u8; 32], &[], &[], &[0u8; 32],
    )
}

fn test_release(
    buyer_sig_valid: bool, seller_sig_valid: bool, fee_paid: bool,
) -> Result<(), kaspa_txscript_errors::TxScriptError> {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let compiled = compile_krc20(&buyer, &seller, &[0u8; 32], 2_000_000_000, &treasury);

    let input_value: u64 = 100_000;
    let mut outputs = vec![TransactionOutput::new(input_value, p2pk_script(&pubkey_bytes(&random_keypair())))];
    if fee_paid {
        outputs.push(TransactionOutput::new(500, p2pk_script(&pubkey_bytes(&treasury))));
    }

    let tx = Transaction::new(1, vec![TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([1u8; 32]), 0), vec![], 0, 0u8,
    )], outputs, 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();

    let bs = if buyer_sig_valid { let s = buyer.sign_schnorr(msg); let mut v = Vec::from(s.as_ref()); v.push(SIG_HASH_ALL.to_u8()); v } else { vec![0u8; 65] };
    let ss = if seller_sig_valid { let s = seller.sign_schnorr(msg); let mut v = Vec::from(s.as_ref()); v.push(SIG_HASH_ALL.to_u8()); v } else { vec![0u8; 65] };

    mtx.tx.inputs[0].signature_script = compiled.build_sig_script(
        entrypoints::RELEASE,
        vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(bs),
             daglock_contracts::silverscript_lang::ast::Expr::bytes(ss)],
    ).expect("build_sig_script");

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    vm.execute()
}

fn test_swap(preimage_correct: bool, fee_paid: bool) -> Result<(), kaspa_txscript_errors::TxScriptError> {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let secret = b"krc20-swap-secret-v1";
    let trade_hash = sha256_hash(secret);
    let compiled = compile_krc20(&buyer, &seller, &trade_hash, 2_000_000_000, &treasury);

    let input_value: u64 = 100_000;
    let mut outputs = vec![TransactionOutput::new(input_value, p2pk_script(&pubkey_bytes(&random_keypair())))];
    if fee_paid {
        outputs.push(TransactionOutput::new(500, p2pk_script(&pubkey_bytes(&treasury))));
    }

    let tx = Transaction::new(1, vec![TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([2u8; 32]), 0), vec![], 0, 0u8,
    )], outputs, 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let secret_use: Vec<u8> = if preimage_correct { secret.to_vec() } else { b"wrong-secret-for-krc20-tx".to_vec() };
    mtx.tx.inputs[0].signature_script = compiled.build_sig_script(
        entrypoints::SWAP,
        vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(secret_use)],
    ).expect("build_sig_script");

    let reused = SigHashReusedValuesUnsync::new();
    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    vm.execute()
}

fn test_refund(after_timeout: bool) -> Result<(), kaspa_txscript_errors::TxScriptError> {
    let buyer = random_keypair();
    let seller = random_keypair();
    let treasury = random_keypair();
    let timeout: i64 = 1_600_000_000;
    let compiled = compile_krc20(&buyer, &seller, &[0u8; 32], timeout, &treasury);

    let input_value: u64 = 100_000;
    let tx = Transaction::new(1, vec![TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([3u8; 32]), 0), vec![], 0, 0u8,
    )], vec![TransactionOutput::new(input_value, p2pk_script(&pubkey_bytes(&buyer)))],
        if after_timeout { timeout as u64 } else { 0 }, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(input_value, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let sig_raw = buyer.sign_schnorr(msg);
    let mut bs = Vec::from(sig_raw.as_ref());
    bs.push(SIG_HASH_ALL.to_u8());

    mtx.tx.inputs[0].signature_script = compiled.build_sig_script(
        entrypoints::REFUND,
        vec![daglock_contracts::silverscript_lang::ast::Expr::bytes(bs)],
    ).expect("build_sig_script");

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo, ctx, flags);
    vm.execute()
}

#[test]
fn krc20_release_succeeds_with_both_sigs_and_fee() {
    assert!(test_release(true, true, true).is_ok());
}

#[test]
fn krc20_release_fails_without_fee() {
    assert!(test_release(true, true, false).is_err());
}

#[test]
fn krc20_release_fails_with_only_buyer_sig() {
    assert!(test_release(true, false, true).is_err());
}

#[test]
fn krc20_swap_succeeds_with_correct_preimage_and_fee() {
    assert!(test_swap(true, true).is_ok());
}

#[test]
fn krc20_swap_fails_without_fee() {
    assert!(test_swap(true, false).is_err());
}

#[test]
fn krc20_swap_fails_with_wrong_preimage() {
    assert!(test_swap(false, true).is_err());
}

#[test]
fn krc20_refund_succeeds_after_timeout() {
    assert!(test_refund(true).is_ok());
}

#[test]
fn krc20_refund_fails_before_timeout() {
    assert!(test_refund(false).is_err());
}
