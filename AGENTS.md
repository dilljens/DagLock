# DagLock

**Trustless escrow & atomic swaps on Kaspa L1 via SilverScript covenants.**

---

## Quick Reference

| Field | Value |
|---|---|
| Language | SilverScript (contracts), Rust (indexer/CLI), TypeScript (web), Node.js (Telegram bot) |
| Network | Kaspa Testnet 12 → Mainnet (Toccata hard fork, ~June 30, 2026) |
| Contract format | UTXO covenants (KIP-17/KIP-20) |
| Compiler | `silverscript-lang` branch `master` |
| Node SDK | `rusty-kaspa` tag `v2.0.1` — `kaspa-wrpc-client`, `kaspa-txscript` |
| Wallet target | KasWare (web extension), Kaspium (mobile) |
| Indexer DB | PostgreSQL or SQLite via SQLx |
| Fee model | 0.5% (1/200) protocol fee to DagLock treasury |
| Target users | KRC-20 token communities, OTC traders, whale-to-whale KAS swaps |
| Dev status | **Pre-mainnet** — Audit completed June 6, 2026. 7 critical/high security findings + 7 usability issues. Fix plan in progress (target: June 30). |

---

## Assets Supported

| Asset | Covenant | Phase |
|---|---|---|
| Native KAS | `daglock.sil` | Phase 0 |
| KRC-20 tokens | `daglock_krc20.sil` | Phase 0 |
| Time-locked vault | `daglock_vault.sil` | Phase 0 |
| Arbiter (mediator/jury) | `daglock_arbiter.sil` | Phase 0 |
| Cross-chain HTLC (BTC/LTC) | Future | Phase 6+ |

---

## Product Surfaces

| Surface | Audience | Channel |
|---|---|---|
| **DagLock Telegram Bot** | KRC-20 traders, community members | `@DagLock_bot` on Telegram |
| **Web Dashboard** | Desktop users, whales, OTC desks | `daglock.io` |
| **CLI Tool** | Power users, integrators, testers | `daglock-cli` binary |
| **REST API** | Other dApps embedding DagLock escrow | `api.daglock.io/v1` |

---

## Key Features (Market-Informed)

1. **KRC-20 support at launch** — KAS-only P2P volume is negligible. The KRC-20 community IS the user base.
2. **Counterparty discovery board** — Users need to find each other. Public listing of open escrow offers.
3. **Telegram bot** — The Kaspa community lives on Telegram. Meet them there.
4. **Proposal-before-commit** — Negotiate terms before locking funds. Reduces friction.
5. **On-chain reputation** — Trade count, volume, account age derivable from indexer data.
6. **Settlement receipts** — Cryptographic proof of completed trades. Exportable, verifiable.
7. **Volume-based fee tiers** — Off-chain rebates for high-volume traders. Covenant stays simple.
8. **Atomic swap wizard** — Abstracts hash preimage protocol behind a guided UI.

---

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                     Users                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐               │
│  │ Telegram │  │   Web    │  │   CLI    │               │
│  │   Bot    │  │ Dashboard│  │   Tool   │               │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘               │
│       │             │             │                      │
│       └─────────────┼─────────────┘                      │
│                     │ KasWare / Kaspium signing          │
└─────────────────────┼────────────────────────────────────┘
                      │
         ┌────────────▼────────────┐
         │    DagLock Indexer      │
         │  ┌──────────────────┐   │
         │  │ wRPC Listener    │   │
         │  │ Template Matcher │   │
         │  │ SQLite/Postgres  │   │
         │  │ REST API         │   │
         │  │ Offer Board      │   │
         │  │ Reputation Engine│   │
         │  └──────────────────┘   │
         └────────────┬────────────┘
                      │ wRPC
         ┌────────────▼────────────┐
         │   Kaspa Node (TN12)     │
         │   BlockDAG + UTXO Set   │
         └─────────────────────────┘
