# Findings: Mainnet Readiness

## S3 Deep Dive
`daglock_krc20.sil` defines KCC-20 template metadata + `KCC20State` struct but NO entrypoint uses them. Multi-sig (both sigs + SIGHASH_ALL) mitigates the practical exploit, but zero defense-in-depth in the covenant itself.

**Fix attempt order:** ICC pattern in covenant first → off-chain verification fallback → docs-only.

## Infrastructure
- VPS (CX23) can't run kaspad — need external wRPC endpoint (`kaspa.infstone.io`)
- Current VPS issues: fd limit, root user, `--no-wrpc` mode

## Production `.unwrap()` Sites
9 call sites in production code (all UUID-gen except 1 mutex + 1 treasury key). UUID ones are "safe" (always succeed) but violate Rule #1.

## Hardcoded Fee Denominator
`indexer/src/api/escrows.rs:192` and `indexer/src/db/queries/offers.rs:276` use `amount_sompi / 200` instead of `daglock_shared::FEE_DENOMINATOR`.

## Flaky Crypto Tests
`std::sync::OnceLock<Mutex<()>>` causes `PoisonError` on parallel run. Fix: `parking_lot::Mutex`.

## Decision Log
| # | Decision | Rationale |
|---|----------|-----------|
| 1 | Try S3 covenant fix first | User preference |
| 2 | Off-chain fallback if A1 fails | Still ships with defense-in-depth |
| 3 | External wRPC for mainnet | VPS too small for kaspad |
