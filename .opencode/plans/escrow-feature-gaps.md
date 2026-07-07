# Plan: Escrow Feature Gap — High & Medium Effort Items

> **Origin:** Web research gap analysis (July 6, 2026) comparing DagLock features against industry-standard escrow platforms (Escrow.com, Bisq, HodlHodl, TrustSwap).
>
> **Status:** Plan created — tracks scoped, decisions pre-resolved. Awaiting execution.

---

## Goal

Close 5 feature gaps identified in the gap analysis: milestone payments, recurring subscriptions, multi-party escrow, dispute escalation tiers, and security deposits. Plus 1 low-effort quick win (time-based auto-release).

---

## Requirements

- [ ] **R1** Milestone / partial payment escrows — release funds in stages
- [ ] **R2** Subscription / recurring payment covenant — scheduled draws
- [ ] **R3** Multi-party escrow — 3+ parties in one covenant (buyer + seller + broker)
- [ ] **R4** Dispute escalation tiers — starter mediation before full jury
- [ ] **R5** Security deposit covenant — both parties stake deterrent bonds
- [ ] **R6** Time-based auto-release — auto-settle when timer expires + no dispute

---

## Pre-resolved Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| Covenant pattern | All new covenants use existing ICC + template hash pattern | Consistency with daglock_krc20.sil, audit-tested |
| Fee model | 0.5% fee applies to all new covenants via FEE_DENOMINATOR | Shared constant, covenant-enforced |
| Existing patterns reuse | Milestone extends daglock_vault.sil timed-release pattern | Proven time-lock logic, reduce audit surface |
| Subscription pattern | New covenant `daglock_subscription.sil` (design doc exists) | Separate from one-shot escrow — different lifecycle |
| Multi-party pattern | Single covenant with N beneficiaries, N signatures to release | UTXO stays atomic, covenant handles splits |
| Dispute escalation | Time-based auto-escalation: mediation (2d) → jury (5d) → admin override (10d) | Minimal code change to existing jury system |
| Security deposit | Separate covenant wrapping existing daglock.sil as inner | Clean separation, reusable with any escrow type |
| Time-based auto-release | Modify existing daglock.sil settle path — check timer + dispute flag | Low effort, one entrypoint addition |
| Indexer changes | New query modules per domain, same pattern as escrow_service.rs | Consistency with A3 split |
| UI | New pages + components following existing web/ pattern | Use existing App.tsx routing + component pattern |
| Bot | New commands following existing grammY pattern | Consistency with bot/ commands |
| Testing | Existing covenant execution test pattern + lifecycle integration tests | Follows A4 pattern |
| Priority | Time-based auto-release first (quick win), then milestone + subscription (adjacent covenant work), then multi-party + security deposit + dispute tiers | Group covenant work to minimize context switching |

---

## Track A: Time-based Auto-Release `[ ]`

**Description:** Add auto-settle entrypoint to `daglock.sil` — when the escrow timer expires and no dispute is active, the locked funds are released to the seller without requiring the buyer's signature.

**Timebox:** 3-5 days

### Phase A1: Covenant change `[ ]` [3-4 hrs]
- [ ] Add `auto_settle()` entrypoint to `daglock.sil`
- [ ] Check: block number ≥ timeout + no dispute flag set
- [ ] On pass: release to seller with fee deduction same as normal settle
- [ ] On fail: locked to buyer
- [ ] Compile + verify template hash
- ✅ **Checkpoint:** `cargo test -p daglock-contracts` passes
- ⚙ **Fallback:** If covenant size limit hit, tighten timeout comparison logic

### Phase A2: Indexer support `[ ]` [1-2 days]
- [ ] Add systemd timer / cron job to check expired escrows every 10 min
- [ ] New `POST /v1/escrows/:id/auto-settle` endpoint (triggered by timer or user)
- [ ] OR broadcast auto-settle tx directly if indexer has signing capability
- ✅ **Checkpoint:** `cargo test -p daglock-indexer` passes
- ⚙ **Fallback:** Start with user-triggered endpoint, add auto-timer later

