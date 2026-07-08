# Project: UI Overhaul + Pre-Mainnet Hardening

**Goal:** Fix the navigation overload (18 sidebar items, 10 escrow sub-tabs) and resolve remaining pre-mainnet hardening items before mainnet launch.

**Deadline:** Mainnet target June 30 — UI work can ship incrementally before and after.

---

## Requirements

- [ ] **R1** Sidebar navigation is grouped into focused sections, not a flat list
- [ ] **R2** Escrows page tabs consolidated — Milestones, Multi-party, Swaps integrated into detail views
- [ ] **R3** First-use experience guides users to the primary action
- [ ] **R4** Self-hosted Kaspa node enables real UTXO verification
- [ ] **R5** Remaining audit items resolved (H1, U7, Q4)

---

## Pre-resolved Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| Nav grouping | Collapsible sections in sidebar | Keeps all items accessible, no feature loss |
| Tab consolidation | Merge Milestones/Multi/Swaps into escrow detail — show/hide based on escrow type | Reduces tabs from 10→4 without removing functionality |
| First-use UX | Improved empty states + smart defaults, not a modal | Onboarding modal already exists (U7), needs polish not rebuild |
| Node | Follow existing `docs/local-testnet-node.md` plan | Already scoped, just wait on RAM upgrade |
| Audit fixes | H1 first (real security bug), then U7/Q4 | H1 is an actual vulnerability |
| No new dependencies | Still no external UI libraries | Keep bundle small, no framework churn |

---

## Track A: UI Information Architecture Overhaul `[ ]`
**Description:** Reduce cognitive load by grouping the sidebar, consolidating escrow tabs, and improving first-use guidance.
**⏱ Timebox:** 3-5 days

### Phase A1: Sidebar Navigation Redesign `[ ]`
**Goal:** Replace flat 18-item list with 6 collapsible groups.

**Current sidebar** (Sidebar.tsx):
```
Dashboard  Offers  Escrows  Swap  Vaults  Subscriptions
Reputation  Jury  Blog  Security  Merchant  Stats
Docs  Tokens  Create Token  Testnet  Settings  Help
```

**Proposed groups:**
```
📊 Overview
  Dashboard  Stats

🔄 Trade
  Offers  Escrows  Swap

🔒 Finance
  Vaults  Subscriptions

👥 Community
  Reputation  Jury  Settings

📚 Resources
  Blog  Security  Docs  Help

⚙️ Advanced
  Merchant  Tokens  Testnet   (Create Token inside Tokens page)
```

- [ ] Refactor `Sidebar.tsx` NAV_ITEMS into grouped structure
- [ ] Add collapsible section headers (default: Trade + Finance expanded, others collapsed)
- [ ] Remove "Create Token" from sidebar — move to a button inside Tokens page
- [ ] Remove "Swap" from sidebar if it duplicates Escrows → better: keep it
- [ ] Add quick-action buttons at top of sidebar: "Create Escrow", "Deposit"
- ✅ **Checkpoint:** Sidebar shows 6 groups, all 18 items still accessible
- ⚙ **Fallback:** Flat list with section dividers only (no collapse)

### Phase A2: Escrows Tab Consolidation `[ ]`
**Goal:** Reduce from 10 tabs to 4 by merging Milestones, Multi-party, and Swaps into the escrow detail view.

**Current tabs:**
```
My Escrows | My Swaps | Create | Lookup | Receipt | Invoice
Milestones | + Milestone | Multi | + Multi
```

**Proposed tabs:**
```
My Escrows | Create | Lookup | Receipt | Invoice
```
- Milestones and Multi-party are just escrow types with extra fields. The **My Escrows** list shows all escrows, with a badge for milestone/multi/swap type.
- The detail view adapts to show milestone progress bar, multi-party signature status, or swap hash based on the escrow type.
- "Create" detects the deal type and shows the appropriate form (standard / atomic swap / milestone / multi).
- "+ Milestone" and "+ Multi" become deal type presets inside the Create flow.

- [ ] Remove separate tabs: My Swaps, Milestones, + Milestone, Multi, + Multi
- [ ] Add escrow type badge to escrow cards (standard, swap, milestone, multi)
- [ ] Make escrow detail view context-aware: show milestone progress for milestones, signature board for multi-party, hash/preimage for swaps
- [ ] Add deal type selector to Create flow (already exists as presets — just needs polish)
- [ ] Redirect old URLs (/milestones, /multi) to /escrows with a query param
- ✅ **Checkpoint:** Escrows page has 4-5 tabs. All escrow types visible in My Escrows.
- ⚙ **Fallback:** Keep Milestones as a subtab within Escrows (still better than separate)

### Phase A3: First-Use Experience `[ ]`
**Goal:** A new user should know what to do within 5 seconds.

