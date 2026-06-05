# DagLock Development Roadmap

> Market-informed delivery timeline. Prioritized by what Kaspa users actually need — based on OTC safety research, KRC-20 community analysis, and Telegram-bot-dominant ecosystem reality.

**Toccata Mainnet Hard Fork:** June 5–20, 2026.

---

## Research-Driven Priority Shifts

| Finding | Impact on Plan |
|---|---|
| KSPR Bot dominates KRC-20 minting/trading — Telegram IS the Kaspa UI | Must ship Telegram bot alongside web UI |
| KRC-20 tokens (Nacho, Kaspy) have 20K+ active holders trading P2P via risky middlemen | KRC-20 support moves from Phase 7 → Phase 1. Without it, zero users at launch. |
| OTC traders demand: multi-sig escrow, transparent fees, counterparty verification, settlement receipts | Adds: reputation system, settlement receipts, price oracle, proposal-before-commit flow |
| Kaspa community is Telegram-native, not web-native | Telegram bot is a launch requirement, not a nice-to-have |
| $50M+ OTC scams via fake middlemen in 2024-2025 | DagLock's trustless covenant directly solves this — the marketing message writes itself |

---

## Phase 0: Covenant + Compiler (Week 1)

**Goal:** KAS and KRC-20 covenants that compile and pass all tests.

| Task | Deliverable |
|---|---|
| Write `daglock.sil` (KAS escrow) | 3 entrypoints: `release`, `swap`, `refund` |
| Write `daglock_krc20.sil` (KRC-20 escrow) | Token-aware covenant with KRC-20 balance checks |
| Compile both against `silverc` | Verified bytecode output |
| Debugger unit tests — all paths + negatives | 7+ tests per covenant |
| Script mass benchmarking | Verify both fit within Kaspa limits |
| Template hash extraction | Deterministic fingerprint for indexer |
| Publish open-source on GitHub | Audit invitation to Kaspa core devs |

**Validation:**
- `silverc` compiles both `.sil` files without errors
- All spending paths pass execution tests in `TxScriptEngine`
- Script mass < `MAX_SCRIPT_PUBLIC_KEY_MASS`
- Template hashes are deterministic and documented

**Duration:** 5–7 days

---

## Phase 1: Indexer + Counterparty Discovery (Week 2)

**Goal:** Real-time escrow tracking plus a way for users to find each other.

| Task | Deliverable |
|---|---|
| wRPC listener | Subscribe to BlockDAG, detect DagLock UTXOs (KAS + KRC-20 template hashes) |
| PostgreSQL/SQLite schema | `escrows` table: lifecycle states, amounts, participants, token type |
| REST API v1 | `GET /escrows/:id`, `GET /escrows?address=...`, `POST /escrows` |
| **Counterparty discovery board** | `GET /offers` — public listing of open escrow offers with filters (token, amount range, expiration) |
| **Offer creation** | `POST /offers` — "I want to buy/sell X KAS for Y KRC-20. Escrow not yet funded." Proposal stage, no funds locked. |
| On-chain reputation endpoints | `GET /reputation/:address` — trade count, volume, dispute rate, account age |

**Validation:**
- Indexer detects both KAS and KRC-20 DagLock UTXOs on TN12
- Discovery board shows offers; counterparty can accept one
- Reputation endpoint returns stats for known addresses

**Duration:** 5–7 days

---

## Phase 2: Telegram Bot + CLI (Week 2–3 overlap)

**Goal:** Meet Kaspa users where they are — Telegram — and provide power-user CLI access.

| Task | Deliverable |
|---|---|
| **DagLock Telegram Bot** | `/create` — initiate escrow from chat\`/claim <id>` — claim an escrow\`/offers` — browse open offers\`/reputation <address>` — check counterparty\`/receipt <id>` — export settlement receipt |
| CLI tool | `daglock-cli create --amount 5000 --counterparty kaspa:...`\`daglock-cli claim <escrow-id>`\`daglock-cli offers --token KRC20:NACHO` |
| Trade link deep links | `https://t.me/DagLock_bot?start=claim_abc123` — open Telegram directly to claim |
| KasWare wallet bridge | Bot sends unsigned tx to KasWare for signing (no private key exposure) |

**Validation:**
- Full create → share link → claim flow works entirely within Telegram
- Bot never sees private keys
- CLI produces identical transactions to bot for same parameters

**Duration:** 3–5 days (parallel with Phase 1)

---

## Phase 3: Web UI (Week 3)

**Goal:** Browser dashboard for desktop users and the premium OTC experience.

| Task | Deliverable |
|---|---|
| Create Escrow page | KAS and KRC-20 forms, proposal-or-commit toggle, KasWare sign integration |
| Claim page | Trade link detection, terms display, one-click claim |
| Dashboard | All escrows for connected wallet, filter by status/token |
| **Offer Board** | Browse open offers, filter by token/amount, accept with one click |
| **Reputation view** | Counterparty stats inline on every trade card |
| **Settlement receipts** | Downloadable, shareable PDF/JSON proof of completed trade |
| Atomic swap wizard | Guided flow: pick tokens → generate secret → lock → counterparty locks → reveal |

