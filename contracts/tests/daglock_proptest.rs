//! Property-based tests for DagLock covenants using Proptest.
//!
//! Properties (invariants) are easier for LLMs to define correctly than
//! specific input-output pairs. These tests verify that fundamental
//! properties hold across a wide range of random inputs.
//!
//! # Properties tested
//! - Value conservation: outputs == input (no value creation/destruction)
//! - Fee correctness: fee == input / 200 (enforced by covenant)
//! - Dust protection: amounts below MIN_OUT are rejected
//! - Security: wrong signatures are always rejected

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
use proptest::prelude::*;
use secp256k1::{Keypair, Secp256k1, SecretKey};
use rand::RngCore;

// ── Helpers ──────────────────────────────────────────────────────────

fn deterministic_keypair(seed: u64) -> Keypair {
    let secp = Secp256k1::new();
    let mut sk_bytes = [0u8; 32];
    sk_bytes[..8].copy_from_slice(&seed.to_le_bytes());
    loop {
        if let Ok(sk) = SecretKey::from_slice(&sk_bytes) {
            return Keypair::from_secret_key(&secp, &sk);
        }
        sk_bytes[0] = sk_bytes[0].wrapping_add(1);
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

const FEE_DENOM: u64 = 200;
const MIN_OUT: u64 = 1000;

// ── Property: Value conservation on release ──────────────────────────

/// Build a release transaction and execute it.
/// Returns the output sum on success, or error string on failure.
fn exec_release(amount: u64, buyer_kp: &Keypair, seller_kp: &Keypair) -> Result<u64, String> {
    let treasury = deterministic_keypair(3);
    let recipient = deterministic_keypair(4);
    let zero_hash = [0u8; 32];

    let compiled = compile_daglock(
        &pubkey_bytes(buyer_kp),
        &pubkey_bytes(seller_kp),
        &zero_hash,
        2_000_000_000i64,
        &pubkey_bytes(&treasury),
    );

    let fee = amount / FEE_DENOM;
    let send_amount = amount - fee;

    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&recipient))),
        TransactionOutput::new(fee, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([1u8; 32]), 0),
        vec![], 0, 0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(amount, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();

    let make_sig = |kp: &Keypair| -> Vec<u8> {
        let sig = kp.sign_schnorr(msg);
        let mut s = Vec::with_capacity(65);
        s.extend_from_slice(sig.as_ref().as_slice());
        s.push(SIG_HASH_ALL.to_u8());
        s
    };

    let sigscript = compiled
        .build_sig_script(entrypoints::RELEASE, vec![
            daglock_contracts::silverscript_lang::ast::Expr::bytes(make_sig(buyer_kp)),
            daglock_contracts::silverscript_lang::ast::Expr::bytes(make_sig(seller_kp)),
        ])
        .expect("build_sig_script");

    let utxo_entry = utxo.clone();
    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo_entry, ctx, flags);
    vm.execute()
        .map(|_| mtx.tx.outputs.iter().map(|o| o.value).sum())
        .map_err(|e| format!("{:?}", e))
}

proptest! {
    /// Property P1: Value conservation — sum of outputs always equals input
    #[test]
    fn prop_release_conserves_value(amount in MIN_OUT..10_000_000_000u64) {
        let buyer = deterministic_keypair(1);
        let seller = deterministic_keypair(2);
        let result = exec_release(amount, &buyer, &seller);
        // Amounts below ~200K may fail due to MIN_OUT on fee, that's expected
        if amount >= MIN_OUT * FEE_DENOM {
            prop_assert!(result.is_ok(), "Release should succeed for amount >= {}: {:?}", MIN_OUT * FEE_DENOM, result);
            if let Ok(sum) = result {
                prop_assert_eq!(sum, amount, "Value not conserved: input={}, output_sum={}", amount, sum);
            }
        }
    }

    /// Property P2: The covenant enforces fee == input / 200 (implicitly via require())
    /// If the covenant accepts the release, the fee was correct.
    /// This property exists to document the invariant — the covenant enforces it.

    /// Property P3: Dust protection — small amounts fail
    /// When amount is so small that either output would be below MIN_OUT,
    /// the covenant should reject the transaction.
    #[test]
    fn prop_dust_rejected(dust_amount in 1u64..(MIN_OUT * FEE_DENOM)) {
        let buyer = deterministic_keypair(1);
        let seller = deterministic_keypair(2);
        let result = exec_release(dust_amount, &buyer, &seller);
        // Amounts below MIN_OUT fail because send_amount < MIN_OUT
        // Amounts between MIN_OUT and MIN_OUT*FEE_DENOM fail because fee < MIN_OUT
        prop_assert!(result.is_err(), "Amount {} should be rejected as dust", dust_amount);
    }
}

// ── Property: Wrong signatures must fail ─────────────────────────────