### Phase A3: UI + Bot hints `[ ]` [1 day]
- [ ] Web: show "Auto-settle in 3 days" countdown on escrow detail
- [ ] Bot: `/status` shows auto-settle time
- [ ] Notification: email/bot ping when escrow auto-settles
- ✅ **Checkpoint:** `cd web && npm test && npm run build` + bot test
- ⚙ **Fallback:** Just indexer + notification, skip UI countdown

---

## Track B: Partial / Milestone Payments `[ ]`

**Description:** New covenant `daglock_milestone.sil` that holds total funds and releases them in N stages. Each milestone has a condition (signer approval or timer). Supports 2-10 milestones.

**Timebox:** 3-4 weeks

### Phase B1: Covenant design + implementation `[ ]` [1-2 weeks]
- [ ] Design `daglock_milestone.sil` data model:
  - `total_amount: u64` — total locked
  - `milestones: [Milestone; N]` — each with amount, condition (time or sig), released flag
  - `current_milestone: u32`, `dispute_flag: bool`
  - Beneficiaries: seller gets milestone releases, buyer can approve/veto each
- [ ] Entrypoints:
  - `release_milestone(idx)` — seller claims milestone if condition met
  - `approve_milestone(idx)` — buyer approves early release
  - `dispute_milestone(idx)` — buyer disputes specific milestone
  - `refund_remaining()` — buyer gets back un-released funds
- [ ] Fee: 0.5% on each milestone release (or bulk at end — TBD)
- [ ] Template hash extraction
- ✅ **Checkpoint:** `cargo test -p daglock-contracts` — new milestone tests pass
- ⚙ **Fallback:** Start with fixed 3-milestone max, generalize later

### Phase B2: Indexer milestone tracking `[ ]` [1 week]
- [ ] New `milestones` table (escrow_id, idx, amount, status, released_at)
- [ ] New query module `indexer/src/db/queries/milestones.rs`
- [ ] New endpoints:
  - `GET /v1/escrows/:id/milestones` — list milestones
  - `POST /v1/escrows/:id/milestones/:idx/release` — release milestone tx
  - `POST /v1/escrows/:id/milestones/:idx/approve` — buyer approves
- [ ] Template hash registration for milestone covenant
- ✅ **Checkpoint:** `cargo test -p daglock-indexer` + lifecycle tests
- ⚙ **Fallback:** Reuse escrows table with milestone JSON field, skip dedicated table

### Phase B3: Web UI `[ ]` [1 week]
- [ ] New page `/escrows/create-milestone` — milestone config wizard (add milestones, amounts, conditions)
- [ ] `MilestoneProgressBar` component — visual progress
- [ ] Escrow detail: show milestone table with release/approve buttons
- [ ] Activity log per milestone
- ✅ **Checkpoint:** `cd web && npm test && npm run build`
- ⚙ **Fallback:** Show milestone info in escrow detail page, skip dedicated create wizard

### Phase B4: Bot support `[ ]` [2-3 days]
- [ ] `/create_milestone` — wizard with milestone steps
- [ ] `/milestones` — list milestones for an escrow
- [ ] `/release` — release specific milestone
- [ ] `/approve` — approve milestone as buyer
- ✅ **Checkpoint:** `cd bot && npm test`
- ⚙ **Fallback:** Bot links to web UI for complex milestone config

---

## Track C: Subscription / Recurring Payments `[ ]`

**Description:** Implement `daglock_subscription.sil` covenant for recurring draws from a pre-funded UTXO. Design doc exists (from PENDING.md audit log entry 2026-07-03).

**Timebox:** 2-3 weeks

