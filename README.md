# DagLock 🔒⛓️

> ⚠️ **Testnet phase.** DagLock is deployed on Kaspa Testnet 12 for testing. Covenants activate on mainnet with the Toccata hard fork (June 5, 2026). Do not send real KAS yet.

**Trustless escrow & atomic swaps on Kaspa L1 via SilverScript covenants.**

Lock assets directly into Kaspa's BlockDAG state. Release them only when cryptographic conditions are met. No intermediaries. No admin keys. No custodial risk.

---

## For Users

**Try it now:**

- **Web app:** Visit [daglock.com](https://daglock.com) — no installation needed
- **Telegram bot:** Message [@DagLockBot](https://t.me/DagLockBot) on Telegram — see [BOT-README.md](docs/BOT-README.md) for commands
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

### Features

| Surface | What you can do |
|---------|----------------|
| **Web dashboard** | Create offers & escrows, check reputation, send messages, vote on jury cases, link Telegram |
| **Telegram bot** | Same features from chat — `/create`, `/offers`, `/reputation`, `/msg`, `/jury` |
| **CLI** | Power-user terminal tool — create, claim, status, reputation, message, receipt |
| **REST API** | 30+ endpoints for programmatic access. See [API.md](docs/API.md) |
| **Covenant templates** | Compile and deploy any DagLock covenant from the UI or API — no SilverScript knowledge needed |

---

## Covenant Templates

DagLock lets you compile and deploy covenants without running the SilverScript compiler yourself. This is the main way wallets, bots, and other applications integrate DagLock.

| Template | What it does | Use case |
|----------|-------------|----------|
| **daglock** | KAS escrow: buyer+seller release, timeout refund, atomic swap | Standard OTC trades |
| **daglock_arbiter** | KAS escrow + optional mediator or jury resolution | High-value trades with dispute protection |
| **daglock_vault** | Time-locked self-custody vault (withdraw after timeout) | Personal savings, inheritance, cold storage |

**From the web UI:** Open the **Compile covenant** tab in the Actions panel. Pick a template, fill in the params, click Compile. You get the compiled bytecode and deploy address immediately — no terminal needed.

**From the API:**
```bash
curl -X POST https://api.daglock.io/v1/compile \
  -H "Content-Type: application/json" \
  -d '{"template":"daglock_vault","params":{"owner_key":"<64 hex chars>","timeout":"2000000000"}}'
```
Returns the compiled script, template hash, ABI, and prefix/suffix for UTXO detection.

**In your own code:**
```rust
use daglock_contracts::compile_daglock_vault;
let compiled = compile_daglock_vault(&owner_key, timeout);
```

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
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design, component relationships, data flow |
| [PROTOCOL.md](docs/PROTOCOL.md) | Covenant semantics, tx structure, parameter encoding |
| [API.md](docs/API.md) | Indexer REST API reference (30+ endpoints) |
| [ROADMAP.md](docs/ROADMAP.md) | Phased delivery timeline |
| [SECURITY.md](docs/SECURITY.md) | Threat model, audit checklist |
| [KRC20-TESTNET.md](docs/KRC20-TESTNET.md) | KRC-20 testnet deployment guide |
| [WIKI](docs/wiki/_index.md) | AI-optimized codebase map |

## Integrations

| What | How |
|------|-----|
| **KasWare wallet** | Connect via KasWare browser extension — web UI detects it automatically |
| **Kaspium mobile** | Scan QR codes from the web UI to sign transactions |
| **Custom wallets** | POST /v1/compile returns bytecode + address. Submit via any Kaspa transaction builder |
| **KRC-20 tokens** | Deploy KRC-20 escrows following [KRC20-TESTNET.md](docs/KRC20-TESTNET.md) |

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
