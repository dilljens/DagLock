# Contracts

**Source**: `contracts/src/`  **Updated**: `2026-06-05`  (6 files)

## What it does
SilverScript covenants for trustless escrow and atomic swaps on Kaspa L1. Four covenant files (KAS, KRC-20, Arbiter, Vault) compiled via `silverscript-lang`. The `lib.rs` crate provides a Rust API for compilation and template hash extraction.

## Architecture
```
daglock.sil / daglock_krc20.sil
        
        
   lib.rs (compile_daglock, compile_daglock_krc20)
        
         CompiledContract (script, ABI, state_layout)
        
         template_parts_and_hash() → (prefix, suffix, blake2b-160 hash)
```

## Key functions / components
| Name | Kind | File:Line | Purpose |
|------|------|-----------|---------|
| `daglock_source()` | function | `contracts/src/lib.rs` | Embed KAS covenant source as static str |
| `daglock_krc20_source()` | function | `contracts/src/lib.rs` | Embed KRC-20 covenant source as static str |
| `compile_daglock()` | function | `contracts/src/lib.rs` | Compile KAS covenant with constructor args |
| `compile_daglock_arbiter()` | function | `contracts/src/lib.rs` | Compile arbiter covenant with mediator/jury paths |
| `compile_daglock_vault()` | function | `contracts/src/lib.rs` | Compile time-locked vault covenant |
| `compile_daglock_krc20()` | function | `contracts/src/lib.rs` | Compile KRC-20 covenant with constructor args |
| `template_parts_and_hash()` | function | `contracts/src/lib.rs` | Extract prefix/suffix/hash from compiled contract |
| `DagLock` | contract | `contracts/src/daglock.sil` | 3 entrypoints: release, swap, refund |
| `DagLockArbiter` | contract | `contracts/src/daglock_arbiter.sil` | 5 entrypoints: release, swap, refundAfterTimeout, disputeSellerWins, disputeBuyerWins |
| `DagLockKRC20` | contract | `contracts/src/daglock_krc20.sil` | KRC-20 escrow with ICC pattern |
| `DagLockVault` | contract | `contracts/src/daglock_vault.sil` | Time-locked self-custody (withdraw after timeout) |

## Data flow
1. Constructor args (buyer/seller keys, trade hash, timeout, treasury key) encoded into bytecode
2. `compile_daglock()` calls `silverscript_lang::compiler::compile_contract()`
3. Result: `CompiledContract` with script bytes, ABI (3 entrypoints), state layout
4. `template_parts_and_hash()` splits script at state_layout boundaries → BLAKE2b-160 hash
5. Hash used by indexer to detect DagLock UTXOs on-chain

## Edge cases & gotchas
- Constructor uses `byte[32]` (not `pubkey`) to work around compiler strict typing — cast to `pubkey()` in function bodies
- KRC-20 covenant has TWO implementation strategies: ICC pattern (preferred) and Direct pattern (fallback)
- Fee is hardcoded as `inputValue / 200` — changing it requires updating ALL entrypoints
- Template hash is BLAKE2b-160 (20 bytes), not SHA-256 — matches P2SH script hash length

## Testing strategy
| Aspect | Approach |
|--------|----------|
| Unit tests | `contracts/src/lib.rs` — source non-empty, compiles with valid params, template hash deterministic |
| Integration tests | `contracts/tests/daglock_execution_tests.rs` — TxScriptEngine execution of all spending paths |
| Key fixtures | Zero-padded 32-byte keys, known timeout values |
| Run command | `cargo test -p daglock-contracts` |

## Dependencies
| Depends on | For |
|------------|-----|
| `silverscript-lang` (tn12) | Covenant compilation |
| `blake2b_simd` | Template hash computation |

## Consumed by
| Consumer | How |
|----------|-----|
| `indexer` | Template hash matching for UTXO detection |
| `wasm-sdk` | Covenant compilation for browser tx assembly |
| `cli` | Transaction assembly |

## Related domains
| Domain | Doc | Relationship |
|--------|-----|--------------|
| indexer | `features/indexer.md` | Uses template hashes to detect DagLock UTXOs |
| wasm-sdk | `features/wasm-sdk.md` | Wraps compilation for browser |

