# DagLock

**Trustless escrow & atomic swaps on Kaspa L1 via SilverScript covenants.**

---

## Quick Reference

| Field | Value |
|---|---|
| Language | SilverScript (contracts), Rust (indexer/CLI), TypeScript (web), Node.js (Telegram bot) |
| Network | Kaspa Testnet 12 → Mainnet (Toccata hard fork, June 5–20, 2026) |
| Contract format | UTXO covenants (KIP-17/KIP-20) |
| Compiler | `silverscript-lang` branch `tn12` |
| Node SDK | `rusty-kaspa` branch `tn12` — `kaspa-wrpc-client`, `kaspa-txscript` |
| Wallet target | KasWare (web extension), Kaspium (mobile) |
| Indexer DB | PostgreSQL or SQLite via SQLx |
| Fee model | 0.5% (1/200) protocol fee to DagLock treasury |
| Target users | KRC-20 token communities, OTC traders, whale-to-whale KAS swaps |
| Dev status | Pre-alpha — covenant written, tests passing structure defined |

---

## Assets Supported

| Asset | Covenant | Phase |
|---|---|---|
| Native KAS | `daglock.sil` | Phase 0 |
| KRC-20 tokens | `daglock_krc20.sil` | Phase 0 |
| Cross-chain HTLC (BTC/LTC) | Future | Phase 6+ |

---

## Product Surfaces

| Surface | Audience | Channel |
|---|---|---|
| **DagLock Telegram Bot** | KRC-20 traders, community members | `@DagLockBot` on Telegram |
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

## Repository Structure

```
daglock/
├── contracts/         # SilverScript source
│   ├── src/daglock.sil          # KAS escrow covenant
│   ├── src/daglock_krc20.sil    # KRC-20 escrow covenant
│   ├── src/lib.rs               # Compile + template hash extraction
│   └── tests/                   # TxScriptEngine execution tests
│
├── indexer/           # Rust daemon — wRPC listener + REST API + offer board
│
├── cli/               # daglock-cli — power-user terminal tool
│
├── bot/               # Telegram bot (Node.js or Rust)
│
├── web/               # React + Vite dashboard
│
├── docs/              # Architecture, protocol, API, roadmap, security
│   └── reference/     # SilverScript language docs (from upstream)
│
└── scripts/           # Dev tooling
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

**Phase 0 — Covenant development.** `daglock.sil` written and compilable. `daglock_krc20.sil` pending. Tests defined. Reference docs ingested from upstream SilverScript.

---

## Codebase Wiki

AI-optimized codebase map at `docs/wiki/`.

**For AI agents (cold start):**
1. `docs/wiki/_glossary.md` — project vocabulary
2. `docs/wiki/_index.md` — architecture topology + domain one-liners
3. `docs/wiki/_standards.md` § Rules — what never to do
4. `docs/wiki/_standards.md` § Practices — how to write new code
5. `docs/wiki/features/<domain>.md` — the domain you're working on
6. `docs/wiki/_standards.md` § Patterns — match conventions during generation

**Commands:** `/wiki:make` (init), `/wiki:update` (refresh), `/wiki:check` (verify consistency)
