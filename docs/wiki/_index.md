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
| DB queries (11 modules) | `indexer/src/db/queries/` |
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
| `compile.rs` | Covenant compilation (dev utility) |
| `escrows.rs` | CRUD + lifecycle (create, settle, refund, dispute, cancel) |
| `evidence.rs` | Dispute evidence log |
| `identity.rs` | Telegram handle verification |
| `jury.rs` | Community jury system |
| `messages.rs` | Escrow-threaded chat (encrypted) |
| `network.rs` | Chain info + fee estimates |
| `offers.rs` | Counterparty discovery board |
| `receipts.rs` | Settlement receipts |
| `reputation.rs` | On-chain derived reputation |
| `status.rs` | Platform status + uptime |
| `swap.rs` | Atomic swap hash preimage generation |
| `vaults.rs` | Vault CRUD + lifecycle |
| `vouches.rs` | Web of Trust / vouching |
| `webhooks.rs` | Webhook dispatch for lifecycle events |

### Test per domain
| Domain | Run | Notes |
|--------|-----|-------|
| shared | `cargo test -p daglock-shared` | 20 tests (constants + validation) |
| contracts | `cargo test -p daglock-contracts` | 5 test files — TxScriptEngine execution tests |
| indexer | `cargo test -p daglock-indexer` | 3 test files — unit + lifecycle + edge cases |
| cli | `cargo test -p daglock-cli` | Config + arg parsing |
| wasm-sdk | `cargo test -p daglock-wasm-sdk` | Native (not wasm) compilation tests |
| web | `cd web && npm test` | 36 tests across 9 files (Vitest + RTL) |
| bot | `cd bot && npm test` | API client + command handlers |
| simulation | `python3 scripts/simulation.py --trades 30 --bots 3` | Mass trade generation + reputation testing |

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
| Browser | `https://daglock.com` | Web UI (Cloudflare Pages, free tier) |
| API | `https://api.daglock.com` | REST API (VPS, proxied via Cloudflare) |
| Telegram | `@DagLock_bot` | Bot on Hetzner VPS |
| `cargo run -p daglock-indexer` | `indexer/src/main.rs` | Start REST API + optional wRPC listener |
| `cargo run -p daglock-cli -- <cmd>` | `cli/src/main.rs` | CLI subcommand dispatch |
| `cd web && npm run dev` | `web/src/App.tsx` | Vite dev server |
| `cd bot && node src/index.js` | `bot/src/index.js` | Telegram bot |
| `python3 scripts/simulation.py` | `scripts/simulation.py` | Generate trades + verify reputation |

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

   shared/             (Shared Rust crate)
    src/constants.rs          -- FEE_DENOMINATOR = 200
    src/validation.rs         -- validate_trade_hash, validate_kaspa_address, kas_to_sompi
    src/lib.rs                -- re-exports

   indexer/            (Rust backend)
    src/main.rs               -- api, config, crypto, db, listener, ratelimit
    src/config.rs             -- argparse + production flags, Config::validate()
    src/crypto.rs             -- AES-256-GCM message encryption
    src/auth.rs               -- Schnorr signature verification, nonce-based replay protection
    src/verification.rs       -- UTXO verification (WrpcVerifier / MockVerifier)
    src/ratelimit.rs          -- 30 req/min per-IP rate limiter
    src/listener.rs           -- DAA polling, CoinGecko price, UTXO scanning
    src/websocket.rs          -- WebSocket broadcast
    src/types.rs              -- serde (all shared types)
    src/lib.rs                -- library re-exports
    src/db/schema.rs          -- sqlx, migrations/ (16 migrations)
    src/db/queries/           -- 12 domain query modules
    src/api/                  -- 16 handler modules (apps, compile, escrows, evidence, identity, jury, messages, network, offers, receipts, reputation, status, swap, vaults, vouches, webhooks)
    src/services/             -- business logic layer
      escrow_service.rs       -- EscrowService: create/settle/refund/dispute/cancel/atomic_swap
      webhooks.rs             -- webhook dispatch with exponential backoff retry
    tests/                    -- 3 test files (api, edge_cases, lifecycle)

  cli/                (CLI tool)
    src/main.rs               -- clap, commands/*
    src/config.rs             -- ~/.daglock/config.toml
    src/tx.rs                 -- kas_to_sompi, tx assembly
    src/wallet.rs             -- sign_with_kaspawallet, parse_hex_key
    src/commands/             -- 10 command files (create, claim, offer, status, reputation, receipt, message, swap, vault, evidence)

  wasm-sdk/           (Browser SDK)
    src/lib.rs                -- wasm-bindgen, compile_escrow, kas_to_sompi, validate_trade_hash

  web/                (React dashboard + Vitest + Biome)
    src/App.tsx               -- main app (layout, routing, tab management)
    src/router.tsx            -- hash-based client-side router
    src/api.ts                -- REST API client + TypeScript types
    src/helpers.tsx           -- utilities (money, sompi, time, badge, errMsg)
    src/ui.tsx                -- reusable UI primitives (Panel, FormField, etc.)
    src/kasware.ts            -- KasWare wallet integration
    src/styles.css            -- dark Kaspa green theme
    src/context/              -- WalletContext (wallet connection state)
    src/layout/               -- Sidebar, Toast (notification system)
    src/pages/                -- 7 pages: Dashboard, Escrows, Offers, Vaults, Reputation, Jury, Swap
    src/components/           -- 8 domain component files (23+ components)
    src/__tests__/            -- 9 test files (36 tests)
    biome.json                -- lint config
    vite.config.ts            -- build + vitest config

  bot/                (Telegram bot)
    src/index.js              -- grammY, 16 commands + 4-step /create wizard
    src/lib/api.js            -- REST API client with retry/backoff

  scripts/            (Dev tooling)
    simulation.py             -- mass trade gen + reputation test
    e2e.py                    -- end-to-end integration test
    genkeys.py                -- key generation utility
    deploy-testnet.sh         -- testnet deployment
    deploy-mainnet.sh         -- mainnet Docker deployment
    deploy-contracts.sh       -- contract deployment
    deploy-web.sh             -- web deployment
    local-testnet.sh          -- local simnet kaspad

  Dockerfile           (Multi-stage build for production)
  nginx.conf           (Reverse proxy config)
```

## Need to change X? Start here
| Change | Look at |
|--------|---------|
| Escrow spending rules | `contracts/src/daglock.sil` |
| KRC-20 token escrow | `contracts/src/daglock_krc20.sil` |
| Arbiter dispute paths | `contracts/src/daglock_arbiter.sil` |
| Covenant compilation API | `contracts/src/lib.rs` |
| Add REST endpoint | `indexer/src/api/mod.rs` (router) + new handler module |
| Add DB table/column | `indexer/src/db/schema.rs` (migration) + `indexer/src/db/queries/<domain>.rs` |
| Change reputation formula | `indexer/src/db/queries/reputation.rs` (calculate_reputation_score) |
| Add CLI subcommand | `cli/src/main.rs` (enum) + new `cli/src/commands/*.rs` |
| Add web component | `web/src/components/<domain>.tsx` + add to `App.tsx` imports |
| Add web test | `web/src/__tests__/<Component>.test.tsx` (mock `../api` with `mockApi()`) |
| Add bot command | `bot/src/index.js` (command handler) |
| Change fee percentage | `contracts/src/daglock.sil` (hardcoded `inputValue / 200`) |
| Add vault type | Write new `.sil` file + add `compile_*()` in `contracts/src/lib.rs` |
| SDK integration | `npm install @daglock/sdk` |
| Embed escrow widget | `<script src="https://unpkg.com/@daglock/widget"><daglock-escrow>` |
| Change template hash logic | `contracts/src/lib.rs` (`template_parts_and_hash()`) |
| Add authentication | `indexer/src/auth.rs` (SignatureVerifier trait) |
| Change rate limit | `indexer/src/ratelimit.rs` (RateLimiter struct, max_requests/window_secs) |
| Add UTXO verification | `indexer/src/verification.rs` (EscrowVerifier trait) |
| Change message encryption | `indexer/src/crypto.rs` (AES-256-GCM, DAGLOCK_MESSAGE_KEY) |
| Change jury rules | `indexer/src/api/jury.rs` |
| Run reputation sim | `scripts/simulation.py` |
| Add web page | `web/src/router.tsx` (Route type) + `web/src/App.tsx` (import + case) + new `.tsx` in `web/src/pages/` |
| Change dashboard content | `web/src/pages/Dashboard.tsx` |
| Add developer docs tab | `web/src/pages/DocsPage.tsx` |
| Change offer card display | `web/src/pages/OffersPage.tsx` (OfferCard function) |
| Add vault type | `web/src/pages/VaultsPage.tsx` (VAULT_TYPE_INFO map + CreateVault form) |
| Run trade bot | `scripts/trade-bot.py` (systemd timer on VPS, every 10 min) |

## Domain registry
| Domain | Doc | Files | Purpose |
|--------|-----|-------|---------|
| shared | `—` (part of indexer) | 3 | Shared constants + validation crate |
| contracts | `features/contracts.md` | 12 (7 src + 5 tests) | SilverScript covenants + compilation + tests |
| indexer | `features/indexer.md` | 48 (45 src + 3 tests) | REST API, DB, auth, crypto, verification, jury, messages, webhooks, apps, services |
| cli | `features/cli.md` | 14 | Command-line power-user tool |
| wasm-sdk | `features/wasm-sdk.md` | 1 | Browser/JS SDK for tx assembly |
| web | `features/web.md` | 37 (28 src + 9 tests) | React + Vite dashboard (Dashboard, Docs, 6 feature pages) |
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

### 2026-06-06 — Pre-Mainnet Security & Usability Audit

**Scope:** Full codebase — contracts, indexer, CLI, WASM SDK, web, bot

#### Fix Status (June 16, 2026) — 27 of 30 tasks completed

**Critical Security:** ✅ All 7 fixed | 0 remaining

| ID | Issue | Severity | Status |
|----|-------|----------|--------|
| **S1** | MockVerifier used in production — no real UTXO verification | CRITICAL | ✅ Fixed — async `WrpcVerifier` with `get_utxos_by_addresses()` |
| **S2** | KRC-20 fee validation only boolean, not exact amount | HIGH | ✅ Fixed — exact `outputs[1].value == inputValue` + treasury script check |
| **S3** | KRC-20 KCC-20 output ownership validation | HIGH | ⏭️ Closed — multi-sig design prevents (both signers must agree with SIGHASH_ALL) |
| **S4** | trade_hash not validated on escrow creation | MEDIUM | ✅ Fixed — `daglock_shared::validate_trade_hash()` rejects malformed input |
| **S5** | No replay protection on signed messages | MEDIUM | ✅ Fixed — `action:id:ts:nonce` format, 5-min window, DB-backed nonce store (migration 014) |
| **S6** | Bot stores addresses in plaintext /tmp | MEDIUM | ✅ Fixed — AES-256-GCM encryption with `BOT_ENCRYPTION_KEY` env var |
| **S7** | Dockerfile runs as root | MEDIUM | ✅ Fixed — non-root `daglock` user |

**Usability:** 6 fixed | 1 remaining

| ID | Issue | Status |
|----|-------|--------|
| U1 | CLI create uses dummy keys, no wallet integration | ✅ Fixed — `cli/src/wallet.rs` with `sign_with_kaspawallet()`, `parse_hex_key()`, `kaspawallet_available()` |
| U2 | Web CreateEscrowForm generates fake lock_tx_id | ✅ Fixed — KasWare sendKaspa() returns real tx_id |
| U3 | No wallet signing in CLI | ✅ Fixed — `cli/src/wallet.rs` with `sign_with_kaspawallet()` |
| U4 | Bot /create redirects to web, no native flow | ✅ Fixed — 4-step conversation wizard |
| U5 | Generic "internal error" for all 500s | ✅ Fixed — `ApiErrorCode` enum (21 variants) |
| U6 | CoinGecko price fetch no fallback/caching | ⚠️ Partial — 5-min TTL cache exists, CoinGecko fallback not fully wired |
| U7 | No web onboarding for first-time users | ✅ Fixed — welcome modal on first visit |

**Structural:** 5 fixed | 3 remaining

| ID | Issue | Status |
|----|-------|--------|
| A1 | EscrowVerifier trait sync but wRPC is async | ✅ Fixed (done with S1) |
| A2 | Migration .ok() silences failures | ✅ Fixed — proper PRAGMA table_info checks |
| A3 | queries.rs split into 11 modules | ✅ Fixed |
| A4 | No full lifecycle integration test | ✅ Fixed — `indexer/tests/lifecycle_tests.rs` with 17 test functions covering escrow, offer, vault, dispute flows |
| A5 | Handlers mix HTTP + business logic + DB | ✅ Fixed — `indexer/src/services/escrow_service.rs` with `create()`, `settle()`, `refund()`, `dispute()`, `cancel()`, `atomic_swap()` |
| A6 | Bot is Node.js, rest is Rust | ❌ Open (out of scope for mainnet) |
| A7 | No OpenAPI spec | ❌ Open |
| A8 | No template hash verification on create | ✅ Fixed — checks against configured templates |

**Code Quality:** 8 fixed | 0 remaining

| ID | Issue | Status |
|----|-------|--------|
| Q1 | .unwrap() in production code | ✅ Fixed — zero found in indexer/src/ + cli/src/ |
| Q2/Q3 | Magic number 200 scattered in 5+ locations | ✅ Fixed — `FEE_DENOMINATOR` wired in indexer, CLI, WASM SDK |
| Q4 | trade_hash handling inconsistent | ✅ Fixed — `daglock_shared::validate_trade_hash()` |
| Q5 | No structured request tracing | ✅ Fixed — `request_id_middleware` generates UUID v4 per request, `X-Request-Id` header on responses |
| Q6 | Config validation gaps | ✅ Fixed — `Config::validate()` panics on invalid combinations at startup |
| Q7 | Web API no request timeout | ✅ Fixed — 30s AbortController on all fetch calls |
| Q8 | Bot API no retry/backoff | ✅ Fixed — 3 attempts, 1s/2s/4s exponential backoff |

#### Key Files Changed

| File | Change |
|------|--------|
| `shared/` (4 new files) | Constants + validation crate with 20 tests |
| `contracts/src/daglock_krc20.sil` | Exact fee enforcement in release/swap paths |
| `contracts/tests/daglock_krc20_execution_tests.rs` | Wrong-fee rejection test added |
| `indexer/src/verification.rs` | Async `WrpcVerifier` with real `get_utxos_by_addresses()` |
| `indexer/src/auth.rs` | Replay protection: `action:id:ts:nonce`, nonce DB, backward compat |
| `indexer/src/services/escrow_service.rs` | Service layer: create/settle/refund/dispute/cancel/atomic_swap |
| `indexer/tests/lifecycle_tests.rs` | 17 lifecycle integration tests |
| `indexer/src/api/mod.rs` | `request_id_middleware` + tracing span per request |
| `indexer/src/config.rs` | `Config::validate()` startup validation |
| `indexer/src/api/escrows.rs` | `.await` on all verification calls, trade_hash validation, template hash validation |
| `indexer/src/db/queries/auth.rs` | `store_auth_nonce()`, `check_auth_nonce_exists()` |
| `indexer/src/db/schema.rs` | Migration 014 (auth_nonces), migration idempotency fixes |
| `cli/src/wallet.rs` | `sign_with_kaspawallet()`, `parse_hex_key()`, `kaspawallet_available()` |
| `wasm-sdk/src/lib.rs` | `kas_to_sompi` + `validate_trade_hash` WASM exports |
| `bot/src/index.js` | AES-256-GCM encryption for user address storage |
| `bot/src/lib/api.js` | 3-attempt retry with exponential backoff + 10s timeout |
| `web/src/api.ts` | 30s AbortController timeout on all requests |
| `web/src/App.tsx` | First-visit onboarding modal |
| `Dockerfile` | Non-root `daglock` user |

#### Rules Compliance (updated)

| Rule | Status |
|------|--------|
| #1: Never `.unwrap()` outside tests | ✅ Compliant (Q1 verified) |
| #2: Never hardcode addresses/keys in covenant | ✅ Compliant |
| #3: Never skip fee validation in release/swap | ✅ Fixed |
| #4: Never expose private keys | ✅ Compliant |
| #5: Never change fee denominator without updating all paths | ⚠️ `FEE_DENOMINATOR` constant exists but covenants still hardcode `200` |
| #6: Never use non-atomic updates for lifecycle | ✅ Compliant |
| #7: Never skip address validation on create | ✅ Compliant |

#### Remaining Work (3 tasks, targeting June 30)

| Phase | Priority | Tasks | Effort |
|-------|----------|-------|--------|
| Structural | Low (nice-to-have) | A6 (bot rewrite), A7 (OpenAPI spec) | ~3 days |
| Usability | Low (nice-to-have) | U6 (CoinGecko full wiring) | ~2 hours |

**Next priority:** Testnet deploy with real wRPC node, mainnet readiness checklist.