**Validation:**
- Full create → share → claim flow works on TN12 via browser
- Offer board shows real testnet escrows
- Receipt exports include all on-chain verification data

**Duration:** 5–7 days

---

## Phase 4: KRC-20 Community Launch (Week 4)

**Goal:** Alpha launch targeting KRC-20 communities with real testnet tokens.

| Task | Deliverable |
|---|---|
| Target Nacho the Kat community | DM admins, offer co-branded escrow portal |
| Target Kaspy community | Same — largest active trading groups |
| Testnet faucet for KRC-20 tokens | Deploy test tokens for community testing |
| Community audit period | Open bug bounty for covenant logic (5 days) |
| Feedback loop | Collect feature requests, bug reports in Discord/Telegram |
| **Security audit** | Internal review of both covenants + indexer API |

**Validation:**
- 5+ external users complete full escrow flow on TN12
- Zero covenant vulnerabilities found during audit period
- Community feedback collected and prioritized

**Duration:** 5–7 days

---

## Phase 5: Mainnet Launch (Week 5+, Post-Toccata)

**Goal:** Production deployment on Kaspa mainnet.

| Task | Deliverable |
|---|---|
| Deploy indexer on mainnet | Swap TN12 for mainnet wRPC |
| Deploy Telegram bot on mainnet | `@DagLock_bot` live |
| Deploy web UI | `daglock.io` |
| Publish audit report | Public covenant audit |
| **Volume-based fee tiers** | Track off-chain volume per address; offer rebates at thresholds (e.g., >100K KAS vol → 0.25% fee) |
| **Batch escrow UI** | Group multiple UTXOs into one "deal" for whale risk management |
| Monitoring | Basic health checks, error alerting |
| **Documentation site** | `docs.daglock.io` — protocol spec, API reference, integration guide |

**Validation:**
- Full escrow flow works on mainnet with real KAS and KRC-20
- Treasury address collects fees
- Indexer handles mainnet block rate (10 BPS → 100 BPS)
- All systems operational 72+ hours

---

## Phase 6: Premium Features (Q3 2026)

**Goal:** Differentiators that attract whales and institutional users.

| Task | Deliverable |
|---|---|
| **Price oracle integration** | CoinGecko KAS/USD feed at escrow creation; both parties lock in rate |
| **Multi-asset batch swaps** | Single UI for "swap 5000 KAS + 100K NACHO for 200K KASPY" — coordinated multi-UTXO settlement |
| **Fiat escrow connector** | Allow parties to record fiat leg off-chain with signed attestation |
| **Advanced reputation system** | Weighted scores, dispute history, on-chain verification of trade completion |
| **API for integrators** | REST + WebSocket API for other dApps to embed DagLock escrow |
| **Analytics dashboard** | Public stats: total volume, active escrows, fee revenue, top tokens |

---

## Summary Timeline

```
Week 1  ████████░░░░░░░░░░░░░░░░░░░░  Phase 0: Covenants (KAS + KRC-20)
Week 2  ░░░░░░░░████████░░░░░░░░░░░░  Phase 1: Indexer + Discovery
Week 2  ░░░░░░░░░░░░░░████░░░░░░░░░░  Phase 2: Telegram Bot + CLI (parallel)
Week 3  ░░░░░░░░░░░░░░░░░░██████░░░░  Phase 3: Web UI
Week 4  ░░░░░░░░░░░░░░░░░░░░░░░░████  Phase 4: KRC-20 Community Launch
Week 5  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░  Phase 5: Mainnet (post-Toccata)
         ▲                                                      ▲
    Start here                                          Toccata Mainnet
                                                         (Jun 5–20)
```

---

## What Was Cut vs. Previous Plan

| Cut | Reason |
|---|---|
| WASM SDK as separate crate | Absorbed into CLI + Telegram bot + web UI — shared tx assembly code, not a published NPM package |
| OTC Desk (premium UI) | Merged into Web UI as Offer Board — same functionality, less ceremony |
| Docker / Prometheus / PagerDuty | Deferred to post-mainnet. Ship the binary first, monitor when there are users. |
| Mobile SDK (React Native / Flutter) | Telegram bot IS the mobile experience. Kaspa users don't download dApp-specific apps. |
| Bug bounty with dollar amounts | Replaced with community audit. Bounties require a treasury that doesn't exist yet. |

## What Was Added vs. Previous Plan

| Added | Reason |
|---|---|
| KRC-20 covenant at Phase 0 | Without KRC-20, no users. KAS-only P2P volume is negligible. |
| Counterparty discovery board | Escrow is useless without counterparties. Users need to find each other. |
| Telegram bot at Phase 2 | Kaspa users live on Telegram. KSPR Bot proved this model. |
| Reputation system | OTC traders demand counterparty verification. Derivable from on-chain data. |
| Settlement receipts | Institutional requirement. Proof of delivery for compliance/accounting. |
| Proposal-before-commit flow | Users want to negotiate terms before locking funds. Reduces friction. |
| Volume-based fee tiers | Whales negotiate. Off-chain rebates preserve covenant simplicity. |
| Price oracle integration | Eliminates manual price negotiation for fiat-anchored trades. |