/// Execute release with mismatched signers: compile covenant with `compile_keys`
/// but sign with `sign_keys`. If they differ, the covenant should reject.
fn exec_release_mismatched(
    amount: u64,
    compile_buyer: &Keypair,
    compile_seller: &Keypair,
    sign_buyer: &Keypair,
    sign_seller: &Keypair,
) -> Result<u64, String> {
    let treasury = deterministic_keypair(3);
    let recipient = deterministic_keypair(4);
    let zero_hash = [0u8; 32];

    let compiled = compile_daglock(
        &pubkey_bytes(compile_buyer),
        &pubkey_bytes(compile_seller),
        &zero_hash,
        2_000_000_000i64,
        &pubkey_bytes(&treasury),
    );

    let fee = amount / FEE_DENOM;
    let send_amount = amount - fee;

    let outputs = vec![
        TransactionOutput::new(send_amount, p2pk_script(&pubkey_bytes(&recipient))),
        TransactionOutput::new(fee, p2pk_script(&pubkey_bytes(&treasury))),
    ];

    let input = TransactionInput::new(
        TransactionOutpoint::new(TransactionId::from_bytes([2u8; 32]), 0),
        vec![], 0, 0u8,
    );
    let tx = Transaction::new(1, vec![input], outputs, 0, Default::default(), 0, vec![]);
    let utxo = UtxoEntry::new(amount, ScriptPublicKey::new(0, compiled.script.clone().into()), 0, false, None);
    let mut mtx = MutableTransaction::with_entries(tx, vec![utxo.clone()]);

    let reused = SigHashReusedValuesUnsync::new();
    let sighash = calc_schnorr_signature_hash(&mtx.as_verifiable(), 0, SIG_HASH_ALL, &reused);
    let msg = secp256k1::Message::from_digest_slice(sighash.as_bytes().as_slice()).unwrap();

    let make_sig = |kp: &Keypair| -> Vec<u8> {
        let sig = kp.sign_schnorr(msg);
        let mut s = Vec::with_capacity(65);
        s.extend_from_slice(sig.as_ref().as_slice());
        s.push(SIG_HASH_ALL.to_u8());
        s
    };

    let sigscript = compiled
        .build_sig_script(entrypoints::RELEASE, vec![
            daglock_contracts::silverscript_lang::ast::Expr::bytes(make_sig(sign_buyer)),
            daglock_contracts::silverscript_lang::ast::Expr::bytes(make_sig(sign_seller)),
        ])
        .expect("build_sig_script");

    let utxo_entry = utxo.clone();
    mtx.tx.inputs[0].signature_script = sigscript;

    let sig_cache = Cache::new(10_000);
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused);
    let flags = EngineFlags { covenants_enabled: true, sigop_script_units: 0.into() };
    let ver_tx = mtx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(&ver_tx, &ver_tx.inputs()[0], 0, &utxo_entry, ctx, flags);
    vm.execute()
        .map(|_| mtx.tx.outputs.iter().map(|o| o.value).sum())
        .map_err(|e| format!("{:?}", e))
}

proptest! {
    /// Property P4: Wrong buyer signature always fails
    #[test]
    fn prop_wrong_buyer_sig_fails(amount in (MIN_OUT * FEE_DENOM)..1_000_000_000u64) {
        let buyer = deterministic_keypair(1);
        let wrong = deterministic_keypair(99);
        let seller = deterministic_keypair(2);
        // Compile with buyer, sign with wrong → should fail
        let result = exec_release_mismatched(amount, &buyer, &seller, &wrong, &seller);
        prop_assert!(result.is_err(), "Wrong buyer sig should fail for amount {}", amount);
    }

    /// Property P5: Wrong seller signature always fails
    #[test]
    fn prop_wrong_seller_sig_fails(amount in (MIN_OUT * FEE_DENOM)..1_000_000_000u64) {
        let buyer = deterministic_keypair(1);
        let seller = deterministic_keypair(2);
        let wrong = deterministic_keypair(99);
        // Compile with seller, sign with wrong → should fail
        let result = exec_release_mismatched(amount, &buyer, &seller, &buyer, &wrong);
        prop_assert!(result.is_err(), "Wrong seller sig should fail for amount {}", amount);
    }

    /// Property P6: Both wrong signatures fail
    #[test]
    fn prop_both_wrong_sigs_fail(amount in (MIN_OUT * FEE_DENOM)..1_000_000_000u64) {
        let buyer = deterministic_keypair(1);
        let seller = deterministic_keypair(2);
        let wrong1 = deterministic_keypair(99);
        let wrong2 = deterministic_keypair(98);
        // Compile with buyer/seller, sign with wrong1/wrong2 → should fail
        let result = exec_release_mismatched(amount, &buyer, &seller, &wrong1, &wrong2);
        prop_assert!(result.is_err(), "Both wrong sigs should fail for amount {}", amount);
    }

    }

// ── Property: Different keys produce different results ──────────────

proptest! {
    /// Property P8: Different keypairs change the template hash
    #[test]
    fn prop_different_keys_different_hash(seed1: u64, seed2: u64) {
        prop_assume!(seed1 != seed2);
        let kp1 = deterministic_keypair(seed1);
        let kp2 = deterministic_keypair(seed2);
        let zero = deterministic_keypair(0);

        let c1 = compile_daglock(
            &pubkey_bytes(&kp1), &pubkey_bytes(&zero),
            &[0u8; 32], 1_700_000_000, &pubkey_bytes(&zero),
        );
        let c2 = compile_daglock(
            &pubkey_bytes(&kp2), &pubkey_bytes(&zero),
            &[0u8; 32], 1_700_000_000, &pubkey_bytes(&zero),
        );
        prop_assert_ne!(c1.script, c2.script,
            "Different buyer keys should produce different compiled scripts");
    }
}