### Phase C1: Covenant implementation `[ ]` [1 week]
- [ ] Review existing `daglock_subscription.sil` design doc for entrypoints
- [ ] Implement:
  - `subscribe(funding_tx, amount_per_period, interval_blocks, max_periods)`
  - `draw(subscription_id)` — seller claims period payment
  - `cancel(subscription_id)` — buyer stops future draws
  - `refund_remaining()` — buyer gets back unused funds
- [ ] Fee: 0.5% per draw OR flat per subscription — pick one
- [ ] Template hash
- ✅ **Checkpoint:** `cargo test -p daglock-contracts` — subscription tests pass
- ⚙ **Fallback:** Flat fee per subscription (simpler covenant math)

### Phase C2: Indexer + scheduler `[ ]` [1 week]
- [ ] New `subscriptions` table + query module
- [ ] REST endpoints:
  - `POST /v1/subscriptions` — create
  - `GET /v1/subscriptions/:id` — status
  - `POST /v1/subscriptions/:id/draw` — seller triggers draw
  - `POST /v1/subscriptions/:id/cancel` — buyer cancels
- [ ] Systemd timer: auto-draw service that checks due subscriptions
- [ ] Event hooks: notify buyer on each draw
- ✅ **Checkpoint:** `cargo test -p daglock-indexer`
- ⚙ **Fallback:** No auto-draw timer — seller triggers manually

### Phase C3: UI + Bot `[ ]` [3-5 days]
- [ ] Web: subscription management page (active, history, cancel)
- [ ] Bot: `/subscribe`, `/subscriptions`, `/cancel_subscription`
- [ ] Dashboard widget: "Active subscriptions" count + next draw
- ✅ **Checkpoint:** `cd web && npm test && npm run build` + bot test
- ⚙ **Fallback:** Web-only, skip bot commands

---

## Track D: Multi-Party Escrow `[ ]`

**Description:** New covenant supporting 3+ parties — e.g., buyer, seller, and broker all get paid from one escrow. Distribution ratios defined at creation.

**Timebox:** 2-3 weeks

### Phase D1: Covenant `[ ]` [1 week]
- [ ] Design `daglock_multi.sil`:
  - `parties: [Party; M]` — each with address, share_basis_points (e.g., 7000/2000/1000)
  - `release_threshold: u32` — signatures required to release (e.g., M-1)
  - Entrypoints: `release(signatures[])`, `dispute()`, `refund()`
- [ ] Share math: `amount = total * share_bp / 10000` per party
- [ ] Fee: 0.5% from total before distribution
- ✅ **Checkpoint:** `cargo test -p daglock-contracts`
- ⚙ **Fallback:** Start with fixed 3-party, extend to N later

### Phase D2: Indexer `[ ]` [3-5 days]
- [ ] Extend `escrows` table: `parties JSON` field or separate `escrow_parties` table
- [ ] New endpoint: `POST /v1/escrows/:id/release-multi` (submits multi-sig release)
- [ ] Signature collection: track which parties have signed
- ✅ **Checkpoint:** `cargo test -p daglock-indexer`
- ⚙ **Fallback:** Track signatures in-memory with expiry, skip DB persistence

### Phase D3: Web + Bot `[ ]` [3-5 days]
- [ ] Escrow creation: "Add broker" toggle, percentage split inputs
- [ ] Release flow: collect signatures from each party
- [ ] Status page: show each party, their share, signatory status
- [ ] Bot: signature request notifications
- ✅ **Checkpoint:** `cd web && npm test`
- ⚙ **Fallback:** CLI-only multi-party, skip web wizard initially

---

## Track E: Security Deposit Covenant `[ ]`

**Description:** Wrapper covenant that holds a deposit from both parties alongside the main escrow. Forfeited to the counterparty on proven bad behavior (arbitrated).

**Timebox:** 1-2 weeks

