# DagLock 🔒⛓️

> ⚠️ **Testnet phase.** DagLock is deployed on Kaspa Testnet 12 for testing. Covenants activate on mainnet with the Toccata hard fork (June 5, 2026). Do not send real KAS yet.

**Trustless escrow & atomic swaps on Kaspa L1 via SilverScript covenants.**

Lock assets directly into Kaspa's BlockDAG state. Release them only when cryptographic conditions are met. No intermediaries. No admin keys. No custodial risk.

---

## For Users

**Try it now:**

- **Web app:** Visit [daglock.com](https://daglock.com) — no installation needed
- **Telegram bot:** Message [@DagLockBot](https://t.me/DagLockBot) on Telegram
- **CLI:** `cargo install --git https://github.com/dilljens/DagLock daglock-cli`

**Create escrows, check reputation, find counterparties, and resolve disputes — all without trusting a middleman.**

---

## For Developers

### Quick Start

```bash
# Prerequisites: Rust 1.85+, Node 22+

# 1. Start the indexer
cargo run -p daglock-indexer

# 2. Start the web UI (separate terminal)
cd web && npm install && npm run dev

# 3. Open http://localhost:5173

# 4. Generate test data
python3 scripts/simulation.py --trades 20 --bots 2
```

### Deployment

One-click deploy on Railway + Cloudflare Pages:

```bash
git push origin main  # Railway auto-deploys the indexer
                      # Cloudflare Pages auto-deploys the web UI
```

See [DEPLOYMENT-RAILWAY.md](docs/DEPLOYMENT-RAILWAY.md) for the full guide.

### Features

| Surface | What you can do |
|---------|----------------|
| **Web dashboard** | Create offers & escrows, check reputation, send messages, vote on jury cases, link Telegram |
| **Telegram bot** | Same features from chat — `/create`, `/offers`, `/reputation`, `/msg`, `/jury` |
| **CLI** | Power-user terminal tool — create, claim, status, reputation, message, receipt |
| **REST API** | 30+ endpoints for programmatic access. See [API.md](docs/API.md) |
| **Covenant templates** | Compile and deploy any DagLock covenant from the UI or API — no SilverScript knowledge needed |

Want to compile a custom escrow or vault without running the compiler? The web UI has a **Compile covenant** tab, and the API has `POST /v1/compile`. Fill in the params, get the bytecode + address back. This is how wallets and bots integrate DagLock without installing any toolchain. See the **Covenant template** action tab in the web UI.

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


## Documentation

| Document | What it covers |
|---|---|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design, component relationships, data flow |
| [PROTOCOL.md](docs/PROTOCOL.md) | Covenant semantics, tx structure, parameter encoding |
| [API.md](docs/API.md) | Indexer REST API reference |
| [DEPLOYMENT.md](docs/DEPLOYMENT.md) | Production deployment + config reference |
| [DEPLOYMENT-RAILWAY.md](docs/DEPLOYMENT-RAILWAY.md) | Railway + Cloudflare Pages deploy guide |
| [HANDOFF.md](HANDOFF.md) | Full walkthrough: deploy, test, iterate |
| [ROADMAP.md](docs/ROADMAP.md) | Phased delivery timeline |
| [SECURITY.md](docs/SECURITY.md) | Threat model, audit checklist, bug bounty |
| [KRC20-TESTNET.md](docs/KRC20-TESTNET.md) | KRC-20 testnet deployment guide |
| [API.md](docs/API.md) | REST API reference |
| [PROTOCOL.md](docs/PROTOCOL.md) | Covenant semantics and transaction structure |
| [WIKI](docs/wiki/_index.md) | AI-optimized codebase map |

---

## Roadmap

| Phase | Status | What |
|-------|--------|------|
| 0 | ✅ | KAS + KRC-20 + Arbiter covenants written, compiled, tested |
| 1 | ✅ | Indexer with REST API, offers board, reputation system |
| 2 | ✅ | Telegram bot, CLI tool, encrypted messaging |
| 3 | ✅ | Web dashboard with full action UI |
| 4 | ⏳ | KRC-20 community launch (NACHO, KASPY) |
| 5 | 🔜 | Mainnet deployment (Toccata hard fork: June 5, 2026) |

See [ROADMAP.md](docs/ROADMAP.md) for the full timeline.

## License

GNU Affero General Public License v3.0 — see [LICENSE](LICENSE).
