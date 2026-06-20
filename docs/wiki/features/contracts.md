# contracts

SilverScript covenants for trustless escrow and atomic swaps on Kaspa L1. Six covenant files (KAS, KRC-20, Arbiter, Vault, VaultSoftlock, VaultMultisig) compiled via `silverscript-lang`. The `lib.rs` crate provides a Rust API for compilation and template hash extraction.

## Rules & Conventions

- ****S2**: KRC-20 fee validation only boolean — feePaid loop checks if *any* output pays treasury, not the correct 0.5%**
  - Status: ✅ Fixed | Domain: contracts
- ****S3**: KRC-20 KCC-20 output ownership validation**
  - Status: ⏭️ Closed | Domain: contracts
- ****Phase 0**: Shared crate with FEE_DENOMINATOR**
  - Status: ✅ Done | Domain: contracts
- ****Q2/Q3**: Magic number `200` hardcoded in 3 covenant files — no single source of truth**
  - Status: Consistency risk; `FEE_DENOMINATOR` constant now in `shared/src/constants.rs` | Domain: contracts
- ****Q4**: `trade_hash` handling: KAS/Arbiter use `byte[32]`, KRC-20 uses `byte[32]` — consistent but API validation missing**
  - Status: `daglock_shared::validate_trade_hash()` now validates 64-hex-char input on escrow creation | Domain: contracts
- **Phase 1: On-chain Reputation Covenant**
  - Completed June 18, 2026. d6f4eb9: daglock_reputation.sil + tests. 0f1c62c: template hash in indexer config + backfill script.

---
*Confidence: 0.95 · Last updated: 6/17/2026*