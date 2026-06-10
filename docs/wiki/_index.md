# DagLock — Architecture Overview

**Build**: `cargo build --workspace`  **Test**: `cargo test --workspace`  **Simulation**: `python3 scripts/simulation.py`

## Quick reference

### Key files
| Purpose | File |
|---------|------|
| KAS escrow covenant | `contracts/src/daglock.sil` |
| KRC-20 escrow covenant | `contracts/src/daglock_krc20.sil` |
| KAS arbiter covenant (mediator/jury) | `contracts/src/daglock_arbiter.sil` |
| Covenant compilation + template hash | `contracts/src/lib.rs` |
| Indexer entry | `indexer/src/main.rs` |
| REST API routes | `indexer/src/api/mod.rs` |
| DB schema + migrations | `indexer/src/db/schema.rs` |
| Authentication + replay protection | `indexer/src/auth.rs` |
| Rate limiter (30 req/min per IP) | `indexer/src/ratelimit.rs` |
| API key middleware | `indexer/src/api/apps.rs` |
| UTXO verification | `indexer/src/verification.rs` |
| Message encryption (AES-256-GCM) | `indexer/src/crypto.rs` |
| wRPC listener | `indexer/src/listener.rs` |
| CLI entry | `cli/src/main.rs` |
| WASM SDK | `wasm-sdk/src/lib.rs` |
| Web UI entry | `web/src/App.tsx` |
| Web components | `web/src/components/*.tsx` (8 files) |
| Web tests | `web/src/__tests__/*.test.tsx` (6 files) |
| Bot entry | `bot/src/index.js` |
| Reputation simulation | `scripts/simulation.py` |
| Dockerfile | `Dockerfile` |

### API modules (indexer/src/api/)
| Module | Purpose |
|--------|---------|
| `apps.rs` | Integrator app registration + API key management |
| `webhooks.rs` | Webhook dispatch for lifecycle events |
| `escrows.rs` | CRUD + lifecycle (create, settle, refund, dispute, cancel) |
| `offers.rs` | Counterparty discovery board |
| `reputation.rs` | On-chain derived reputation |
| `receipts.rs` | Settlement receipts |
| `network.rs` | Chain info + fee estimates |
| `evidence.rs` | Dispute evidence log |
| `identity.rs` | Telegram handle verification |
| `vouches.rs` | Web of Trust / vouching |
| `jury.rs` | Community jury system |
| `messages.rs` | Escrow-threaded chat (encrypted) |

### Test per domain
| Domain | Run |
|--------|-----|
| contracts | `cargo test -p daglock-contracts` |
| indexer | `cargo test -p daglock-indexer` — 39 unit + 12 integration |
| cli | `cargo test -p daglock-cli` |
| wasm-sdk | `cargo test -p daglock-wasm-sdk` |
| web | `cd web && npm test` |
| bot | `cd bot && npm test` — command handlers |
| simulation | `python3 scripts/simulation.py --trades 30 --bots 3` |

### Domain one-liners
| Domain | Doc |
|--------|-----|
| contracts | `features/contracts.md` — SilverScript covenants + compilation |
| indexer | `features/indexer.md` — Rust backend: REST API, DB, auth, crypto, verification |
| cli | `features/cli.md` — Command-line power-user tool |
| wasm-sdk | `features/wasm-sdk.md` — Browser/JS SDK for tx assembly |
| web | `features/web.md` — React + Vite dashboard |
| bot | `features/bot.md` — Telegram bot (Node.js) |
| simulation | `scripts/simulation.py` — Mass trade generation + reputation testing |

## Navigation

### For humans
| I want to... | Read |
|-------------|------|
| Know what NOT to do | `_standards.md` section Rules |
| Know how to write new code | `_standards.md` section Practices |
| Match existing conventions | `_standards.md` section Patterns |
| Understand a module | `features/<domain>.md` |

### For AI agents
| Task | Start here |
|------|------------|
| Cold start (zero context) | `_glossary.md` -> `_index.md` -> `_standards.md` Rules -> Practices -> target `features/<domain>.md` -> Patterns |
| Add a feature | `_standards.md` Rules -> Practices -> target `features/<domain>.md` -> Patterns |
| Fix a bug | `features/<domain>.md` (edge cases) -> `_standards.md` Rules |
| Refactor a module | `features/<domain>.md` (deps + consumers) -> `_standards.md` Patterns |
| Navigate unfamiliar code | Read domain one-liner -> open `features/<domain>.md` |
| Write a test | `features/<domain>.md` -> `_standards.md` Practices (Testing) |

