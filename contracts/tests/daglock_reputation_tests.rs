//! Execution tests for DagLock Reputation covenant.

use daglock_contracts::{compile_daglock_reputation, entrypoints};
use daglock_contracts::silverscript_lang::ast::Expr;
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

fn find_entrypoint(
    compiled: &daglock_contracts::silverscript_lang::compiler::CompiledContract,
    name: &str,
) -> u16 {
    compiled.abi.iter().position(|e| e.name == name).map(|i| i as u16)
        .expect(format!("Entrypoint '{name}' not found").as_str())
}

fn compile_reputation_with_key(treasury: &Keypair) -> daglock_contracts::silverscript_lang::compiler::CompiledContract<'static> {
    compile_daglock_reputation(&pubkey_bytes(treasury))
}

#[test]
fn record_trade_with_both_signatures_succeeds() {
    let treasury = random_keypair();
    let compiled = compile_reputation_with_key(&treasury);
    let _entrypoint = find_entrypoint(&compiled, entrypoints::RECORD_TRADE);

    let buyer = random_keypair();
    let seller = random_keypair();
    let buyer_pk = pubkey_bytes(&buyer);
    let seller_pk = pubkey_bytes(&seller);

    // Build a basic transaction (will be signed after we have the sig script)
    let input_value: u64 = 100_000;
    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::default(), 0),
        vec![],
        0,
        0u8,
    );

    let outputs = vec![
        TransactionOutput::new(1000, p2pk_script(&pubkey_bytes(&treasury))),
        TransactionOutput::new(input_value - 1000, p2pk_script(&pubkey_bytes(&treasury))),
    ];

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

    // Generate both signatures (Schnorr + SIGHASH_ALL)
    let reused_sig = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(
        &mtx.as_verifiable(),
        0,
        SIG_HASH_ALL,
        &reused_sig,
    );
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();
    let sig_raw_buyer = buyer.sign_schnorr(msg);
    let mut sig_buyer = Vec::with_capacity(65);
    sig_buyer.extend_from_slice(sig_raw_buyer.as_ref().as_slice());
    sig_buyer.push(SIG_HASH_ALL.to_u8());

    let sighash2 = calc_schnorr_signature_hash(
        &mtx.as_verifiable(),
        0,
        SIG_HASH_ALL,
        &reused_sig,
    );
    let msg2 = secp256k1::Message::from_digest_slice(sighash2.as_bytes().as_slice()).unwrap();
    let sig_raw_seller = seller.sign_schnorr(msg2);
    let mut sig_seller = Vec::with_capacity(65);
    sig_seller.extend_from_slice(sig_raw_seller.as_ref().as_slice());
    sig_seller.push(SIG_HASH_ALL.to_u8());

    // Build the unlocking script with ALL entrypoint parameters + signatures
    let sigscript = compiled
        .build_sig_script(
            entrypoints::RECORD_TRADE,
            vec![
                Expr::bytes(buyer_pk),
                Expr::bytes(seller_pk),
                Expr::int(100_000_000),    // amount (1 KAS)
                Expr::int(0),               // outcome (settled)
                Expr::int(1_700_000_000),   // timestamp
                Expr::bytes(vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]), // nonce
                Expr::bytes(sig_buyer),
                Expr::bytes(sig_seller),
            ],
        )
        .expect("build_sig_script should succeed");

    mtx.tx.inputs[0].signature_script = sigscript;

    // Execute through the script engine
    let mut sig_cache = Cache::new(10_000);
    let reused = SigHashReusedValuesUnsync::new();
    let ctx = EngineCtx::new(&mut sig_cache).with_reused(&reused);
    let flags = EngineFlags {
        covenants_enabled: true,
        sigop_script_units: 0.into(),
    };

    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(
        &ver_tx,
        &ver_tx.inputs()[0],
        0,
        &utxo,
        ctx,
        flags,
    );
    let result = vm.execute();
    assert!(result.is_ok(), "recordTrade should succeed: {:?}", result);
}

#[test]
fn reputation_compiles_with_different_keys() {
    let k1 = random_keypair();
    let k2 = random_keypair();
    let c1 = compile_daglock_reputation(&pubkey_bytes(&k1));
    let c2 = compile_daglock_reputation(&pubkey_bytes(&k2));
    assert_ne!(c1.script, c2.script);
}

#[test]
fn reputation_template_hash_is_deterministic() {
    let key = random_keypair();
    let c1 = compile_daglock_reputation(&pubkey_bytes(&key));
    let c2 = compile_daglock_reputation(&pubkey_bytes(&key));
    let (_, _, h1) = daglock_contracts::template_parts_and_hash(&c1);
    let (_, _, h2) = daglock_contracts::template_parts_and_hash(&c2);
    assert_eq!(h1, h2);
}

#[test]
fn reputation_source_not_empty() {
    let src = daglock_contracts::daglock_reputation_source();
    assert!(!src.is_empty());
}
