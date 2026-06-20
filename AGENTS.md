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
| Indexer | Hetzner VPS | ✅ Running (wRPC connected) | `api.daglock.com` |
| Bot | Hetzner VPS | ✅ Running | `@DagLock_bot` on Telegram |
| Web UI | Cloudflare Pages | ✅ Running | `daglock.com` |
| Kaspa Node | Hetzner VPS | ✅ Syncing testnet-12 | `46.224.171.239:16610` |
| Trade Bot | Hetzner VPS | ✅ Systemd timer (10 min) | — |

**Architecture:**
```
┌─────────────────────────────────────────────────────────────┐
│  Hetzner VPS CX23 ($5/mo) — 46.224.171.239                 │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   kaspad     │  │ daglock-     │  │ daglock-bot       │  │
│  │ (testnet-12) │◄─┤ indexer      │  │ (Telegram)        │  │
│  │ wRPC :16610  │  │ :8443        │  │                    │  │
│  └──────────────┘  └──────┬───────┘  └──────────────────┘  │
│                           │ nginx                          │
│                           │ :443 (Cloudflare SSL)          │
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
| KRC-20 | `8a43a8438d183a92bc7b94337c031196ff16725b` |
| Reputation | `65c54102c64a331414b602760cbd76efac3d69df` |

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

**Audit Fix Phase (June 6 — June 30, 2026).** Comprehensive audit completed June 6 identified:
- **7 Critical/High security issues** (S1-S7) — must fix before mainnet
- **7 High-priority usability issues** (U1-U7) — CLI/web/bot can't create real escrows
- **8 Structural/architectural concerns** (A1-A8) — tech debt
- **8 Code quality issues** (Q1-Q8) — polish

**30-task fix plan** in `.pi/last-plan.md` across 4 phases:
- Phase 0: Shared constants (2 tasks)
- Phase 1: Critical security (8 tasks) — **blocks mainnet**
- Phase 2: Usability (7 tasks)
- Phase 3: Structural (6 tasks)
- Phase 4: Polish (6 tasks)

**Target mainnet launch: June 30, 2026** (same day as Toccata hard fork activation).

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
- [ ] **S1** Real async `WrpcVerifier` with `get_utxos_by_outpoints`
- [ ] **S2** KRC-20 exact fee enforcement in covenant
- [ ] **S3** KRC-20 KCC-20 output ownership validation
- [ ] **S4** Validate `trade_hash` on escrow creation
- [ ] **S5** Replay protection (timestamp + nonce in auth messages)
- [ ] **S6** Bot encrypt user addresses at rest (libsodium)
- [ ] **S7** Dockerfile non-root user

### Usability (High Priority)
- [ ] **U1** CLI create with real wallet keys (`kaspawallet sign`)
- [ ] **U2** Web real `lock_tx_id` flow (WASM → KasWare → broadcast → submit)
- [ ] **U3** CLI wallet module (shared signing)
- [ ] **U4** Bot native `/create` wizard (grammY conversations)
- [ ] **U5** Structured API errors (`ApiErrorCode` enum)
- [ ] **U6** CoinGecko fallback + caching (TTL 15min)
- [ ] **U7** Web onboarding modal (first-visit)

### Structural
- [ ] **A1** Async verifier trait (done with S1)
- [ ] **A2** Migration idempotency (PRAGMA checks)
- [ ] **A3** Split `queries.rs` into 11 modules
- [ ] **A4** Lifecycle integration tests
- [ ] **A5** Service layer (`EscrowService`)
- [ ] **A7** OpenAPI spec (utoipa)
- [ ] **A8** Template hash verification on create

### Code Quality
- [ ] **Q1** Remove `.unwrap()` in production
- [ ] **Q2/Q3** Shared fee constant everywhere
- [ ] **Q4** `TradeHash` newtype with `FromStr`
- [ ] **Q5** Request tracing (request_id, user_address, escrow_id)
- [ ] **Q7** Web API timeout (AbortController 30s)
- [ ] **Q8** Bot API retry/backoff (3 attempts, exponential)

---

## Verification Gates (Pre-Mainnet)

Before June 30 launch, ALL must pass:
- [ ] `cargo test --workspace`
- [ ] `cargo test -p daglock-contracts` — all covenant execution tests
- [ ] `cargo test -p daglock-indexer --test lifecycle_tests` — full lifecycle
- [ ] `cd web && npm test && npm run lint && npm run build`
- [ ] `cd bot && npm test`
- [ ] Manual: Testnet deploy with `--wrpc-url` → web create → KasWare sign → broadcast → settle → receipt
- [ ] Manual: CLI `daglock-cli create` → `kaspawallet sign` → broadcast → settle
- [ ] Manual: Bot `/create` wizard → deep link → complete flow