## Entry points
| Trigger | File | Description |
|----------|------|-------------|
| `cargo run -p daglock-indexer` | `indexer/src/main.rs` | Start REST API + optional wRPC listener |
| `cargo run -p daglock-cli -- <cmd>` | `cli/src/main.rs` | CLI subcommand dispatch |
| `cd web && npm run dev` | `web/src/App.tsx` | Vite dev server |
| `cd bot && node src/index.js` | `bot/src/index.js` | Telegram bot |
| `docker run daglock/indexer` | `Dockerfile` | Production containerized deployment |
| `python3 scripts/simulation.py` | `scripts/simulation.py` | Generate trades + verify reputation |
| Pre-flight checklist | `docs/manual-verification-plan.md` | 16 checks before testnet/mainnet deploy |

## Topology
```
DagLock/
  contracts/          (SilverScript covenants + Rust compilation)
    src/daglock.sil                    -- KAS escrow covenant
    src/daglock_krc20.sil              -- KRC-20 escrow covenant
    src/daglock_arbiter.sil            -- Arbiter (mediator/jury)
    src/daglock_vault.sil              -- Standard time-locked vault
    src/daglock_vault_softlock.sil     -- Password-recoverable vault
    src/daglock_vault_multisig.sil     -- Multi-sig vault (up to 3-of-3)
    src/lib.rs                         -- Compilation + template hash
    tests/                             -- TxScriptEngine execution tests

  indexer/            (Rust backend)
    src/main.rs              -- api, config, crypto, db, listener, ratelimit
    src/config.rs            -- argparse + production flags (--allow-mainnet, --cors-origin)
    src/crypto.rs            -- AES-256-GCM message encryption
    src/auth.rs              -- Schnorr signature verification (real, not mock), nonce-based replay protection
    src/verification.rs      -- UTXO verification (WrpcVerifier / MockVerifier)
    src/ratelimit.rs         -- 30 req/min per-IP rate limiter
    src/websocket.rs         -- WebSocket broadcast
    src/types.rs             -- serde (all shared types)
    src/db/schema.rs         -- sqlx, migrations/ (8+ files)
    src/db/queries.rs        -- all SQL queries + vouch/mediator scoring
    src/api/                 -- 12 handler modules (apps, compile, escrows, evidence, identity, jury, messages, network, offers, receipts, reputation, status, swap, vaults, vouches, webhooks)
    src/services/webhooks.rs -- webhook dispatch with exponential backoff retry
    tests/                   -- integration + edge case tests

  cli/                (CLI tool)
    src/main.rs              -- clap, commands/*
    src/config.rs            -- ~/.daglock/config.toml
    src/tx.rs                -- kas_to_sompi, tx assembly
    src/commands/            -- 7 command files (create, claim, offer, status, rep, receipt, message)

  wasm-sdk/           (Browser SDK)
    src/lib.rs               -- wasm-bindgen, compile_escrow

  web/                (React dashboard + Vitest + Biome)
    src/App.tsx              -- main app (layout, routing, tab management)
    src/api.ts               -- REST API client + TypeScript types
    src/helpers.tsx          -- utilities (money, sompi, time, badge, errMsg)
    src/ui.tsx               -- reusable UI primitives (Panel, FormField, etc.)
    src/kasware.ts           -- KasWare wallet integration
    src/styles.css           -- dark-theme CSS
    src/components/          -- 8 domain component files (23 components)
    src/__tests__/           -- 6 test files (26 tests)
    biome.json               -- lint config
    vite.config.ts           -- build + vitest config

  bot/                (Telegram bot)
    src/index.js             -- grammY, lib/api.js

  scripts/            (Dev tooling)
    simulation.py            -- mass trade gen + reputation test
    deploy-testnet.sh        -- testnet deployment
    deploy-mainnet.sh        -- mainnet Docker deployment
    local-testnet.sh         -- local simnet kaspad

  Dockerfile          (Multi-stage build for production)
```