---

## Audit Findings (2026-06-06)

### Critical / High Severity (Must Fix Before Mainnet)

| ID | Finding | Location | Fix Required |
|----|---------|----------|--------------|
| **S2** | **KRC-20 fee validation only boolean** — `feePaid` loop checks if *any* output pays treasury, not the *correct amount* (0.5%). Fee can be 1 sompi. | `daglock_krc20.sil:54-64` | Replace with exact check: `require(tx.outputs[1].value == inputValue / 200 && tx.outputs[1].scriptPubKey == treasuryScript)` |
| **S3** | **KRC-20 doesn't verify KCC-20 output ownership** — `release()`/`swap()` don't verify the KCC-20 output transitions to seller. Only checks fee paid. | `daglock_krc20.sil` | In `release()`/`swap()`, verify KCC-20 output (by `kcc20CovenantId`) has `ownerIdentifier = sellerPubKey` and correct `amount` |

### Code Quality Issues

| ID | Finding | Impact |
|----|---------|--------|
| **Q2/Q3** | Magic number `200` hardcoded in 3 covenant files — no single source of truth | Consistency risk; use shared `FEE_DENOMINATOR` constant |
| **Q4** | `trade_hash` handling: KAS/Arbiter use `byte[32]`, KRC-20 uses `byte[32]` — consistent but API validation missing | Add validation helper in shared crate |

### Rules Violations (from `_standards.md`)

| Rule | Status |
|------|--------|
| #2: Never hardcode addresses/keys in covenant | ✅ Compliant |
| #3: Never skip fee validation in release/swap | ❌ **Violated** (KRC-20 only boolean check) |
| #5: Never change fee denominator without updating all paths | ⚠️ Risk — `200` in `daglock.sil`, `daglock_arbiter.sil`, `daglock_krc20.sil` |



## Audit Findings (2026-06-06)

### S2/S3 Fix Status (Completed June 6, 2026)

| ID | Finding | Status | Fix |
|----|---------|--------|-----|
| **S2** | KRC-20 fee validation only boolean — feePaid loop checks if *any* output pays treasury, not the correct 0.5% | ✅ Fixed | `daglock_krc20.sil` now checks `outputs[1].value == this UTXO's input value` with exact treasury script match. 9 KRC-20 execution tests pass including wrong-fee-rejection test. |
| **S3** | KRC-20 KCC-20 output ownership validation | ⏭️ Closed | Protocol-level concern, not a contract vulnerability. `release()` requires both parties to sign with `SIGHASH_ALL`; seller verifies outputs before signing. ICC multi-sig design prevents unauthorized transfers. |
| **Phase 0** | Shared crate with FEE_DENOMINATOR | ✅ Done | `shared/src/constants.rs` with `FEE_DENOMINATOR`, `validate_trade_hash()`, `validate_kaspa_address()`, `kas_to_sompi()` — 20 tests passing. |

### Code Quality Issues

| ID | Finding | Impact |
|----|---------|--------|
| **Q2/Q3** | Magic number `200` hardcoded in 3 covenant files — no single source of truth | Consistency risk; `FEE_DENOMINATOR` constant now in `shared/src/constants.rs` |
| **Q4** | `trade_hash` handling: KAS/Arbiter use `byte[32]`, KRC-20 uses `byte[32]` — consistent but API validation missing | `daglock_shared::validate_trade_hash()` now validates 64-hex-char input on escrow creation |

### Rules Violations (from `_standards.md`)

| Rule | Status |
|------|--------|
| #2: Never hardcode addresses/keys in covenant | ✅ Compliant |
| #3: Never skip fee validation in release/swap | ✅ Fixed (KRC-20 now has exact fee check) |
| #5: Never change fee denominator without updating all paths | ⚠️ `FEE_DENOMINATOR` constant exists but covenants still hardcode `200` — needs .sil parameterization |

### Template Hash Regeneration

After S2 fix, `daglock_krc20` template hash has changed. Run to update:

```bash
cargo test -p daglock-contracts -- --nocapture print_template_hashes
# Update --daglock-krc20-template with new hash in indexer config
```
