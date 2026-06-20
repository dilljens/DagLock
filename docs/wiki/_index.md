# DagLock — Architecture Overview

**Key Rules:** Rule #1: Never `.unwrap()` outside `#[cfg(test)]` · Rule #2: Never hardcode addresses/keys in covenant source · Rule #3: Never skip fee validation in release/swap paths · Rule #4: Never expose private keys in bot/CLI/WASM · Rule #5: Never change the fee denominator (200) without updating all paths · Rule #6: Never use non-atomic updates for lifecycle transitions · Rule #7: Never skip address validation on create

## Domains
| Domain | Description |
|--------|-------------|
| -------- | See ----- |
| contracts | SilverScript covenants for trustless escrow and atomic swaps on Kaspa L1. Six covenant files (KAS, KRC-20, Arbiter, Vault, VaultSoftlock, VaultMultisig) compiled via `silverscript-lang`. Rusty-kaspa SDK: v2.0.1 (Toccata). The `lib.rs` crate provides a Rust API for compilation and template hash extraction. |
| indexer | Rust backend serving the DagLock REST API. Handles escrow lifecycle (create, settle, refund, dispute), offer board, reputation, vaults, jury, encrypted messaging, app registration, webhook dispatch, and WebSocket real-time updates. Uses SQLite or PostgreSQL via SQLx. |
| cli | Command-line power-user tool for DagLock escrow operations. Connects to the indexer REST API for queries and assembles unsigned transactions for signing with kaspawallet or KasWare. |
| wasm-sdk | Browser/JavaScript SDK for assembling DagLock transactions in the web UI. Compiles covenants and constructs unsigned transactions that can be signed via KasWare browser extension. |
| web | React + Vite dashboard for browser-based users. Provides escrow creation, claiming, offer board, and reputation views. Communicates with the indexer REST API. Uses Vitest + React Testing Library for component tests, Biome for lint. |
| bot | Telegram bot (`@DagLock_bot`) for DagLock escrow operations. Meet Kaspa users where they are — Telegram. Uses grammY framework, communicates with indexer REST API. |
| simulation | See `scripts/simulation.py` — Mass trade generation + reputation testing |

## For humans
| I want to... | Read |
|-------------|------|
| Know what NOT to do | `_standards.md` § Rules |
| Understand a module | `features/<domain>.md` |
| Find a term | `_glossary.md` |

## Code Graph
Use codebase-memory-mcp tools for structural code queries:
- `search_graph` — find functions, classes, routes by name
- `trace_path` — trace who calls what
- `get_architecture` — high-level project structure
- `/knowledge:query` — unified search across code + concepts + decisions

## Audit Log
| Date | Change |
|------|--------|
| 2026-06-18 | SDK migration: rusty-kaspa `tn12` → `v2.0.1` (Toccata mainnet). All 215 tests pass, template hashes unchanged. Add `--allow-mainnet` flag. |

> Knowledge graph last synced: 6/18/2026
> Run `/knowledge:sync` to refresh from codebase