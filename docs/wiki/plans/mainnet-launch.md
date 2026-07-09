# Mainnet Launch Plan

> **Status:** Pre-launch. Toccata hard fork activated June 30, 2026. A direct competitor (Zilliqant's Escrow) already deployed on mainnet. Launch urgency: **HIGH**.
>
> **Target:** August 2026 (soft launch July — real wRPC verification, select users).
>
> **Current state:** Testnet with MockVerifier (offline mode). Blocked on RAM upgrade (~July 13) for local kaspad node. 376 tests passing. 28/30 audit items complete.

---

## Competitive Context

Research (July 8, 2026) found:

| Factor | Signal |
|--------|--------|
| **Zilliqant's Escrow** | Direct competitor live on mainnet — same covenant-based model, AI mediator, human arbiter. Posted on r/kaspa (45pts, 1 day ago). |
| **Community trust deficit** | CEX withdrawal freezes, Discord OTC shut down, Reddit bans OTC posts. Users are desperate for trustless alternatives but skeptical. |
| **Multiple people asked** | "Why would anyone trust your software?" — trust is earned through transparency, open-source code, and real usage. |
| **KRC-20 is underserved** | No existing OTC infrastructure for KRC-20 token block trades. This is DagLock's unique wedge. |
| **KaspaCom, KasBonds adjacent** | Could expand into escrow. First-mover advantage is finite. |

---

## Launch Phases

### Phase 0: Infrastructure (Weeks 1-2) ⏳ Blocked

**Dependency:** RAM upgrade to 32 GB (~July 13) for local kaspad node.

| Task | Effort | Who | Done? |
|------|--------|-----|-------|
| Install kaspad testnet-11 with `--utxoindex` | 1 hr | Ops | ❌ |
| Sync testnet-11 node (~20-50 GB) | 4-12 hrs | Passive | ❌ |
| Switch indexer from `--no-wrpc` to wRPC | 2 hr | Dev | ❌ |
| Verify `EscrowVerifier::verify_utxo_exists()` with real node | 1 hr | Dev | ❌ |
| Verify `AnchorService::flush_pending()` broadcasts txs | 2 hr | Dev | ❌ |
| DAA score tracking for vault sweep + offer expiry | 1 hr | Dev | ❌ |
| Keep `--no-wrpc` fallback flag | 30 min | Dev | ⬜ Already designed |
| Rotate Cloudflare API token + DAGLOCK_MESSAGE_KEY | 30 min | Ops | ❌ Needs you |

**Deliverable:** Indexer running with real wRPC verification on testnet-11.

---

### Phase 1: Production Hardening (Week 3)

Based on `docs/wiki/plans/production-hardening.md`.

| Task | Effort | Priority |
|------|--------|----------|
| nginx `client_max_body_size 1m` | 5 min | P1 |
| Daily creation cap per address (50/day) | 30 min | P1 |
| Message/evidence size limits | 15 min | P1 |
| Rate limit tuning (auth vs real requests) | 30 min | P2 |
| Offer expiry cron job | 1 hr | P2 |
| Escrow auto-timeout detection | 2 hr | P2 |
| Rate limit by endpoint tier | 2 hr | P3 |
| Admin API for moderation | 4 hr | P3 |

**Deliverable:** Indexer hardened against memory exhaustion, spam, and API abuse.

---

### Phase 2: Pre-Launch Verification (Week 3-4)

#### Security & Audit

**Approach:** Internal audit (28/30 items complete) + open-source transparency at launch. Bug bounty and/or third-party audit funded from treasury revenue after mainnet generates fees.

| Task | Effort | Priority |
|------|--------|----------|
| Publish audit report: `docs/security-audit.md` with all 28/30 items, the 7 fixed criticals, and the 2 remaining low-priority items | 1 hr | **P0** |
| Open GitHub Security Advisories for responsible disclosure | 30 min | **P0** |
| Verify all 28/30 audit items on mainnet | 1 day | P1 |
| Close remaining 2 items (H2 dust check, U7 onboarding) | 2 days | P2 |
| Penetration test on indexer API (self-run) | 2 days | P2 |
| **Set up treasury-funded security reserve:** Allocate XX% of protocol fees to bug bounty / audit fund | 1 hr | P1 |
| Launch bug bounty (Immunefi or custom) once treasury has meaningful funds | Ongoing | P3 |
| Third-party audit once treasury can cover it ($50-150K target) | Future | P4 |

#### Test Gates (All Must Pass)

| Gate | Status |
|------|--------|
| `cargo test --workspace` | ✅ 293 Rust tests pass |
| `cd web && npm test` | ✅ 44 Web tests pass |
| `cd bot && npm test` | ✅ 39 Bot tests pass |
| Manual: Web create → KasWare sign → broadcast → submit → settle → receipt | ❌ Needs wRPC |
| Manual: CLI `daglock-cli create` → `kaspawallet sign` → broadcast | ❌ Needs wRPC |
| Manual: Bot `/create` wizard → deep link → complete flow | ❌ Needs wRPC |
| Lifecycle integration tests (`cargo test --test lifecycle_tests`) | ✅ |
| Template hash verification on escrow create | ✅ A8 fixed |

#### Mainnet Covenant Hashes

Must recompute and register mainnet template hashes before launch:

| Covenant | Testnet Hash | Mainnet Hash (TBD) |
|----------|-------------|-------------------|
| KAS | `3502219e8c85ff1f4eb3c1f20ff1049518302d2c` | — |
| KRC-20 | `da57b7b66dd2f9a35dcb83a7fea3c05d1300c28e` | — |
| Reputation | `65c54102c64a331414b602760cbd76efac3d69df` | — |
| Vault (standard) | `23734973784f8d47adf0c0a43744955817258d1d` | — |
| Vault (softlock) | `9777c9eb9e6271a32fac75d3533bc27d25b20d39` | — |
| Vault (multisig) | `b0cddcd4dc716532fd86d1809a05f8ea7e74113d` | — |
| Subscription | (new) | — |
| Milestone | (new) | — |
| Advanced | (new) | — |

---

### Phase 3: Counterparty Discovery Board (Week 4)

**Why P1:** The Discord OTC channel was shut down. Reddit bans OTC posts. The community has **nowhere to find trading partners**. This is DagLock's biggest differentiator vs. Zilliqant.

| Task | Effort |
|------|--------|
| Offer listing API (already exists) | ✅ Done |
| Offer browsing UI on web dashboard | 2 days |
| Telegram `/offers` command with filters | 1 day |
| Offer expiry + auto-cleanup | 1 day |
| Email/Telegram notification on match | 2 days |
| KRC-20 token filter in offer board | 1 day |

**Deliverable:** Users can post "Wanted: 100K NACHO tokens for 5000 KAS" and find counterparties.

---

### Phase 4: Marketing & Community Rollout (Ongoing)

Based on market research — the Kaspa community lives on Telegram and Reddit. You're already active on Reddit and X. This phase formalizes and scales that presence.

#### Ongoing Presence (Already Active)

| Channel | Current Activity | Amplification |
|---------|-----------------|---------------|
| **r/kaspa** | ✅ Already posting | Increase frequency: dev updates, use cases, trade stories |
| **X (Twitter)** | ✅ Already posting | Thread format: covenant explainers, real trades, comparison vs CEX |
| **Kaspa Discord** | Not yet | Post in #projects, #general |
| **KRC-20 Telegram groups** | Not yet | Direct outreach to NACHO, other KRC-20 communities |
| **Kaspa Telegram groups** | Not yet | Share DagLock bot, demo |

#### Launch Day

| Action | Detail |
|--------|--------|
| Flip mainnet switch (indexer config → mainnet template hashes) | 5 min |
| Monitoring: dashboard, alerts, error rates | Continuous |
| Be in Telegram bot to respond to users | 24h coverage |
| Post launch thread on r/kaspa | Focus on: KRC-20 support, trustless, open-source, audit report |

#### Post-Launch (Weeks 2-4)

| Action | Detail |
|--------|--------|
| Track first 100 escrows manually | Validate each one |
| Bug-fix sprint based on real usage | Fast iteration |
| Reach out to KRC-20 projects for partnership | "DagLock is your escrow layer" |
| Apply for Kaspa Ecosystem Foundation grant | Funding |
| Publish case studies / trade receipts | Social proof |

---

### Phase 5: Feature Polish (Weeks 6-8)

Based on market research findings:

| Feature | Why | Effort |
|---------|-----|--------|
| **Onboarding modal** (U7) | First-visit users need guidance | 2 days |
| **KasWare signing flow polish** | Wallet signing is the UX bottleneck | 3 days |
| **Kaspium deeplink integration** | Mobile wallet support | 2 days |
| **Settlement receipt export (PDF)** | Proof for OTC desks | 2 days |
| **Volume-based fee tiers** | Whale retention | 3 days |
| **Bot mainnet auth fix** — /submit_sig now correctly forwards user signatures | Critical — broken on mainnet | ✅ Done |
| **Chat signature verification** (P4) | Security — needs wRPC first | 2 days |
| **Subscription covenant web UI** | Completeness | 2 days |
| **Milestone escrow web UI** | Completeness | 2 days |

---

### Phase 6: Cross-Chain HTLC (Phase 6+, Future)

**Not a launch blocker.** The cross-chain swap market is early (only PoCs exist). DagLock's covenant-based HTLC is a building block. Ship mainnet without it, then add when demand materializes.

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| **Zilliqant's Escrow captures mindshare** | High | High | Ship faster; differentiate via KRC-20, bot, reputation |
| **Covenant bug locks user funds** | Low | Critical | Thorough audit, circuit-breaker (timeout paths), insurance fund |
| **Indexer gets DDoSed** | Medium | High | Rate limiting, body limits, Cloudflare DDoS protection |
| **wRPC node goes down** | Medium | High | `--no-wrpc` fallback, monitoring, redundant node |
| **KRC-20 standard changes** | Medium | Medium | Covenant parameterized; update template hash |
| **KaspaCom adds escrow** | Medium | Medium | Differentiate via L1 covenant (not L2 AMM), build community lock-in |
| **Low initial adoption** | Medium | Medium | Target KRC-20 communities directly; offer first N trades free |
| **SilverScript compiler bug** | Low | High | Extensive execution tests (104 tests), conservative entrypoints |

---

## Rollback Plan

| Scenario | Action |
|----------|--------|
| Critical bug found in first 24h | Disable /escrows/create endpoint, post explanation, fix, re-enable |
| Covenant bug (funds at risk) | Publish emergency notice, guide users to refund via timeout paths |
| Indexer compromised | Revoke API keys, rotate secrets, restore from backup |
| wRPC node failure | Toggle `--no-wrpc` fallback (mock verification — degraded but alive) |
| Bot token leaked | Revoke in @BotFather, deploy new token, update env |

---

## Launch Checklist (Quick Reference)

### 🔴 Must Have (Launch Blocker)

- [ ] Local kaspad node synced with `--utxoindex`
- [ ] Indexer running with real wRPC verification
- [ ] wRPC fallback tested and documented
- [ ] All test gates pass (Rust + Web + Bot + Manual)
- [ ] Mainnet template hashes computed and deployed
- [ ] nginx body limit, rate limits, creation caps in place
- [ ] Secrets rotated (Cloudflare + DAGLOCK_MESSAGE_KEY)
- [ ] Cloudflare DDoS protection configured
- [ ] Monitoring + alerting set up
- [ ] Rollback plan documented and tested

### 🟡 Should Have (Week 1 Blocking)

- [ ] Third-party audit OR bug bounty live
- [ ] Counterparty discovery board live
- [ ] KRC-20 token filter in offer board
- [ ] Onboarding flow for first-time users
- [ ] Fee display in USD on create flow
- [ ] Error messages user-friendly (ApiErrorCode enum)

### 🟢 Nice to Have (Week 2+)

- [ ] Kaspium deeplinks
- [ ] Settlement receipt PDF export
- [ ] Volume-based fee tiers
- [ ] Email/Telegram trade notifications
- [ ] Casino mode (testnet tokens for demo)
- [ ] Analytics dashboard (trade volume, user growth)

---

## Success Metrics

| Metric | Target (30 days) | Target (90 days) |
|--------|-----------------|------------------|
| Escrows created | 100 | 1,000 |
| Total volume locked | 100K KAS | 1M KAS |
| Registered users (bot) | 500 | 5,000 |
| Settled trades | 50 | 500 |
| Unique trading pairs | 10 | 50 |
| Protocol revenue | 500 KAS | 5,000 KAS |
| Reputation entries | 100 | 1,000 |
| Reddit mentions | 5 | 20 |

---

## Comms Plan

### Positioning Statement

> **DagLock is the first trustless escrow protocol for Kaspa and KRC-20 tokens.** No admin keys, no custody, no CEX risk. Lock funds in a covenant — they're released only when the agreed terms are met. Built on Kaspa L1 covenants (Toccata). Backed by a formal security audit.

### Target Audiences

| Audience | Message | Channel |
|----------|---------|---------|
| **KRC-20 traders** | "Trade KRC-20 tokens safely — no DEX needed for your next whale deal" | Telegram, Reddit |
| **OTC desks** | "Settle large KAS trades without CEX counterparty risk" | Direct outreach |
| **Kaspa community** | "Real covenant use case, live today" | r/kaspa, Discord |
| **Crypto Twitter** | "What Kaspa covenants can actually do" | X thread |
| **Developers** | "Open-source, audited, embeddable via REST API" | GitHub, dev forums |

---

## Post-Launch: First 24h Runbook

| Time | Action |
|------|--------|
| T-1h | Final health check: indexer, bot, web, node |
| T-0 | Flip config to mainnet, restart indexer |
| T+5min | Smoke test: create escrow on web → sign → broadcast |
| T+15min | Monitor error rates, response times |
| T+1h | Post launch announcement on r/kaspa |
| T+2h | Monitor Telegram bot for user issues |
| T+24h | Post-launch review: what broke, what surprised us |

---

*Created: 2026-07-08. Based on market research across Reddit, X/Twitter, Kaspa community channels, DeFi ecosystem analysis, and OTC market data.*