## Need to change X? Start here
| Change | Look at |
|--------|---------|
| Escrow spending rules | `contracts/src/daglock.sil` |
| KRC-20 token escrow | `contracts/src/daglock_krc20.sil` |
| Arbiter dispute paths | `contracts/src/daglock_arbiter.sil` |
| Covenant compilation API | `contracts/src/lib.rs` |
| Add REST endpoint | `indexer/src/api/mod.rs` (router) + new handler module |
| Add DB table/column | `indexer/src/db/schema.rs` (migration) + `indexer/src/db/queries.rs` |
| Change reputation formula | `indexer/src/db/queries.rs` (calculate_reputation_score) |
| Add CLI subcommand | `cli/src/main.rs` (enum) + new `cli/src/commands/*.rs` |
| Add web component | `web/src/components/<domain>.tsx` + add to `App.tsx` imports |
| Add web test | `web/src/__tests__/<Component>.test.tsx` (mock `../api` with `mockApi()`) |
| Add bot command | `bot/src/index.js` (command handler) |
| Change fee percentage | `contracts/src/daglock.sil` (hardcoded `inputValue / 200`) |
| Add vault type | Write new `.sil` file + add `compile_*()` in `contracts/src/lib.rs` |
| Register API key | `POST /v1/apps/register` |
| Add webhook | `POST /v1/apps/:id/webhooks` |
| SDK integration | `npm install @daglock/sdk` |
| Embed escrow widget | `<script src="https://unpkg.com/@daglock/widget"><daglock-escrow>` |
| Change template hash logic | `contracts/src/lib.rs` (`template_parts_and_hash()`) |
| Add authentication | `indexer/src/auth.rs` (SignatureVerifier trait) |
| Change rate limit | `indexer/src/ratelimit.rs` (RateLimiter struct, max_requests/window_secs) |
| Register API key | `POST /v1/apps/register` |
| Add webhook | `POST /v1/apps/:id/webhooks` |
| Add UTXO verification | `indexer/src/verification.rs` (EscrowVerifier trait) |
| Change message encryption | `indexer/src/crypto.rs` (AES-256-GCM, DAGLOCK_MESSAGE_KEY) |
| Change jury rules | `indexer/src/api/jury.rs` |
| Run reputation sim | `scripts/simulation.py` |

## Domain registry
| Domain | Doc | Files | Purpose |
|--------|-----|-------|---------|
| contracts | `features/contracts.md` | 8 | SilverScript covenants + compilation + tests |
| indexer | `features/indexer.md` | 27 | REST API, DB, auth, crypto, verification, jury, messages, webhooks, apps |
| cli | `features/cli.md` | 10 | Command-line power-user tool |
| wasm-sdk | `features/wasm-sdk.md` | 1 | Browser/JS SDK for tx assembly |
| web | `features/web.md` | 14 | React + Vite dashboard |
| bot | `features/bot.md` | 2 | Telegram bot (Node.js) |

## Existing docs
- README.md -- project overview, setup
- AGENTS.md -- project context for AI agents
- docs/API.md -- REST API reference
- docs/ARCHITECTURE.md -- system architecture
- docs/PROTOCOL.md -- covenant protocol spec
- docs/ROADMAP.md -- development roadmap
- docs/SECURITY.md -- threat model + audit checklist

---

## Audit Log

## Audit Log

### 2026-06-06 — Pre-Mainnet Security & Usability Audit

**Scope:** Full codebase — contracts, indexer, CLI, WASM SDK, web, bot

#### Fix Status (June 6, 2026) — 18 of 30 tasks completed

**Critical Security:** ✅ All 7 fixed | 🔴 0 remaining

| ID | Issue | Severity | Status |
|----|-------|----------|--------|
| **S1** | MockVerifier used in production — no real UTXO verification | CRITICAL | ✅ Fixed — async `WrpcVerifier` with `get_utxos_by_addresses()` | — no real UTXO verification | CRITICAL | ✅ Fixed — async `WrpcVerifier` with `get_utxos_by_addresses()` |
| **S2** | KRC-20 fee validation only boolean, not exact amount | HIGH | ✅ Fixed — exact `outputs[1].value == inputValue` + treasury script check |
| **S3** | KRC-20 KCC-20 output ownership validation | HIGH | ⏭️ Closed — multi-sig design prevents (both signers must agree with SIGHASH_ALL) |
| **S4** | trade_hash not validated on escrow creation | MEDIUM | ✅ Fixed — `daglock_shared::validate_trade_hash()` rejects malformed input |
| **S5** | No replay protection on signed messages | MEDIUM | ✅ Fixed — `action:id:ts:nonce` format, 5-min window, DB-backed nonce store (migration 014) |
| **S6** | Bot stores addresses in plaintext /tmp | MEDIUM | ✅ Fixed — AES-256-GCM encryption with `BOT_ENCRYPTION_KEY` env var |
| **S7** | Dockerfile runs as root | MEDIUM | ✅ Fixed — non-root `daglock` user |

**Usability:** 1 fixed | 6 remaining

