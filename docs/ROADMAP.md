# DagLock Development Roadmap

> Updated June 6, 2026. Post-audit revision — critical security and usability issues identified.

**Toccata Mainnet Hard Fork:** Activates at DAA score 474,165,565 (~June 30, 16:15 UTC).
**DagLock mainnet launch target:** June 30, 2026 (same day as Toccata).

---

## Current State

Everything in Phases 0-3 is **built, tested, and running on Testnet 12**. However, a comprehensive audit on June 6 identified **7 critical/high security issues** and **7 high-priority usability issues** that must be resolved before mainnet launch. The audit also revealed structural debt that should be addressed.

**The code is NOT production-ready until Phase 1 (Critical Security) of the audit fix plan is complete.**

---

## Phase 0: Covenants  (Done)

| Task | Status |
|---|---|
| `daglock.sil` — KAS escrow (release, swap, refund) |  Done |
| `daglock_krc20.sil` — KRC-20 escrow via ICC pattern |  Done |
| `daglock_arbiter.sil` — KAS escrow with mediator/jury paths |  Done |
| `daglock_vault.sil` — time-locked self-custody vault |  Done |
| `daglock_vault_softlock.sil` — password-recoverable vault with beneficiary |  Done |
| `daglock_vault_multisig.sil` — multi-sig vault (up to 3-of-3) |  Done |
| Compilation against `silverc` |  Done |
| TxScriptEngine unit tests — all paths + negatives |  Done (7+ per covenant) |
| Script mass benchmarking |  Done |
| Template hash extraction |  Done |
| Published open-source |  github.com/dilljens/DagLock |

**Remaining (from audit):**
- [ ] **S2: KRC-20 exact fee enforcement in covenant** — currently only boolean check
- [ ] **S3: KRC-20 KCC-20 output ownership validation** — verify seller receives tokens
- [ ] **Template hash regeneration** after covenant changes

---

## Phase 1: Indexer + Counterparty Discovery  (Mostly Done)

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
| wRPC listener |  **Stub** — detects nothing, logs only. **S1: Must implement real UTXO verification** |

**Critical Remaining (from audit):**
- [ ] **S1: Real async WrpcVerifier** — implement `get_utxos_by_outpoints` via wRPC
- [ ] **S4: Validate trade_hash on escrow creation** (64 hex chars)
- [ ] **S5: Replay protection for signed messages** (timestamp + nonce)
- [ ] **A2: Fix migration .ok() silent failures** — use PRAGMA table_info checks
- [ ] **A8: Template hash verification on create** — reject unknown templates
- [ ] **A7: OpenAPI spec** — add utoipa annotations
- [ ] **Q5: Structured request tracing** — request_id, user_address, escrow_id spans

---

## Phase 2: Telegram Bot + CLI  (Done but with gaps)

| Task | Status |
|---|---|
| Telegram bot — `/create`, `/claim`, `/offers`, `/list`, `/reputation`, `/receipt`, `/dispute`, `/msg`, `/messages`, `/status` |  Done, running |
| CLI tool — `daglock-cli` with 7 command modules |  Done |
| Trade link deep links (`t.me/DagLock_bot?start=claim_abc123`) |  Done |
| KasWare wallet bridge |  Not implemented |

**Critical Remaining (from audit):**
- [ ] **U1: CLI create uses real wallet keys** — integrate `kaspawallet sign` subprocess
- [ ] **U3: CLI wallet integration module** — shared signing logic for all commands
- [ ] **U4: Bot native /create wizard** — grammY conversation flow
- [ ] **S6: Bot encrypt user addresses at rest** — libsodium with BOT_ENCRYPTION_KEY
- [ ] **Q8: Bot API retry/backoff** — 3 attempts, exponential backoff

---

## Phase 3: Web UI  (Done but with gaps)

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
| Atomic swap wizard |  **Deferred** — covenant supports it, no guided UI |

**Critical Remaining (from audit):**
- [ ] **U2: Web CreateEscrowForm uses real lock_tx_id** — WASM compile → KasWare sign → broadcast → submit tx_id
- [ ] **U7: Web onboarding modal** — first-visit: "Need KasWare + testnet KAS + connect wallet"
- [ ] **Q7: Web API request timeout** — AbortController 30s timeout
- [ ] **Q2/Q3: Use shared fee constant** — replace all `200` with `FEE_DENOMINATOR`

---