- [ ] Polish onboarding modal (U7 audit item): reduce to 3 slides, add "Connect test wallet" CTA
- [ ] Improve empty states: every empty page has a clear "do this next" action
- [ ] Dashboard shows quick-start steps for unconnected users (mirror the Reddit testnet tutorial flow)
- [ ] Add contextual tooltips on first visit to advanced features (Juror registration, Merchant API keys)
- [ ] Smart defaults: pre-fill amount with 100, pre-select OTC deal type on escrow create
- ✅ **Checkpoint:** Fresh incognito user can create+settle an escrow in under 2 minutes
- ⚙ **Fallback:** Ship empty-state improvements only, defer onboarding modal changes

### Phase A4: Visual Polish `[ ]`
**Goal:** Fix consistency issues across the UI.

- [ ] Consistent page header pattern (title + subtitle + action button)
- [ ] Loading states: all SkeletonTable replacements have matching component
- [ ] Mobile: sidebar drawer animation, tap targets at least 44px, horizontal scroll fixes
- [ ] Form validation: inline error messages, consistent styling
- [ ] Remove emoji from most UI text (keep in empty states only)
- [ ] Verify `` colors match design system variables
- ✅ **Checkpoint:** Lighthouse mobile score > 70 (from current likely ~50)
- ⚙ **Fallback:** Skip Lighthouse target, just fix the most obvious mobile breaks

---

## Track B: Pre-Mainnet Hardening `[ ]`
**Description:** Resolve remaining security/audit items and infrastructure gaps before mainnet.
**⏱ Timebox:** 3-5 days

### Phase B1: Self-Hosted Kaspa Node `[ ]`
**Goal:** Replace MockVerifier with real wRPC UTXO verification. Gated by 32GB RAM VPS upgrade (~July 13).

- [ ] OVHcloud VPS-2 (4 vCore, 8 GB, 75 GB NVMe) — or wait for RAM upgrade
- [ ] Install `kaspad` v2.0.1 with `--utxoindex`
- [ ] Wire indexer: `--wrpc-url ws://localhost:17210` + remove `--no-wrpc`
- [ ] Test `verify_utxo_exists()` against real node
- [ ] Add DAA score + sync status to `/v1/network` endpoint
- [ ] Add monitoring (systemd watchdog + healthchecks.io)
- ✅ **Checkpoint:** Indexer starts without `--no-wrpc`, `verify_utxo_exists()` returns correct results
- ⚙ **Fallback:** Keep MockVerifier as fallback with a warning log entry. Ship mainnet with "--no-wrpc" but mark "beta" until node is up.

### Phase B2: Audit Fixes `[ ]`
**Goal:** Close remaining audit items.

- [ ] **H1: Subscription rate limiting** — `daglock_subscription.sil` needs `require(tx.time >= startTime + currentPeriod * intervalSeconds)` — this is the unfixed HIGH finding from the audit. (1 day)
- [ ] **U7: Onboarding modal** — polish the existing modal (already wired, just needs content updates) (1 day)
- [ ] **Q4: TradeHash newtype** — add `TradeHash` struct with `FromStr` impl, replace raw string in API types (1 day)
- [ ] **H2: Minimum fee check** — verify dust-level escrows (inputValue < 200,000 sompi) pay at least 1 sompi fee (½ day)
- ✅ **Checkpoint:** `cargo test --workspace` passes. Audit checklist shows 30/30 complete.
- ⚙ **Fallback:** Defer Q4 and H2 to post-launch. Ship with H1 fixed.

### Phase B3: Final Testnet Verification `[ ]`
**Goal:** Run full lifecycle tests against deployed testnet (not just in-memory).

- [ ] Deploy latest indexer to VPS (already have `deploy-testnet.sh`)
- [ ] Run lifecycle tests against `api.daglock.com` (not localhost)
- [ ] Run manual walkthrough from `docs/manual-verification-plan.md`
- [ ] Verify all 4 Reddit tutorial screenshots still match current UI
- ✅ **Checkpoint:** All 303+ tests pass. Manual verification checklist 25/25 passes.
- ⚙ **Fallback:** Focus on automated tests only, skip manual walkthrough if time is tight

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Sidebar groups confuse existing users | Medium | Low | Ship as update with a "What's new" tooltip on first load |
| Tab consolidation breaks deep links | Medium | High | Add URL redirects for old paths (/milestones → /escrows?type=milestone) |
| Kaspa node RAM insufficient | High (32GB needed) | High | Fallback: ship mainnet with MockVerifier + "beta" label |
| UI changes introduce regressions | Low | Medium | Add Playwright smoke tests for main flows before shipping |
| Mainnet launch without real wRPC | High (if node isn't ready) | Medium | Be transparent: "DagLock beta — covenant verification pending. Escrows are trustless; indexer uses dev mode for UTXO display." |
