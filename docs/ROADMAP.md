# DagLock Development Roadmap

> Updated June 5, 2026. The original plan was written during Phases 0-3. This version reflects what actually shipped and what's left.

**Toccata Mainnet Hard Fork:** Activates at DAA score 474,165,565 (~June 30, 16:15 UTC).
**DagLock mainnet launch:** Same day.

---

## Current State

Everything in Phases 0-3 is **built, tested, and running on Testnet 12**. The code is production-ready. The remaining work before mainnet is:

1. wRPC listener — auto-detect on-chain escrows instead of manual API calls
2. Testnet feedback — let people break it for 3.5 weeks, fix what they find
3. Deploy — flip the switch when Toccata activates

---

## Phase 0: Covenants  (Done)

| Task | Status |
|---|---|
| `daglock.sil` — KAS escrow (release, swap, refund) |  Done |
| `daglock_krc20.sil` — KRC-20 escrow via ICC pattern |  Done |
| `daglock_arbiter.sil` — KAS escrow with mediator/jury paths |  Done |
| `daglock_vault.sil` — time-locked self-custody vault |  Done |
| Compilation against `silverc` |  Done |
| TxScriptEngine unit tests — all paths + negatives |  Done (7+ per covenant) |
| Script mass benchmarking |  Done |
| Template hash extraction |  Done |
| Published open-source |  github.com/dilljens/DagLock |

**No remaining work in this phase.**

---

## Phase 1: Indexer + Counterparty Discovery  (Done)

| Task | Status |
|---|---|
| REST API v1 — 30+ endpoints |  Done |
| SQLite schema — escrows, offers, reputation, jury, messages, etc. |  Done |
| Offer board — create, list, accept, cancel offers |  Done |
| Reputation system — Beta formula, recency weighting, wash trading signal |  Done |
| Settlement receipts — BLAKE2b-hashed verifiable receipts |  Done |
| Encrypted messaging — AES-256-GCM per-escrow threads |  Done |
| Jury system — random juror selection, voting, case lifecycle |  Done |
| Evidence logging — signed proof during disputes |  Done |
| Vouching / Web of Trust |  Done |
| Identity linking — Telegram handle to Kaspa address |  Done |
| Covenant compiler API — compile templates via REST |  Done |
| wRPC listener |  **Stub** — detects nothing, logs only. Needs implementation before mainnet. |

**Remaining:**
- [ ] **wRPC listener** — connect to Kaspa node, auto-detect DagLock UTXOs by template hash, update escrow state. This is the single most important missing piece.

---

## Phase 2: Telegram Bot + CLI  (Done)

| Task | Status |
|---|---|
| Telegram bot — `/create`, `/claim`, `/offers`, `/list`, `/reputation`, `/receipt`, `/dispute`, `/msg`, `/messages`, `/status` |  Done, running |
| CLI tool — `daglock-cli` with 7 command modules |  Done |
| Trade link deep links (`t.me/DagLock_bot?start=claim_abc123`) |  Done |
| KasWare wallet bridge |  Not critical for launch. Bot works offline with signatures supplied manually. |

**Remaining:** Nothing blocking. KasWare bridge is nice-to-have.

---

## Phase 3: Web UI  (Done)

| Task | Status |
|---|---|
| Create escrow page |  Done |
| Claim page |  Done |
| Dashboard — escrows by status/token |  Done |
| Offer board with filters |  Done |
| Reputation view inline on trade cards |  Done |
| Settlement receipts (JSON) |  Done |
| Jury registration + voting UI |  Done |
| Encrypted messaging UI |  Done |
| Vault creation UI |  Done |
| Telegram linking UI |  Done |
| Evidence submission UI |  Done |
| Atomic swap wizard |  **Deferred** — the `swap` entrypoint works in the covenant, but there's no guided UI. Users can do atomic swaps manually with the hash preimage flow. A polished wizard would be nice but isn't blocking launch. |

**Remaining:** Nothing blocking.

---

## Phase 4: Testnet Feedback (June 5 — June 30)

**Goal:** Let real users break DagLock on Testnet 12 before mainnet.

| Task | Status |
|---|---|
| Deploy testnet infra (indexer, web, bot) |  This week |
| Post on Reddit / Telegram with test wallet |  This week |
| Fix bugs from user feedback | — |
| Community audit period | — |

**Validation:**
- 5+ external users complete full escrow flow on TN12
- Zero covenant vulnerabilities found during testing
- Feedback collected and prioritized

**Duration:** ~3.5 weeks

---

## Phase 5: Mainnet Launch (June 30)

**Goal:** Production deployment on Kaspa mainnet, same day as Toccata activates.

| Task | Status |
|---|---|
| Deploy mainnet indexer on Railway |  ~June 28 (staging, dormant) |
| Swap daglock.com to mainnet API |  June 30 |
| Deploy @DagLock_bot (mainnet) |  June 30 |
| Announce mainnet launch |  June 30 |
| Monitoring — health checks, error alerting |  Basic |
| wRPC listener connecting to a Kaspa mainnet node |  Needs implementation first |
| Documentation site `docs.daglock.io` |  **Deferred** — README + wiki covers it for now |

**What is NOT planned for mainnet launch:**
-  Volume-based fee tier rebates — not worth building until someone asks for a discount
-  Atomic swap wizard UI — the covenant supports it, no guided UI yet
-  Batch escrow UI — nice for whales, not needed at zero users
-  Price oracle — Phase 6 territory

---

## Phase 6: Post-Launch (Q3 2026+)

**Goal:** Build features that real users actually ask for. Nothing on this list is committed — it depends on what the community needs.

| Candidate | Why it might matter |
|---|---|
| **wRPC listener** (if not done by launch) | Auto-detect on-chain state |
| **Atomic swap wizard UI** | Makes cross-token swaps accessible to non-technical users |
| **Price oracle** (CoinGecko KAS/USD) | Lock in fiat-equivalent rate at escrow creation |
| **Batch escrow / multi-asset swaps** | "Swap 5000 KAS + 100K NACHO for 200K KASPY" |
| **Integrator API** | Let other dApps embed DagLock |
| **Analytics dashboard** | Public volume, fees, stats |
| **Volume-based fee rebates** | Only if a whale asks |
| **Cross-chain (BTC)** | If users want it — standard HTLC pattern, ~2-3 weeks work |
| **KasWare bridge for bot** | Sign from browser instead of manual sigs |

---

## Things That Were Cut (from original plan)

| Cut | Reason |
|---|---|
| Volume-based fee tiers | Zero users = zero need. Covenant stays at 0.5%. If whales show up later, rebates can be added. |
| Atomic swap wizard UI | The swap entrypoint works. The guided UI is polish, not necessity. |
| Batch escrow UI | For whales that don't exist yet. |
| Price oracle | Phase 0-3 shipped without it. No user has asked. |
| Mobile SDK | Telegram bot covers mobile. |
| Bug bounty with dollar amounts | No treasury to fund it. Community audit is sufficient pre-launch. |
| Docker/Prometheus/PagerDuty monitoring | Ship first, monitor when there are users. |
