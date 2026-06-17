# WASM SDK

**Source**: `wasm-sdk/src/`  **Updated**: `2026-06-16`  (1 file — 4 WASM exports)

## What it does
Browser/JavaScript SDK for assembling DagLock transactions in the web UI. Compiles covenants and constructs unsigned transactions that can be signed via KasWare browser extension.

## Architecture
```
wasm-sdk/src/lib.rs
    
     contracts::compile_daglock()     (covenant compilation)
     contracts::template_parts_and_hash() (template hash)
     wasm-bindgen exports             (JS API)
```

## Key functions / components
| Name | Kind | File:Line | Purpose |
|------|------|-----------|---------|
| `compile_daglock_escrow()` | function | `wasm-sdk/src/lib.rs` | Compile covenant with constructor params → `{ script, template_hash, abi }` |
| `compile_escrow()` | function | `wasm-sdk/src/lib.rs` | Legacy compile API |
| `kas_to_sompi()` | function | `wasm-sdk/src/lib.rs` | KAS decimal string → sompi u64 |
| `validate_trade_hash()` | function | `wasm-sdk/src/lib.rs` | Validate 64-hex-char trade hash |

## Data flow
1. Web UI calls WASM functions via `wasm-bindgen`
2. SDK compiles covenant with constructor params
3. Assembles unsigned transaction
4. Returns unsigned tx to web UI
5. Web UI passes to KasWare for signing

## Edge cases & gotchas
- Compiled to `wasm32-unknown-unknown` target
- No private key handling — unsigned tx only
- `getrandom` crate uses `js` feature for browser entropy

## Testing strategy
| Aspect | Approach |
|--------|----------|
| Unit tests | `cargo test -p daglock-wasm-sdk` (native, not wasm) |
| Integration tests | Browser-based testing (future) |
| Run command | `cargo test -p daglock-wasm-sdk` |

## Dependencies
| Depends on | For |
|------------|-----|
| `wasm-bindgen` | Rust↔JS interop |
| `js-sys` | JS object access |
| `web-sys` | Browser APIs |
| `daglock-contracts` | Covenant compilation |

## Consumed by
| Consumer | How |
|----------|-----|
| `web` | Import WASM module |

## Related domains
| Domain | Doc | Relationship |
|--------|-----|--------------|
| contracts | `features/contracts.md` | Core compilation logic |
| web | `features/web.md` | Consumer of WASM exports |

---

## Audit Findings (2026-06-06)

### Status (June 16, 2026) — Partially Resolved

| ID | Issue | Status | Fix |
|----|-------|--------|-----|
| **U2** (web) | WASM SDK missing `assemble_unsigned_tx` export | ❌ Open | `compile_daglock_escrow()` exists but `assemble_create_escrow` / `assemble_swap` / `assemble_refund` still needed |
| **Q2/Q3** | Fee denominator not exposed | ✅ Fixed | `kas_to_sompi()` exported; `FEE_DENOMINATOR` via shared crate |
| **Q4** | `validate_trade_hash` not exported | ✅ Fixed | `validate_trade_hash()` exported as WASM function |

### Required WASM Exports (Still Needed)

```rust
#[wasm_bindgen]
pub fn compile_daglock_escrow(params: JsValue) -> Result<JsValue, JsValue> {
    // ✅ DONE — Returns { script: hex, template_hash: hex, abi: [...] }
}

#[wasm_bindgen]
pub fn kas_to_sompi(amount_str: &str) -> Result<u64, JsValue> {
    // ✅ DONE — Converts "10.5" KAS → 1_050_000_000 sompi
}

#[wasm_bindgen]
pub fn validate_trade_hash(hash_hex: &str) -> Result<JsValue, JsValue> {
    // ✅ DONE — Returns { valid: bool, bytes: Uint8Array } or error
}

// STILL NEEDED:
#[wasm_bindgen]
pub fn assemble_create_escrow(script_hex: &str, amount_sompi: u64, output_index: u32, recipient_pk: &str) -> Result<String, JsValue> { ... }

#[wasm_bindgen]
pub fn assemble_swap_escrow(script_hex: &str, amount_sompi: u64, output_index: u32, preimage: &str) -> Result<String, JsValue> { ... }

#[wasm_bindgen]
pub fn assemble_refund_escrow(script_hex: &str, amount_sompi: u64, output_index: u32) -> Result<String, JsValue> { ... }
```

