# WASM SDK

**Source**: `wasm-sdk/src/`  **Updated**: `2026-06-02`  (1 file)

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
| `lib.rs` | module | `wasm-sdk/src/lib.rs` | WASM-bindgen exports for browser use |

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

### Usability Gap (Blocks Web Real Usage)

| ID | Finding | Impact |
|----|---------|--------|
| **U2** (web) | **WASM SDK missing `assemble_unsigned_tx` export** — Web `CreateEscrowForm` needs to compile covenant AND assemble unsigned transaction for KasWare signing. Currently only `compile_escrow` exists. | Web can't create real escrows without this. |

### Code Quality Issues

| ID | Finding | Impact |
|----|---------|--------|
| **Q2/Q3** | Fee denominator `200` not exposed — web has to hardcode or compute from amount | Expose `FEE_DENOMINATOR` constant via WASM |
| **Q4** | `trade_hash` validation not exported — web validates in JS but should use shared logic | Export `validate_trade_hash` via WASM |

### Fix Plan (Phase 0 + Phase 2)

1. **Task 1 (Phase 0):** Create `shared` crate with `FEE_DENOMINATOR` and `validate_trade_hash`
2. **Task 2 (Phase 0):** WASM SDK re-exports shared validation helpers
3. **Task 10 (U2, Phase 2):** Add `assemble_create_escrow`, `assemble_swap`, `assemble_refund` to WASM exports
   - Input: compiled script + constructor params + amount + output index
   - Output: unsigned transaction hex ready for KasWare `signTransaction`

### Required WASM Exports for Web U2 Fix

```rust
#[wasm_bindgen]
pub fn compile_daglock_escrow(params: JsValue) -> Result<JsValue, JsValue> {
    // Returns { script: hex, template_hash: hex, abi: [...] }
}

#[wasm_bindgen]
pub fn assemble_create_escrow(script_hex: &str, amount_sompi: u64, output_index: u32, recipient_pk: &str) -> Result<String, JsValue> {
    // Returns unsigned transaction hex
}

#[wasm_bindgen]
pub fn assemble_swap_escrow(script_hex: &str, amount_sompi: u64, output_index: u32, preimage: &str) -> Result<String, JsValue> {
    // Returns unsigned transaction hex for atomic swap
}

#[wasm_bindgen]
pub fn assemble_refund_escrow(script_hex: &str, amount_sompi: u64, output_index: u32) -> Result<String, JsValue> {
    // Returns unsigned transaction hex for refund
}

#[wasm_bindgen]
pub fn validate_trade_hash(hash_hex: &str) -> Result<JsValue, JsValue> {
    // Returns { valid: bool, bytes: Uint8Array } or error
}

#[wasm_bindgen]
pub const FEE_DENOMINATOR: u32 = 200;
```

### Dependencies

- `shared` crate (Phase 0) for constants and validation
- `kaspa-txscript` / `kaspa-consensus-core` for transaction assembly (already in contracts)
- `wasm-bindgen` for JS interop

### Verification

- [ ] `cargo test -p daglock-wasm-sdk` passes
- [ ] `cd web && npm run build` — WASM module loads without errors
- [ ] Manual: Web `CreateEscrowForm` → WASM compile → WASM assemble → KasWare sign → broadcast → indexer accepts
- [ ] Manual: WASM `validate_trade_hash` rejects malformed input, accepts valid 64-char hex

