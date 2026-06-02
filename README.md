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
├── indexer/       Rust daemon — wRPC listener + REST API
├── wasm-sdk/      WASM bindings for browser wallet integration
├── web/           React + Vite + Tailwind dashboard
├── docs/          Architecture, protocol, API, roadmap, security
└── scripts/       Dev tooling (local testnet, deploy)
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full system design.

---

## Quick Start (Development)

```bash
# Prerequisites: Rust 1.85+, Node 22+, Docker

# 1. Clone and enter the repo
git clone https://github.com/your-org/daglock
cd daglock

# 2. Start a local Kaspa simnet
./scripts/local-testnet.sh

# 3. Compile the covenant
cd contracts && cargo build && cd ..

# 4. Start the indexer (in another terminal)
cd indexer && cargo run -- --config ../config.toml && cd ..

# 5. Start the web UI (in another terminal)
cd web && npm install && npm run dev
```

---

## Documentation

| Document | What it covers |
|---|---|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design, component relationships, data flow |
| [PROTOCOL.md](docs/PROTOCOL.md) | Covenant semantics, tx structure, parameter encoding |
| [API.md](docs/API.md) | Indexer REST API reference |
| [ROADMAP.md](docs/ROADMAP.md) | Phased delivery timeline (weeks 1–6+) |
| [SECURITY.md](docs/SECURITY.md) | Threat model, audit checklist, bug bounty |

---

## License

MIT — see [LICENSE](LICENSE).
