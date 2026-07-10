# Testing & Debugging Improvement Plan

**Goal:** Add comprehensive E2E tests for the three core user flows — wallet, offers, escrows.

**Current E2E state:** 8 wallet tests, 5 escrow-create tests, 3 error-state tests = 16 tests. Good foundation, major gaps in offer lifecycle and wallet error handling.

---

## Pre-resolved Decisions

- **Wallet:** Mock via `page.addInitScript` (no real KasWare extension) — already implemented in `e2e/helpers/kasware.ts`
- **API:** Route-level mocks via `page.route()` — already implemented in `e2e/helpers/api.ts`
- **Runner:** Playwright v1.61 — already configured in `playwright.config.ts`
- **CI:** GitHub Actions job `e2e` — needs to be added to `.github/workflows/ci.yml`
- **Data:** Use `fixtures.ts` auto-setup (dismisses onboarding, applies mocks)

---

## Track A: Wallet Error Handling `[ ]`

**Description:** Test wallet connection error states that the user actually hits — KasWare rejection, network mismatch, account switch, signing failure.

📏 **Scope:** 1 new file (`e2e/wallet-errors.spec.ts`), ~80 lines

### Phase A1: Connection Errors `[ ]`
- [ ] KasWare `requestAccounts` returns empty array → shows error message
- [ ] KasWare `requestAccounts` throws → shows error message
- [ ] KasWare `getNetwork` throws → still connects (graceful degradation)
- [ ] KasWare `getBalance` throws → still connects (graceful degradation)
- [ ] Multiple rapid connect clicks → only one connect attempt
- 📏 Scope: 1 file, ~40 lines
- ✅ Checkpoint: `npx playwright test e2e/wallet-errors.spec.ts --project=kasware-wallet`
- 📋 Verifies: Wallet error states are handled without crashing the UI
- ⚙ Fallback: If selectors differ, use text-based selectors (getByText, getByRole)

### Phase A2: Runtime Errors `[ ]`
- [ ] User rejects KasWare signMessage prompt → shows "User rejected" error
- [ ] KasWare disconnects mid-session → UI shows disconnected state
- [ ] Account change (different address selected) → address updates
- [ ] Network change → network badge updates
- 📏 Scope: 1 file, ~40 lines
- ✅ Checkpoint: All wallet error tests pass
- 📋 Verifies: Runtime wallet events don't crash the app
- ⚙ Fallback: Use `page.evaluate` to manually fire KasWare events from test
- Depends on: A1

---

## Track B: Offer Lifecycle E2E `[ ]`

**Description:** Test the full offer lifecycle from the user's perspective — browse, create, accept, cancel. This is the flow that's been broken.

📏 **Scope:** 1 new file (`e2e/offer-lifecycle.spec.ts`), ~120 lines

### Phase B1: Browse & Create `[ ]`
- [ ] Offers page shows "No open offers" when empty (mock empty response)
- [ ] Create offer form renders all fields (side, asset, amount, memo)
- [ ] Creating an offer triggers KasWare signMessage → offer appears in list
- [ ] Invalid address shows validation error
- 📏 Scope: ~60 lines
- ✅ Checkpoint: `npx playwright test e2e/offer-lifecycle.spec.ts --project=kasware-wallet`
- 📋 Verifies: Offer creation flow works end-to-end with KasWare signing
- ⚙ Fallback: Use `page.route` to override API responses per test
- Depends on: A1 (wallet mock infrastructure)

### Phase B2: Accept & Cancel `[ ]`
- [ ] Browse offers shows mock offers from API
- [ ] Accept button calls `api.acceptOffer` with auth headers
- [ ] Accepting an offer shows success notification
- [ ] Cancelling own offer calls `api.cancelOffer` with auth headers
- [ ] Cancelling shows success notification
- 📏 Scope: ~60 lines
- ✅ Checkpoint: All offer lifecycle tests pass
- 📋 Verifies: Offer accept/cancel flow works with KasWare signing
- ⚙ Fallback: Mock the notification component if toast assertions are flaky
- Depends on: B1

---

## Track C: Escrow Settle/Refund/Dispute E2E `[ ]`

**Description:** Test escrow lifecycle actions that require KasWare signing (settle, refund, dispute).

📏 **Scope:** 1 new file (`e2e/escrow-actions.spec.ts`), ~100 lines

### Phase C1: Settle & Refund `[ ]`
- [ ] Escrow detail page shows settle/refund buttons for active escrow
- [ ] Settle triggers KasWare signMessage → shows success
- [ ] Refund triggers KasWare signMessage → shows success
- [ ] Settle without wallet shows connect prompt
- 📏 Scope: ~50 lines
- ✅ Checkpoint: `npx playwright test e2e/escrow-actions.spec.ts --project=kasware-wallet`
- 📋 Verifies: Escrow settle/refund works with KasWare signing
- ⚙ Fallback: Ensure API mock for settle/refund returns 200

### Phase C2: Dispute & Swap `[ ]`
- [ ] Dispute shows reason input form
- [ ] Submitting dispute triggers KasWare signMessage → shows success
- [ ] Swap page generates preimage and shows swap details
- 📏 Scope: ~50 lines
- ✅ Checkpoint: All escrow action tests pass
- 📋 Verifies: Dispute and swap flows work with KasWare signing
- ⚙ Fallback: Use `page.route` to return custom mock data for each test scenario
- Depends on: C1

---

## Track D: CI Integration `[ ]`

**Description:** Add E2E tests to GitHub Actions CI workflow.

📏 **Scope:** 1 file (`.github/workflows/ci.yml`), ~20 lines added

### Phase D1: CI Job `[ ]`
- [ ] Add `e2e` job to `.github/workflows/ci.yml`
- [ ] Install Playwright browsers in CI
- [ ] Start dev server, run E2E tests
- [ ] Upload Playwright traces on failure
- 📏 Scope: ~20 lines added to ci.yml
- ✅ Checkpoint: `gh workflow run CI` succeeds with E2E job passing
- 📋 Verifies: E2E tests run automatically on every push to main
- ⚙ Fallback: Run E2E as a separate workflow that triggers only on web/ changes
- Depends on: A, B, C (all E2E tests written and passing locally first)
