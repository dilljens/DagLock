# Findings: Escrow Feature Gap Analysis

## Requirements Discovery

**Q: What's the goal?**
A: Close feature gaps between DagLock and industry-standard escrow platforms — specifically high and medium effort items identified in the July 6, 2026 web research.

**Q: What independent workstreams exist?**
A: Six tracks — each is a separate feature that can be designed and implemented independently (they touch different covenant files and API modules).

**Q: What decisions need pre-resolution?**
A: See pre-resolved decisions table in task plan. Key ones: reuse existing covenant patterns, share FEE_DENOMINATOR, follow established module structure.

**Q: What are the constraints?**
A: Mainnet target (June 30) already passed. These are post-launch features. No new dependencies. Audit required on any new covenant before mainnet deployment.

**Q: What's the acceptance criteria?**
A: Each track has checkpoints: contract tests pass, indexer tests pass, web builds pass, bot tests pass. Full lifecycle tests for each new covenant.

**Q: What should each track NOT do?**
A: No new dependencies, no .unwrap() in production, no hardcoded addresses, no fee bypass paths.

---

## Pre-resolved Decisions

### Covenant Architecture
- **Milestone covenant**: Extend `daglock_vault.sil` time-lock pattern (proven, audited)
- **Subscription covenant**: New `daglock_subscription.sil` (design doc exists from 2026-07-03)
- **Multi-party covenant**: New `daglock_multi.sil` with split distribution
- **Security deposit covenant**: Wrapper pattern — `daglock_deposit.sil` wrapping existing escrow
- **Time-based auto-release**: Modify existing `daglock.sil` — add entrypoint, no new covenant
- **Dispute tiers**: Extend existing jury schema — no new covenant

Rationale: Minimize new code surface. Auto-release is one entrypoint change. Wrapper pattern for deposits means the inner escrow is already audited.

### Fee Model
- All covenants use `FEE_DENOMINATOR = 200` from `shared/src/constants.rs`
- Milestone: fee per release (each milestone line-item deducts 0.5%)
- Subscription: flat fee per draw OR one-time fee — covenant decision deferred to implementation
- Multi-party: single 0.5% on total before distribution
- Security deposit: no fee on deposit itself (not a transfer of value)

### Indexer Pattern
- New query modules follow `indexer/src/db/queries/<domain>.rs` pattern (established by A3 split)
- New API handlers follow `indexer/src/api/<domain>.rs` pattern
- Migration files: sequential after existing 026

### UI Pattern
- New page components in `web/src/components/`
- New routes in `App.tsx`
- Bot commands in `bot/src/index.js` (grammY pattern)
- WASM SDK changes only if new covenant compilation needed in browser

### Testing Strategy
- Covenant tests: `contracts/tests/` using TxScriptEngine (existing pattern)
- Indexer tests: lifecycle integration tests at `indexer/tests/`
- Web tests: Vitest + RTL at `web/src/__tests__/`
- Bot tests: existing bot test pattern

---

## Research Notes

### Industry Escrow Feature Comparison (July 6, 2026)

| Feature | Escrow.com | Bisq | HodlHodl | TrustSwap | DagLock |
|---------|-----------|------|----------|-----------|---------|
| Basic lifecycle | ✅ | ✅ | ✅ | ✅ | ✅ |
| KRC-20 tokens | ❌ | ❌ | ❌ | ❌ | ✅ |
| Time-lock vault | ❌ | ❌ | ❌ | ✅ | ✅ |
| Dispute resolution | ✅ manual | ✅ mediation | ❌ | ✅ | ✅ jury |
| Partial release | ✅ milestone | ❌ | ❌ | ✅ vesting | ❌ **GAP** |
| Subscriptions | ❌ | ❌ | ❌ | ✅ vesting | ❌ **GAP** |
| Multi-party | ✅ broker | ❌ | ❌ | ❌ | ❌ **GAP** |
| Auto-release | ❌ | ✅ | ✅ | ❌ | ❌ **GAP** |
| Escalation tiers | ✅ 3-level | ✅ 2-level | ❌ | ❌ | ❌ **GAP** |
| Security deposit | ✅ | ✅ arbitration | ❌ | ❌ | ❌ **GAP** |
| Telegram bot | ❌ | ❌ | ❌ | ❌ | ✅ |
| Atomic swaps | ❌ | ❌ | ❌ | ❌ | ✅ |
| Non-custodial | ❌ | ✅ | ✅ | ✅ | ✅ |
| On-chain rep | ❌ | ✅ | ❌ | ✅ | ✅ |

### Sources Consulted
- Wikipedia — Escrow (general escrow definitions, real estate, M&A patterns)
- Bisq.network — P2P decentralized exchange features (2-of-2 multisig, mediation, security deposit)
- HodlHodl — P2P Bitcoin escrow (multisig, non-custodial, 100+ currencies)
- TrustSwap — Token infrastructure (vesting, lock, subscription patterns)
- Escrow.com — Traditional escrow (milestones, multi-party, dispute tiers, broker model)
- Gemini Cryptopedia — Crypto escrow definitions
- Crypto.com — General product comparison

### Key Insights
1. **Auto-release** is the lowest-effort gap and most common across P2P platforms (Bisq, HodlHodl both have it)
2. **Milestone payments** are the #1 request for service escrows (freelancers, contractors)
3. **Multi-party** (buyer + seller + broker) unlocks the OTC broker use case DagLock targets
4. **Security deposit** is especially important for pseudonymous platforms — economic deterrent replaces identity trust
5. **No competitor covers all features** — DagLock's Telegram bot + on-chain rep + atomic swaps is already differentiated

---

## Open Questions → Resolved

- **Q:** Should milestone fees be per-release or total? → **A:** Covenant design decision deferred — per-release is simpler, total is more user-friendly. Revisit during Phase B1.
- **Q:** Should auto-release timer countdown be in blocks or wall time? → **A:** Blocks (consensus-native). UI converts to approximate wall time.
- **Q:** Is admin override on disputes safe? → **A:** Yes — admin is the indexer operator, and the covenant enforces outcomes anyway. Admin only casts tiebreaker on jury deadlock, not unilateral decisions.
