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
