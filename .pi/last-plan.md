# Plan: Two-Tier Dispute Resolution

## Goal
Add both off-chain dispute evidence/reputation (Tier 1) and on-chain optional arbiter covenant (Tier 2).

---

## Architecture

### Tier 1 — Off-chain (no covenant changes)
- Evidence log: parties submit signed proof to the indexer
- Outcome-weighted reputation: dispute outcomes affect score
- Resolve flow: mark disputes as expunged (filer penalty) or upheld (defendant penalty)

### Tier 2 — On-chain (optional arbiter covenant)
- New `daglock_arbiter.sil` — daglock.sil + `arbiterKey` param
- 2 new entrypoints: `disputeSellerWins(arbiterSig, sellerSig)` and `disputeBuyerWins(arbiterSig, buyerSig)`
- Mediator never acts alone — always needs the winning party's signature
- `arbiterKey = [0u8; 32]` → dispute paths are unreachable

---

## Steps

### 1. Create `contracts/src/daglock_arbiter.sil`
- Copy of `daglock.sil` with:
  - Constructor param `byte[32] arbiterKey` appended after `treasuryKey`
  - Entrypoint `disputeSellerWins(sig arbiterSig, sig sellerSig)`
  - Entrypoint `disputeBuyerWins(sig arbiterSig, sig buyerSig)`

### 2. Update `contracts/src/lib.rs`
- Add `daglock_arbiter_source()` fn
- Add `compile_daglock_arbiter(...)` fn (6 constructor args)
- Add `ARBITRATE_SELLER_WINS` and `ARBITRATE_BUYER_WINS` constants
- Add tests

### 3. Create `contracts/tests/daglock_arbiter_tests.rs`
- All 5 path execution tests + negative cases

### 4. DB migration `004_create_dispute_evidence.sql`
- `dispute_evidence` table
- `ensure_mediator_key_column()` — `mediator_key TEXT` on escrows
- `ensure_dispute_outcome_columns()` — `dispute_outcome`, `dispute_resolved_at`

### 5. Update `indexer/src/db/schema.rs` — include migration

### 6. Update `indexer/src/db/queries.rs` — evidence CRUD, mediator key, dispute resolve

### 7. Create `indexer/src/api/evidence.rs` — evidence + resolve-dispute endpoints

### 8. Update `indexer/src/api/mod.rs` — wire new routes

### 9. Update `indexer/src/types.rs` — new types

### 10. Update `indexer/src/api/escrows.rs` — mediator_key field

### 11. Update `indexer/src/api/reputation.rs` — outcome-weighted scoring

### 12. Update `web/src/api.ts` — evidence + arbiter endpoints

### 13. Update `web/src/App.tsx` — evidence form, mediator checkbox, resolve button

### 14. Update `web/src/styles.css` — new component styles

### 15. Verify: `cargo test --workspace` + `cd web && npm run build`

---

## Risks
- 5 entrypoints in arbiter variant — compiler handles fine
- Zeroed arbiterKey: dispute paths computationally unreachable
- Template hash divergence: indexer tracks both hashes

## Rollback
- `git revert <commits>` + `DROP TABLE dispute_evidence`

## Verification
1. `cargo test --workspace` — all pass
2. `cd web && npm run build` — clean
3. Manual: create arbiter escrow → dispute → submit evidence → resolve
