# DagLock — Trustless Escrow on Kaspa

> **Testnet beta live at [daglock.com](https://daglock.com).** Mainnet launch June 30, 2026 (Toccata hard fork at DAA 474,165,565).

> ⚠️ **Dev mode:** The public Kaspa wRPC resolvers (kaspa.red/green/blue) are offline during the Toccata v2 migration. The indexer uses dev mode for UTXO checks — the covenants enforce all rules trustlessly on-chain. Full wRPC verification will be enabled post-Toccata. [Details](https://github.com/kaspanet/rusty-kaspa/releases/tag/v2.0.0)

**Trustless escrow, atomic swaps, and time-locked vaults on Kaspa L1 via SilverScript covenants.**

Lock assets directly into Kaspa's BlockDAG state. Release them only when cryptographic conditions are met. No intermediaries. No admin keys. No custodial risk.

---

## Quick Links

| Surface | URL | What you can do |
|---------|-----|-----------------|
| **Web dashboard** | [daglock.com](https://daglock.com) | Create offers & escrows, check reputation, vaults, jury, docs |
| **API** | [api.daglock.com](https://api.daglock.com) | REST API (19+ endpoints). [OpenAPI spec](https://api.daglock.com/v1/openapi.json) |
| **Telegram bot** | [@DagLock_bot](https://t.me/DagLock_bot) | Full bot — `/create`, `/offers`, `/swap`, `/vaults`, `/reputation` |
| **CLI** | `cargo install --git ... daglock-cli` | Power-user terminal tool — create, claim, status, reputation |
| **Docs** | `/#/docs` on daglock.com | API reference, CLI guide, bot commands, integration guide |

---

## For Users

### What DagLock offers

| Feature | Description | Fee |
|---------|-------------|-----|
| **Escrow** | Lock KAS or KRC-20 tokens in a covenant. Only buyer or seller can settle. | 0.5% on settlement |
| **Atomic Swaps** | Cross-asset trades via hash preimage. Both parties commit funds, then reveal the secret. | 0.5% on settlement |
| **Vaults** | Time-locked self-custody. Standard, softlock (password-recoverable), or multisig (2-of-3). | 0.1% on withdrawal |
| **Reputation** | On-chain derived scores based on trade history, vouching, and identity verification. | Free |
| **Jury** | Community dispute resolution via randomly selected jurors. | Free |
| **Messaging** | Encrypted chat tied to each escrow (AES-256-GCM). | Free |

### Getting started

1. Install [KasWare](https://kasware.xyz) browser extension
2. Get testnet KAS from the [faucet](https://faucet.testnet12.kaspa.org/)
3. Open [daglock.com](https://daglock.com) and connect your wallet
4. Browse offers or create your first escrow

---

## For Developers

### Quick Start

```bash
# Prerequisites: Rust 1.91+, Node 22+

# 1. Build and start the indexer (v2.0.1 — Toccata SDK)
cargo run -p daglock-indexer -- --network testnet-11 --no-wrpc

# 2. Start the web UI (separate terminal)
cd web && npm ci && npm run dev

# 3. Open http://localhost:5173

# 4. Generate test reputation data
python3 scripts/simulation.py --trades 20 --bots 2
```

### Current Architecture

```
daglock.com (Cloudflare Pages)
  → API calls to api.daglock.com
    → Cloudflare proxy → nginx → daglock-indexer :8443
      → MockVerifier (wRPC resolvers offline during Toccata v2 migration)
      → Telegram bot + trade bot on same VPS

All on one OVHcloud VPS-2

Mainnet (June 30): upgraded to CPX42+ with local kaspad for wRPC verification.
```

### Covenant Templates

| Template | File | Use case |
|----------|------|----------|
| **KAS Escrow** | `daglock.sil` | Standard OTC trades, atomic swaps |
| **KRC-20 Escrow** | `daglock_krc20.sil` | Token-for-KAS escrow with ICC pattern |
| **Arbiter** | `daglock_arbiter.sil` | Escrow with mediator or jury dispute resolution |
| **Vault (standard)** | `daglock_vault.sil` | Time-locked self-custody (withdraw after timeout) |
| **Vault (softlock)** | `daglock_vault_softlock.sil` | Password-recoverable vault |
| **Vault (multisig)** | `daglock_vault_multisig.sil` | 2-of-3 multi-signature vault |

All fees (0.5% escrow, 0.1% vault) are enforced by the covenant itself — DagLock cannot change or waive them.

---

## Repository Structure

```
daglock/
├── contracts/     SilverScript covenant source (.sil) + compiler tests
├── indexer/       Rust daemon — wRPC listener + REST API (19+ endpoints)
├── cli/           Command-line tool for power users
├── bot/           Telegram bot (@DagLock_bot)
├── wasm-sdk/      WASM bindings for browser wallet integration
├── web/           React + Vite dashboard (8 pages, 40 tests)
├── shared/        Shared Rust crate (constants + validation)
├── scripts/       Dev tooling (simulation, deploy, key generation, trade bot)
├── docs/          Architecture, protocol, API, roadmap, security
├── Dockerfile     Multi-stage production build
└── LICENSE        AGPL v3
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full system design.

---

## Documentation

| Document | What it covers |
|---|---|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design, component relationships, data flow |
| [PROTOCOL.md](docs/PROTOCOL.md) | Covenant semantics, tx structure, parameter encoding |
| [API.md](docs/API.md) | Indexer REST API reference (30+ endpoints) |
| [SECURITY.md](docs/SECURITY.md) | Threat model, audit checklist |
| [ROADMAP.md](docs/ROADMAP.md) | Phased delivery timeline |
| [WIKI](docs/wiki/_index.md) | AI-optimized codebase map |
| [AGENTS.md](AGENTS.md) | Full project context for AI coding agents |

---

## Roadmap

| Phase | Status | What |
|-------|--------|------|
| 0 | ✅ | KAS + KRC-20 + Arbiter + Vault covenants written, compiled, tested |
| 1 | ✅ | Indexer with REST API, offers board, reputation system, auth |
| 2 | ✅ | Telegram bot, CLI tool, encrypted messaging, replay protection |
| 3 | ✅ | Web dashboard redesign (8 pages, skeletons, animations, Radix UI) |
| 4 | ✅ | Production hardening (rate tiers, daily caps, WebSocket, TanStack Query) |
| 5 | 🔜 | **Mainnet deployment — June 30, 2026** (Toccata hard fork) |

---

## License

GNU Affero General Public License v3.0 — see [LICENSE](LICENSE).
