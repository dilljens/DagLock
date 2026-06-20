# cli

Command-line power-user tool for DagLock escrow operations. Connects to the indexer REST API for queries and assembles unsigned transactions for signing with kaspawallet or KasWare.

## Rules & Conventions

- ****U1**: CLI create uses dummy keys**
  - Status: ✅ Fixed | Domain: cli
- ****U3**: No wallet signing in CLI**
  - Status: ✅ Fixed | Domain: cli
- ****Q1**: `.unwrap()` in production**
  - Status: ✅ Fixed | Domain: cli
- ****Q2/Q3**: Magic number `200` in `tx.rs`**
  - Status: ✅ Fixed | Domain: cli
- ****Q4**: `trade_hash` not validated**
  - Status: ✅ Fixed | Domain: cli

---
*Confidence: 0.95 · Last updated: 6/17/2026*