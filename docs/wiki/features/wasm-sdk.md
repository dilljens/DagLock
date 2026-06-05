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
