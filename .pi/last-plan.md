# DagLock Improvement Plan — Phase 1 Quick Wins

## Goal
Fix 4 critical issues in the DagLock indexer: expiration logic, reputation formula, receipt verification, and row mapper safety.

## Tasks (in execution order)

### Task 3: Fix reconciliation expiration logic
**File:** `indexer/src/db/queries.rs` — `reconcile_expired_escrows()`
**Problem:** Uses wall-clock 24h threshold (`now - 86_400`) which prematurely expires long-lived escrows.
**Fix:** 
- Remove the timestamp threshold
- Only expire when `expiration_daa_score` is set AND `expiration_daa_score <= current_chain_daa_score`
- Need to fetch current DAA score from wRPC node or pass it as parameter
- For now: pass current DAA score as parameter, caller provides it

### Task 8: Reconcile reputation formula
**Files:** `indexer/src/db/queries.rs`, `docs/ARCHITECTURE.md`
**Problem:** Code formula `(trade+volume)*age*quality` doesn't match docs `log(trade)*log(volume)*(1-dispute)*age`
**Fix:** Update docs to match code (code is more reasonable — log formula was aspirational)
- Update ARCHITECTURE.md § Reputation Model to match actual implementation

### Task 1: Fix receipt verification flags
**File:** `indexer/src/db/queries.rs` — `receipt_from_escrow()`
**Problem:** Hardcodes `covenant_verified: true` and `fee_compliant: fee_sompi >= 0` (always true)
**Fix:**
- `covenant_verified`: Check that template_hash is non-empty (basic check)
- `fee_compliant`: Verify `fee_sompi == amount_sompi / 200`
- `signatures_verified`: Keep as-is (correct — settled/refunded implies signatures were valid)

### Task 11: Add try_get() to row mappers
**File:** `indexer/src/db/queries.rs` — `row_to_escrow()`, `row_to_offer()`
**Problem:** Uses `row.get()` which panics on type mismatches
**Fix:** Use `row.try_get()` with `.unwrap_or_default()` for optional fields, proper error handling for required fields

## Verification
After each task:
1. `cargo test --workspace` — all existing tests pass
2. `cargo clippy -- -D warnings` — no new warnings
3. Manual review of changed code

## Files affected
- `indexer/src/db/queries.rs` (tasks 3, 1, 11)
- `docs/ARCHITECTURE.md` (task 8)

## Risks
- Changing expiration logic could affect existing test fixtures
- Receipt verification change changes API response format (breaking?)