```

---

## Deployment

### Current Setup (June 17, 2026)

| Component | Platform | Status | URL |
|-----------|----------|--------|-----|
| Indexer | Hetzner VPS | ✅ Running (MockVerifier) | `api.daglock.com` |
| Bot | Hetzner VPS | ✅ Running | `@DagLock_bot` on Telegram |
| Web UI | Cloudflare Pages | ✅ Running | `daglock.com` |
| Kaspa Node | — | ❌ No node (VPS too small) | Uses `--no-wrpc` offline mode |
| Trade Bot | Hetzner VPS | ✅ Systemd timer (10 min) | — |

**Architecture:**
```
┌─────────────────────────────────────────────────────────────┐
│  Hetzner VPS CX23 ($5/mo) — 46.224.171.239                 │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   kaspad     │  │ daglock-     │  │ daglock-bot       │  │
│  │ (testnet-11) │◄─┤ indexer      │  │ (Telegram)        │  │
│  │ wRPC :16610  │  │ :8443        │  │                    │  │
│  └──────────────┘  └──────┬───────┘  └──────────────────┘  │
│         ❌ No node        │ nginx                          │
│   (VPS too small)         │ :443 (Cloudflare SSL)          │
│                           └──────┬───────────────          │
│                                  │ api.daglock.com         │
│  ┌──────────────────────┐        │                         │
│  │ daglock-trade-bot    │        │                         │
│  │ (systemd timer 10m)  │        │                         │
│  └──────────────────────┘        │                         │
└──────────────────────────────────┼─────────────────────────┘
                                   │
                     ┌─────────────▼─────────────┐
                     │  Cloudflare Pages (free)   │
                     │  daglock.com               │
                     │  → API calls to            │
                     │    api.daglock.com          │
                     └───────────────────────────┘
```

### Why Not a Local Node?

- Public resolver nodes (kaspa.stream/red/green/blue) are down (wRPC v2 migration)
- 6GB laptop RAM is tight for kaspad + indexer
- 2GB Pi 4 insufficient for kaspad
- VPS is the most reliable option ($5/mo)

### Template Hashes (Testnet-12)

| Covenant | Hash |
|----------|------|
| KAS | `30876e3ea42d0e23bb0980f3fd97ae8807e9c70f` |
| KRC-20 | `ae0946e4a9bd4a7585e6bf9135de38083cb11c85` |
| Reputation | `65c54102c64a331414b602760cbd76efac3d69df` |
| Vault (softlock) | `9777c9eb9e6271a32fac75d3533bc27d25b20d39` |
| Vault (multisig) | `b0cddcd4dc716532fd86d1809a05f8ea7e74113d` |
| Vault (standard) | `6ca87fa1f22b0acde59eb971789664de9c539782` |

---

## Repository Structure

```
daglock/
├── contracts/         # SilverScript source
│   ├── src/daglock.sil          # KAS escrow covenant
│   ├── src/daglock_krc20.sil    # KRC-20 escrow covenant
│   ├── src/daglock_arbiter.sil  # KAS escrow with mediator/jury
│   ├── src/daglock_vault.sil    # Time-locked vault
│   ├── src/lib.rs               # Compile + template hash extraction
│   └── tests/                   # TxScriptEngine execution tests
│
├── indexer/           # Rust daemon — wRPC listener + REST API + offer board
│   ├── src/main.rs              # Entry, config, verifier wiring
│   ├── src/config.rs            # CLI args + production flags
│   ├── src/auth.rs              # Schnorr signature verification
│   ├── src/verification.rs      # UTXO verification (async WrpcVerifier / MockVerifier)
│   ├── src/listener.rs          # wRPC listener + DAA reconciliation + market prices
│   ├── src/crypto.rs            # AES-256-GCM message encryption
│   ├── src/types.rs             # All shared serde types
│   ├── src/db/schema.rs         # SQLx migrations
│   ├── src/db/queries/          # Split into domain modules (escrows, reputation, jury, etc.)
│   ├── src/api/                 # 10 handler modules
│   ├── src/services/            # Business logic layer (escrow_service)
│   └── tests/                   # Integration + lifecycle tests
│
├── cli/               # daglock-cli — power-user terminal tool
│   ├── src/main.rs              # clap dispatch
│   ├── src/config.rs            # ~/.daglock/config.toml
│   ├── src/tx.rs                # kas_to_sompi, tx assembly
│   ├── src/wallet.rs            # kaspawallet sign subprocess
│   └── src/commands/            # 10+ command files
│
├── wasm-sdk/          # Browser SDK
│   └── src/lib.rs               # wasm-bindgen, compile_escrow
│
├── web/               # React + Vite dashboard
│   ├── src/App.tsx              # Main app (layout, routing, tabs)
│   ├── src/api.ts               # REST client + TypeScript types
│   ├── src/kasware.ts           # KasWare wallet integration
│   ├── src/components/          # 8 domain component files
│   └── src/__tests__/           # 6 test files (Vitest + RTL)
│
├── bot/               # Telegram bot (Node.js, grammY)
│   ├── src/index.js             # Command handlers
│   └── src/lib/api.js           # Indexer REST client
│
├── shared/            # Shared Rust crate (NEW)
│   ├── src/constants.rs         # FEE_DENOMINATOR = 200
│   └── src/validation.rs        # validate_trade_hash, etc.
│
├── docs/              # Architecture, protocol, API, roadmap, security
│   └── wiki/                    # AI-optimized codebase map
│
└── scripts/           # Dev tooling
    ├── simulation.py            # Mass trade gen + reputation test
    ├── deploy-testnet.sh
    ├── deploy-mainnet.sh
    └── local-testnet.sh
