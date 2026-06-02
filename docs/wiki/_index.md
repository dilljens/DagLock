# DagLock — Architecture Overview

**Build**: `cargo build --workspace`  **Test**: `cargo test --workspace`  **Setup**: `rustup default stable`

## Quick reference

### Key files
| Purpose | File |
|---------|------|
| KAS escrow covenant | `contracts/src/daglock.sil` |
| KRC-20 escrow covenant | `contracts/src/daglock_krc20.sil` |
| Covenant compilation | `contracts/src/lib.rs` |
| Indexer entry | `indexer/src/main.rs` |
| REST API routes | `indexer/src/api/mod.rs` |
| DB schema + migrations | `indexer/src/db/schema.rs` |
| CLI entry | `cli/src/main.rs` |
| WASM SDK | `wasm-sdk/src/lib.rs` |
| Web UI entry | `web/src/App.tsx` |
| Bot entry | `bot/src/index.js` |

### Test per domain
| Domain | Run |
|--------|-----|
| contracts | `cargo test -p daglock-contracts` |
| indexer | `cargo test -p daglock-indexer` |
| cli | `cargo test -p daglock-cli` |
| wasm-sdk | `cargo test -p daglock-wasm-sdk` |
| web | `cd web && npm test` |
| bot | `cd bot && npm test` |

### Domain one-liners
| Domain | Doc |
|--------|-----|
| contracts | `features/contracts.md` — SilverScript covenants + compilation |
| indexer | `features/indexer.md` — Rust backend: REST API, DB, wRPC listener |
| cli | `features/cli.md` — Command-line power-user tool |
| wasm-sdk | `features/wasm-sdk.md` — Browser/JS SDK for tx assembly |
| web | `features/web.md` — React + Vite dashboard |
| bot | `features/bot.md` — Telegram bot (Node.js) |

## Navigation

### For humans
| I want to... | Read |
|-------------|------|
| Know what NOT to do | `_standards.md` § Rules |
| Know how to write new code | `_standards.md` § Practices |
| Match existing conventions | `_standards.md` § Patterns |
| Understand a module | `features/<domain>.md` |

### For AI agents
| Task | Start here |
|------|------------|
| **Cold start (zero context)** | `_glossary.md` → `_index.md` → `_standards.md` § Rules → § Practices → target `features/<domain>.md` → § Patterns |
| Add a feature | `_standards.md` § Rules → § Practices → target `features/<domain>.md` → § Patterns |
| Fix a bug | `features/<domain>.md` (edge cases) → `_standards.md` § Rules |
| Refactor a module | `features/<domain>.md` (deps + consumers) → `_standards.md` § Patterns |
| Navigate unfamiliar code | Read domain one-liner → open `features/<domain>.md` |
| Write a test | `features/<domain>.md` → `_standards.md` § Practices (Testing) |

## Entry points
| Trigger | File | Description |
|----------|------|-------------|
| `cargo run -p daglock-indexer` | `indexer/src/main.rs` | Start REST API + optional wRPC listener |
| `cargo run -p daglock-cli -- <cmd>` | `cli/src/main.rs` | CLI subcommand dispatch |
| `cd web && npm run dev` | `web/src/App.tsx` | Vite dev server |
| `cd bot && node src/index.js` | `bot/src/index.js` | Telegram bot |

## Topology
```
DagLock/
├── contracts/          (SilverScript covenants + Rust compilation)
│   ├── src/daglock.sil        ──▶ silverscript-lang (compile)
│   ├── src/daglock_krc20.sil  ──▶ silverscript-lang (compile)
│   └── src/lib.rs             ──▶ blake2b_simd, silverscript-lang
│
├── indexer/            (Rust backend)
│   ├── src/main.rs            ──▶ api, db, config, listener
│   ├── src/api/mod.rs         ──▶ axum, sqlx, escrows/offers/reputation/receipts
│   ├── src/db/schema.rs       ──▶ sqlx, migrations/*.sql
│   ├── src/listener.rs        ──▶ db::queries (reconciliation)
│   └── src/types.rs           ──▶ serde (shared types)
│
├── cli/                (CLI tool)
│   ├── src/main.rs            ──▶ clap, commands/*
│   ├── src/tx.rs              ──▶ (transaction assembly)
│   └── src/commands/*.rs      ──▶ indexer REST API (HTTP)
│
├── wasm-sdk/           (Browser SDK)
│   └── src/lib.rs             ──▶ wasm-bindgen, js-sys, contracts
│
├── web/                (React dashboard)
│   └── src/App.tsx            ──▶ api.ts (indexer REST)
│
└── bot/                (Telegram bot)
    └── src/index.js           ──▶ lib/api.js (indexer REST)
```

## "Need to change X? Start here"
| Change | Look at |
|--------|---------|
| Escrow spending rules | `contracts/src/daglock.sil` |
| KRC-20 token escrow | `contracts/src/daglock_krc20.sil` |
| Covenant compilation API | `contracts/src/lib.rs` |
| Add REST endpoint | `indexer/src/api/mod.rs` (router) + new handler module |
| Add DB table/column | `indexer/src/db/schema.rs` (migration) + `indexer/src/db/queries.rs` |
| Add CLI subcommand | `cli/src/main.rs` (enum) + new `cli/src/commands/*.rs` |
| Add web page | `web/src/App.tsx` (routes) + new component |
| Add bot command | `bot/src/index.js` (command handler) |
| Change fee percentage | `contracts/src/daglock.sil` (hardcoded `inputValue / 200`) |
| Change template hash logic | `contracts/src/lib.rs` (`template_parts_and_hash()`) |

## Domain registry
| Domain | Doc | Files | Purpose |
|--------|-----|-------|---------|
| contracts | `features/contracts.md` | 4 | SilverScript covenants + compilation + tests |
| indexer | `features/indexer.md` | 10 | REST API, DB, wRPC listener, types |
| cli | `features/cli.md` | 7 | Command-line power-user tool |
| wasm-sdk | `features/wasm-sdk.md` | 1 | Browser/JS SDK for tx assembly |
| web | `features/web.md` | 4 | React + Vite dashboard |
| bot | `features/bot.md` | 2 | Telegram bot (Node.js) |

## Existing docs
- [README.md](../../README.md) — project overview, setup
- [AGENTS.md](../../AGENTS.md) — project context for AI agents
- [docs/API.md](../API.md) — REST API reference
- [docs/ARCHITECTURE.md](../ARCHITECTURE.md) — system architecture
- [docs/PROTOCOL.md](../PROTOCOL.md) — covenant protocol spec
- [docs/ROADMAP.md](../ROADMAP.md) — development roadmap
- [docs/SECURITY.md](../SECURITY.md) — threat model + audit checklist