### Phase E1: Covenant `[ ]` [3-5 days]
- [ ] Design `daglock_deposit.sil`:
  - Wraps an inner escrow (any `daglock_*.sil` covenant)
  - `deposit_buyer: u64`, `deposit_seller: u64`
  - Forfeit conditions: jury ruling, timeout without action
  - Entrypoints: `lock()` (both deposit), `forfeit(party)`, `release_deposits()`
- [ ] Default deposit: 1% of escrow amount (configurable at creation)
- ✅ **Checkpoint:** `cargo test -p daglock-contracts`
- ⚙ **Fallback:** Fixed 1% deposit only, no custom amounts

### Phase E2: Indexer + Jury integration `[ ]` [3-5 days]
- [ ] New `deposits` table linked to escrows
- [ ] Jury verdicts can rule "forfeit deposit" as outcome
- [ ] Endpoints:
  - `POST /v1/escrows/:id/deposit/lock` — lock deposit
  - `POST /v1/escrows/:id/deposit/forfeit` — trigger forfeit
  - `GET /v1/escrows/:id/deposit` — deposit status
- ✅ **Checkpoint:** `cargo test -p daglock-indexer`
- ⚙ **Fallback:** Jury rules "forfeit deposit" as a string verdict — manual execution

### Phase E3: UI `[ ]` [1-2 days]
- [ ] Escrow creation: toggle "Security deposit (recommended)" + amount
- [ ] Escrow detail: show deposit status, forfeit button
- [ ] Jury panel: show deposit at stake
- ✅ **Checkpoint:** `cd web && npm test`
- ⚙ **Fallback:** Show deposit info in escrow detail only, skip creation wizard

---

## Track F: Dispute Escalation Tiers `[ ]`

**Description:** Add time-based auto-escalation to the existing jury system: mediation (2 days) → jury vote (5 days) → admin override (10 days). Each step auto-escalates if unresolved.

**Timebox:** 1-2 weeks

### Phase F1: Escalation logic `[ ]` [3-5 days]
- [ ] Add `escalation_level` and `escalation_deadline` fields to disputes table
- [ ] Levels: `0=mediation`, `1=jury`, `2=admin`
- [ ] Mediation: buyer + seller chat with encrypted messaging (already exists)
- [ ] Auto-escalation: if no resolution within 2d, move to jury
- [ ] Admin override: if jury can't reach majority, admin steps in
- [ ] Notifications at each escalation step
- ✅ **Checkpoint:** `cargo test -p daglock-indexer`
- ⚙ **Fallback:** Mediation step only (skip admin override)

### Phase F2: UI + Bot `[ ]` [2-3 days]
- [ ] Jury page: show escalation level + deadline for each dispute
- [ ] Bot: `/dispute` shows escalation status
- [ ] Notifications: "Your dispute has escalated to jury vote"
- ✅ **Checkpoint:** `cd web && npm test`
- ⚙ **Fallback:** Bot-only display, skip web

---

## Execution Strategy

```
Priority 1 (Quick Win):
  Track A — Time-based auto-release (3-5 days)

Priority 2 (Covenant batch — minimize context switching):
  Track B — Milestone payments (3-4 weeks)
  Track C — Subscriptions (2-3 weeks)

Priority 3 (Infrastructure + dispute):
  Track F — Dispute escalation tiers (1-2 weeks)

Priority 4 (Advanced covenant patterns):
  Track D — Multi-party escrow (2-3 weeks)
  Track E — Security deposit (1-2 weeks)
```

Tracks within the same priority can be worked in parallel since they touch different covenant files. Each track's covenant, indexer, and UI phases must be sequential within the track.

---

## Anti-scope (what this plan does NOT include)

- Native mobile app
- KYC integration
- Cross-chain (BTC/ETH/LTC) — already in PENDING.md Phase 6+
- NFT / collectible escrow
- AMM / liquidity pools — already in PENDING.md
- Price oracle covenant changes
- Volume-based fee rebates — already deferred in PENDING.md