```

---

## Core Design Principles

1. **One escrow, one UTXO.** Isolated cells. No shared contract wallets. Zero UTXO contention.
2. **Trustless by construction.** The covenant enforces all rules. No admin keys, no backdoors.
3. **Meet users where they are.** Telegram bot + web UI, not a custom mobile app.
4. **KRC-20 from day one.** The token communities are the addressable market.
5. **Proposal before commitment.** Negotiate terms, then lock funds. Lower friction.
6. **Reputation from on-chain data.** No centralized rating system — derived from verifiable trade history.

---

## Current Phase

**Testnet Launch:** Live at `@DagLock_bot` and `daglock.com` since June 17 on testnet-11.
**Audit: 28/30 items complete.** All 7 critical security items fixed (S3 closed via ICC pattern in covenant). 7 usability items fixed. 7/8 structural items. 5/6 code quality items.
**Mainnet target: June 30, 2026** (Toccata hard fork activation).

---

## Codebase Wiki

AI-optimized codebase map at `docs/wiki/`.

**For AI agents (cold start):**
1. `docs/wiki/_glossary.md` — project vocabulary
2. `docs/wiki/_index.md` — architecture topology + domain one-liners + **audit log**
3. `docs/wiki/_standards.md` § Rules — what never to do
4. `docs/wiki/_standards.md` § Practices — how to write new code
5. `docs/wiki/features/<domain>.md` — the domain you're working on
6. `docs/wiki/_standards.md` § Patterns — match conventions during generation

**Commands:** `/wiki:make` (init), `/wiki:update` (refresh), `/wiki:check` (verify consistency)

---

## Key Files for Common Tasks

| Change | Look at |
|--------|---------|
| Escrow spending rules (KAS) | `contracts/src/daglock.sil` |
| KRC-20 token escrow (fee/ownership fixes) | `contracts/src/daglock_krc20.sil` |
| Arbiter dispute paths | `contracts/src/daglock_arbiter.sil` |
| Vault time-lock | `contracts/src/daglock_vault.sil` |
| Covenant compilation API | `contracts/src/lib.rs` |
| Add REST endpoint | `indexer/src/api/mod.rs` + new handler module |
| Add DB table/column | `indexer/src/db/schema.rs` + `indexer/src/db/queries/<domain>.rs` |
| Change reputation formula | `indexer/src/db/queries/reputation.rs` |
| Add CLI subcommand | `cli/src/main.rs` + new `cli/src/commands/*.rs` |
| Add web component | `web/src/components/<domain>.tsx` + add to `App.tsx` imports |
| Add web test | `web/src/__tests__/<Component>.test.tsx` (mock `../api` with `mockApi()`) |
| Add bot command | `bot/src/index.js` (command handler) |
| Fee denominator (shared constant) | `shared/src/constants.rs` |
| Template hash logic | `contracts/src/lib.rs` (`template_parts_and_hash()`) |
| Authentication | `indexer/src/auth.rs` (`SignatureVerifier` trait) |
| UTXO verification | `indexer/src/verification.rs` (`EscrowVerifier` trait — now async) |
| Message encryption | `indexer/src/crypto.rs` (AES-256-GCM, `DAGLOCK_MESSAGE_KEY`) |
| Jury rules | `indexer/src/api/jury.rs` |
| Run reputation sim | `scripts/simulation.py` |

---

## Audit Status (June 6, 2026)

### Critical Security (Must Fix Before Mainnet)
- [x] **S1** Real async `WrpcVerifier` with `get_utxos_by_outpoints`
- [x] **S2** KRC-20 exact fee enforcement in covenant
- [x] **S3** KRC-20 output ownership validation (ICC pattern, gated by kcc20TemplatePrefixLen != 0)
- [x] **S4** Validate `trade_hash` on escrow creation
- [x] **S5** Replay protection (timestamp + nonce in auth messages)
- [x] **S6** Bot encrypt user addresses at rest (AES-256-GCM)
- [x] **S7** Dockerfile + VPS non-root user (daglock user)

### Usability (High Priority)
- [x] **U1** CLI create with real wallet keys (`kaspawallet sign`)
- [x] **U2** Web real `lock_tx_id` flow (WASM → KasWare → broadcast → submit)
- [x] **U3** CLI wallet module (shared signing)
- [x] **U4** Bot native `/create` wizard (grammY conversations)
- [x] **U5** Structured API errors (`ApiErrorCode` enum)
- [x] **U6** CoinGecko fallback + caching (TTL 15min)
- [ ] **U7** Web onboarding modal (first-visit) — low priority

### Structural
- [x] **A1** Async verifier trait
- [x] **A2** Migration idempotency (PRAGMA checks)
- [x] **A3** Split `queries.rs` into 11 modules
- [x] **A4** Lifecycle integration tests
- [x] **A5** Service layer (`EscrowService`)
- [x] **A7** OpenAPI spec (static JSON)
- [x] **A8** Template hash verification on create

### Code Quality
- [x] **Q1** Remove `.unwrap()` in production
- [x] **Q2/Q3** Shared fee constant everywhere
- [ ] **Q4** `TradeHash` newtype with `FromStr` — low priority
- [x] **Q5** Request tracing (request_id, user_address, escrow_id)
- [x] **Q7** Web API timeout (AbortController 30s)
- [x] **Q8** Bot API retry/backoff (3 attempts, exponential)

---

## Verification Gates (Pre-Mainnet)

Before June 30 launch, ALL must pass:
- [x] `cargo test --workspace`
- [x] `cargo test -p daglock-contracts` — all covenant execution tests (incl. S3 ICC fix)
- [x] `cargo test -p daglock-indexer --test lifecycle_tests` — full lifecycle
- [x] `cd web && npm test && npm run build`
- [x] `cd bot && npm test`
- [x] Manual: Testnet deploy with `--no-wrpc` → web create → KasWare sign → broadcast → settle → receipt
- [ ] Manual: CLI `daglock-cli create` → `kaspawallet sign` → broadcast → settle
- [ ] Manual: Bot `/create` wizard → deep link → complete flow

✅ All tests pass (241 Rust, 40 Web, 22 Bot = 303 total)

## VPS

Single OVH VPS running all projects. See `../VPS.md` for connection info, services, and commands.