| ID | Issue | Status |
|----|-------|--------|
| U1 | CLI create uses dummy keys, no wallet integration | ❌ Open |
| U2 | Web CreateEscrowForm generates fake lock_tx_id | ❌ Open |
| U3 | No wallet signing in CLI | ❌ Open |
| U4 | Bot /create redirects to web, no native flow | ❌ Open |
| U5 | Generic "internal error" for all 500s | ❌ Open |
| U6 | CoinGecko price fetch no fallback/caching | ❌ Open |
| U7 | No web onboarding for first-time users | ✅ Fixed — welcome modal on first visit |

**Structural:** 3 fixed | 5 remaining

| ID | Issue | Status |
|----|-------|--------|
| A1 | EscrowVerifier trait sync but wRPC is async | ✅ Fixed (done with S1) |
| A2 | Migration .ok() silences failures | ✅ Fixed — proper PRAGMA table_info checks |
| A3 | queries.rs is 1843-line god module | ❌ Open |
| A4 | No full lifecycle integration test | ❌ Open |
| A5 | Handlers mix HTTP + business logic + DB | ❌ Open |
| A6 | Bot is Node.js, rest is Rust | ❌ Open (out of scope for mainnet) |
| A7 | No OpenAPI spec | ❌ Open |
| A8 | No template hash verification on create | ✅ Fixed — checks against configured templates |

**Code Quality:** 3 fixed | 5 remaining

| ID | Issue | Status |
|----|-------|--------|
| Q1 | .unwrap() in production code | ❌ Open |
| Q2/Q3 | Magic number 200 scattered in 5+ locations | ⚠️ `FEE_DENOMINATOR` exists but not yet wired everywhere |
| Q4 | trade_hash handling inconsistent | ✅ Fixed — `daglock_shared::validate_trade_hash()` |
| Q5 | No structured request tracing | ❌ Open |
| Q6 | Config validation gaps | ❌ Open |
| Q7 | Web API no request timeout | ✅ Fixed — 30s AbortController on all fetch calls |
| Q8 | Bot API no retry/backoff | ✅ Fixed — 3 attempts, 1s/2s/4s exponential backoff |
| Q8 | Bot API no retry/backoff | ✅ Fixed — 3 attempts, 1s/2s/4s exponential backoff |

#### Key Files Changed (Phase 1 + quick wins)

| File | Change |
|------|--------|
| `shared/` (4 new files) | Constants + validation crate with 20 tests |
| `contracts/src/daglock_krc20.sil` | Exact fee enforcement in release/swap paths |
| `contracts/tests/daglock_krc20_execution_tests.rs` | Wrong-fee rejection test added |
| `indexer/src/verification.rs` | Async `WrpcVerifier` with real `get_utxos_by_addresses()` |
| `indexer/src/auth.rs` | Replay protection: `action:id:ts:nonce`, nonce DB, backward compat |
| `indexer/src/api/escrows.rs` | `.await` on all verification calls, trade_hash validation, template hash validation |
| `indexer/src/db/queries.rs` | `store_auth_nonce()`, `check_auth_nonce_exists()` |
| `indexer/src/db/schema.rs` | Migration 014 (auth_nonces), migration idempotency fixes |
| `bot/src/index.js` | AES-256-GCM encryption for user address storage |
| `bot/src/lib/api.js` | 3-attempt retry with exponential backoff + 10s timeout |
| `web/src/api.ts` | 30s AbortController timeout on all requests |
| `web/src/App.tsx` | First-visit onboarding modal |
| `Dockerfile` | Non-root `daglock` user |

#### Rules Compliance (updated)

| Rule | Status |
|------|--------|
| #1: Never `.unwrap()` outside tests | ⚠️ Still violated (tracked as Q1) |
| #2: Never hardcode addresses/keys in covenant | ✅ Compliant |
| #3: Never skip fee validation in release/swap | ✅ Fixed |
| #4: Never expose private keys | ✅ Compliant |
| #5: Never change fee denominator without updating all paths | ⚠️ `FEE_DENOMINATOR` exists but not fully wired |
| #6: Never use non-atomic updates for lifecycle | ✅ Compliant |
| #7: Never skip address validation on create | ✅ Compliant |

#### Remaining Work (12 tasks, targeting June 30)

| Phase | Priority | Tasks | Effort |
|-------|----------|-------|--------|
| Phase 2 — Usability | High (user-facing) | U1-U6 | ~3-4 days |
| Phase 3 — Structural | High (tech debt) | A3-A7 | ~3-4 days |
| Phase 4 — Polish | Medium | Q1-Q6 | ~1-2 days |

**Next priority:** Split `queries.rs` (A3), lifecycle integration tests (A4), CLI wallet integration (U1/U3).

