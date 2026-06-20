# wasm-sdk

Browser/JavaScript SDK for assembling DagLock transactions in the web UI. Compiles covenants and constructs unsigned transactions that can be signed via KasWare browser extension.

## Rules & Conventions

- ****U2** (web): WASM SDK missing `assemble_unsigned_tx` export**
  - Status: ❌ Open | Domain: wasm-sdk
- ****Q2/Q3**: Fee denominator not exposed**
  - Status: ✅ Fixed | Domain: wasm-sdk
- ****Q4**: `validate_trade_hash` not exported**
  - Status: ✅ Fixed | Domain: wasm-sdk

---
*Confidence: 0.95 · Last updated: 6/17/2026*