## Phase 4: Audit Fixes — Critical Security (June 6 — June 20)

**Goal:** Resolve all CRITICAL/HIGH security findings before mainnet.

| Task | Status | Owner |
|---|---|---|
| **0. Shared crate** — `FEE_DENOMINATOR` constant + `validate_trade_hash` |  Not started | |
| **1. S1: Real async WrpcVerifier** — `get_utxos_by_outpoints` |  Not started | |
| **2. S2: KRC-20 exact fee enforcement** — covenant change |  Not started | |
| **3. S3: KRC-20 KCC-20 output ownership validation** |  Not started | |
| **4. S2/S3 execution tests** — TxScriptEngine for new paths |  Not started | |
| **5. S4: Validate trade_hash on create** |  Not started | |
| **6. S5: Replay protection** — timestamp + nonce in auth messages |  Not started | |
| **7. S6: Bot encrypt user addresses** — libsodium |  Not started | |
| **8. S7: Dockerfile non-root user** |  Not started | |

**Dependencies:** Task 1 blocks 2, 3, 6. Task 2 blocks 3, 4. Task 0 feeds into 2, 3, 5.

---

## Phase 5: Audit Fixes — Usability & Structure (June 15 — June 28)

**Goal:** Make CLI/web/bot actually usable for real escrow creation; pay down structural debt.

| Task | Status | Phase |
|---|---|---|
| **9. U1: CLI real wallet keys** — `kaspawallet sign` subprocess |  Not started | 2 |
| **10. U2: Web real lock_tx_id flow** — WASM → KasWare → broadcast → submit |  Not started | 2 |
| **11. U3: CLI wallet module** — shared signing logic |  Not started | 2 |
| **12. U4: Bot native /create wizard** — grammY conversations |  Not started | 2 |
| **13. U5: Structured API errors** — ApiErrorCode enum |  Not started | 2 |
| **14. U6: CoinGecko fallback + caching** — TTL 15min, stale flag |  Not started | 2 |
| **15. U7: Web onboarding modal** |  Not started | 2 |
| **16. A1: Async verifier** (done in S1) |  Done in S1 | 1 |
| **17. A2: Migration idempotency** — PRAGMA checks |  Not started | 3 |
| **18. A3: Split queries.rs** — 11 modules, incremental |  Not started | 3 |
| **19. A4: Lifecycle integration tests** — create→lock→settle→receipt |  Not started | 3 |
| **20. A5: Service layer** — EscrowService |  Not started | 3 |
| **21. A7: OpenAPI spec** — utoipa + /v1/openapi.json |  Not started | 3 |
| **22. A8: Template hash verification** on create |  Not started | 1 |
| **23. Q1: Remove .unwrap() in production** |  Not started | 4 |
| **24. Q2/Q3: Shared fee constant everywhere** |  Not started | 4 |
| **25. Q4: TradeHash newtype** with FromStr |  Not started | 4 |
| **26. Q5: Request tracing** (done with A7) |  Not started | 3 |
| **27. Q7: Web API timeout** |  Not started | 4 |
| **28. Q8: Bot retry/backoff** |  Not started | 4 |

---

## Phase 6: Mainnet Launch (June 28 — June 30)

| Task | Status |
|---|---|
| Deploy mainnet indexer on Railway (staging, dormant) | ~June 28 |
| Final testnet validation with all fixes | June 28-29 |
| Swap daglock.com to mainnet API | June 30 |
| Deploy @DagLock_bot (mainnet) | June 30 |
| Announce mainnet launch | June 30 |
| Monitoring — health checks, error alerting | Basic |

---

## Phase 7: Post-Launch (Q3 2026+)

| Candidate | Why it might matter |
|---|---|
| Atomic swap wizard UI | Makes cross-token swaps accessible |
| Price oracle (CoinGecko KAS/USD) | Lock fiat-equivalent rate at creation |
| Batch escrow / multi-asset swaps | Complex OTC trades |
| Integrator API | Let other dApps embed DagLock |
| Analytics dashboard | Public volume, fees, stats |
| Volume-based fee rebates | Only if a whale asks |
| Cross-chain (BTC) | Standard HTLC pattern, ~2-3 weeks work |
| KasWare bridge for bot | Sign from browser instead of manual sigs |

---

## Audit Reference

Full audit findings and 30-task fix plan: `.pi/last-plan.md` and `docs/wiki/_index.md#audit-log`
