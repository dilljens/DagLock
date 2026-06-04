# DagLock 🔒⛓️

**Trustless escrow & atomic swaps on Kaspa L1 via SilverScript covenants.**

Lock assets directly into Kaspa's BlockDAG state. Release them only when cryptographic conditions are met. No intermediaries. No admin keys. No custodial risk.

---

## Why DagLock?

Kaspa's Toccata Hard Fork (June 2026) introduced native Layer 1 smart contracts via SilverScript covenants (KIP-17/KIP-20). For the first time, UTXO-native financial primitives are possible on Kaspa — and DagLock is the first to build them.

| Problem | DagLock Solution |
|---|---|
| OTC trades require trusted escrow agents | Trustless covenant holds funds; code enforces the terms |
| Cross-chain swaps need centralized bridges | Atomic swaps via hash preimage — no bridge needed |
| High-value token trades lack safe infrastructure | Isolated UTXO per trade — no shared pools, no MEV |
| Escrow fees are opaque and arbitrary | Programmatic 0.5% fee, enforced by the covenant itself |

---

## Repository Structure

```
daglock/
├── contracts/     SilverScript covenant source (.sil) + compiler tests
├── indexer/       Rust daemon — wRPC listener + REST API (30+ endpoints)
├── cli/           Command-line tool for power users
├── bot/           Telegram bot (@DagLockBot)
├── wasm-sdk/      WASM bindings for browser wallet integration
├── web/           React + Vite dashboard (full action UI)
├── docs/          Architecture, protocol, API, roadmap, security
├── scripts/       Dev tooling (simulation, deploy, key generation)
├── Dockerfile     Multi-stage production build
└── LICENSE        AGPL v3
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full system design.

---

## Quick Start (Development)

```bash
# Prerequisites: Rust 1.85+, Node 22+, Docker

# 1. Build and start the indexer
cargo run -p daglock-indexer

# 2. Start the web UI (separate terminal)
cd web && npm install && npm run dev

# 3. Open http://localhost:5173

# 4. Run the simulation (optional, generates test data)
python3 scripts/simulation.py --trades 20 --bots 2
```

---

## Documentation

| Document | What it covers |
|---|---|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design, component relationships, data flow |
| [PROTOCOL.md](docs/PROTOCOL.md) | Covenant semantics, tx structure, parameter encoding |
| [API.md](docs/API.md) | Indexer REST API reference |
| [DEPLOYMENT.md](docs/DEPLOYMENT.md) | Production deployment + config reference |
| [ROADMAP.md](docs/ROADMAP.md) | Phased delivery timeline |
| [SECURITY.md](docs/SECURITY.md) | Threat model, audit checklist, bug bounty |
| [KRC20-TESTNET.md](docs/KRC20-TESTNET.md) | KRC-20 testnet deployment guide |
| [wiki/docs/_index.md](docs/wiki/_index.md) | AI-optimized codebase map |

---

## License

GNU Affero General Public License v3.0 — see [LICENSE](LICENSE).
