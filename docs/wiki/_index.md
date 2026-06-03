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
| Authentication (real Schnorr sigs) | `indexer/src/auth.rs` |
| UTXO verification | `indexer/src/verification.rs` |
| Message encryption (AES-256-GCM) | `indexer/src/crypto.rs` |
| wRPC listener (stub) | `indexer/src/listener.rs` |
| CLI entry | `cli/src/main.rs` |
| WASM SDK | `wasm-sdk/src/lib.rs` |
| Web UI entry | `web/src/App.tsx` |
| Bot entry | `bot/src/index.js` |
| Reputation simulation | `scripts/simulation.py` |
| Dockerfile | `Dockerfile` |

### API modules (indexer/src/api/)
| Module | Purpose |
|--------|---------|
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
| indexer | `cargo test -p daglock-indexer` |
| cli | `cargo test -p daglock-cli` |
| wasm-sdk | `cargo test -p daglock-wasm-sdk` |
| web | `cd web && npm test` |
| bot | `cd bot && npm test` |
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

## Topology
```
DagLock/
  contracts/          (SilverScript covenants + Rust compilation)
    src/daglock.sil          -- silverscript-lang (compile)
    src/daglock_krc20.sil    -- silverscript-lang (compile)
    src/daglock_arbiter.sil  -- daglock.sil + arbiterKey param
    src/lib.rs               -- blake2b_simd, silverscript-lang
    tests/                   -- TxScriptEngine execution tests

  indexer/            (Rust backend)
    src/main.rs              -- api, config, crypto, db, listener
    src/config.rs            -- argparse + production flags (--allow-mainnet, --cors-origin)
    src/crypto.rs            -- AES-256-GCM message encryption
    src/auth.rs              -- Schnorr signature verification (real, not mock)
    src/verification.rs      -- UTXO verification (MockVerifier)
    src/websocket.rs         -- WebSocket broadcast
    src/types.rs             -- serde (all shared types)
    src/db/schema.rs         -- sqlx, migrations/ (8+ files)
    src/db/queries.rs        -- all SQL queries + vouch/mediator scoring
    src/api/                 -- 10 handler modules
    tests/                   -- integration + edge case tests

  cli/                (CLI tool)
    src/main.rs              -- clap, commands/*
    src/config.rs            -- ~/.daglock/config.toml
    src/tx.rs                -- kas_to_sompi, tx assembly
    src/commands/            -- 7 command files (create, claim, offer, status, rep, receipt, message)

  wasm-sdk/           (Browser SDK)
    src/lib.rs               -- wasm-bindgen, compile_escrow

  web/                (React dashboard)
    src/App.tsx              -- all forms + panels
    src/api.ts               -- REST API client
    src/styles.css           -- dark-theme CSS

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
| Add web page | `web/src/App.tsx` (routes) + new component |
| Add bot command | `bot/src/index.js` (command handler) |
| Change fee percentage | `contracts/src/daglock.sil` (hardcoded `inputValue / 200`) |
| Change template hash logic | `contracts/src/lib.rs` (`template_parts_and_hash()`) |
| Add authentication | `indexer/src/auth.rs` (SignatureVerifier trait) |
| Add UTXO verification | `indexer/src/verification.rs` (EscrowVerifier trait) |
| Change message encryption | `indexer/src/crypto.rs` (AES-256-GCM, DAGLOCK_MESSAGE_KEY) |
| Change jury rules | `indexer/src/api/jury.rs` |
| Run reputation sim | `scripts/simulation.py` |

## Domain registry
| Domain | Doc | Files | Purpose |
|--------|-----|-------|---------|
| contracts | `features/contracts.md` | 6 | SilverScript covenants + compilation + tests |
| indexer | `features/indexer.md` | 24 | REST API, DB, auth, crypto, verification, jury, messages |
| cli | `features/cli.md` | 8 | Command-line power-user tool |
| wasm-sdk | `features/wasm-sdk.md` | 1 | Browser/JS SDK for tx assembly |
| web | `features/web.md` | 3 | React + Vite dashboard |
| bot | `features/bot.md` | 2 | Telegram bot (Node.js) |

## Existing docs
- README.md -- project overview, setup
- AGENTS.md -- project context for AI agents
- docs/API.md -- REST API reference
- docs/ARCHITECTURE.md -- system architecture
- docs/PROTOCOL.md -- covenant protocol spec
- docs/ROADMAP.md -- development roadmap
- docs/SECURITY.md -- threat model + audit checklist
