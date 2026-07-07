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

## Key Documents
| Document | What it covers |
|----------|---------------|
| `_glossary.md` | Project vocabulary |
| `_standards.md` | Rules, conventions, patterns |
| `docs/ecosystem-research.md` | Kaspa ecosystem landscape — who builds what, open space analysis |
| `docs/PENDING.md` | Planned features with effort estimates, deferred until after launch |
| `.opencode/plans/bot-labeling.md` | Plan: bot/exchange reputation labeling with different score algorithms |
| `.opencode/plans/pre-announcement.md` | Pre-launch human checklist (friend test, demo video, drafts) |

## Audit Log
| Date | Change |
|------|--------|
| 2026-06-18 | SDK migration: rusty-kaspa `tn12` → `v2.0.1` (Toccata mainnet). All 215 tests pass, template hashes unchanged. Add `--allow-mainnet` flag. |
| 2026-06-21 | Playwright E2E test suite (40 tests across 3 projects). Bug fix: infinite re-render in MyEscrows. Port conflict resolution across 4 projects. Ecosystem research completed. |
| 2026-06-23 | **S3 ICC fix**: KCC-20 input ownership validated via `readInputStateWithTemplate` in `daglock_krc20.sil`. New template hash: `ae0946e4a9bd4a7585e6bf9135de38083cb11c85`. | 
| 2026-06-23 | **Code quality**: All `.unwrap()` removed from production. `FEE_DENOMINATOR` shared constant (i64). Flaky crypto tests fixed. |
| 2026-06-23 | **VPS hardened**: daglock user, LimitNOFILE=65536. Mainnet binary deployed. 303 tests passing. |
| 2026-06-23 | **wRPC discovery**: Kaspa PNN resolver down (wRPC v2 migration). Mainnet endpoints found at `troy.kaspa.stream` (borsh). Testnet-12 kaspad patched (1-line fix). |
| 2026-06-25-26 | **Security audit fixes**: 6 critical/high findings resolved. |
| 2026-06-26 | **Docs liability fix**: 13 files updated. |
| 2026-06-26 | **CI green**: All 5 jobs passing. |
| 2026-06-26 | **Invoice feature**: Escrow-based invoicing. |
| 2026-06-29 | **Usability sprint**: Fee calculator, explorer links, onboarding, help center, blocklist, trade feedback, testnet-10 migration. |
| 2026-06-29 | **Trade bot + offer expiry**: Bot populates offer board, stale offers auto-cleaned. |
| 2026-06-30 | **Atomic swap wizard**: 6-step guided wizard with deep links. |
| 2026-07-01 | **KRC-20 token dashboard**: `/tokens` with price charts, trade history, token detail. |
| 2026-07-01 | **Bot-native escrow create**: Full Telegram wizard with Kaspium deep links, `/settle`, `/refund`. |
| 2026-07-02 | **Counter-offers**: In-app negotiation with accept/decline. Web + bot. |
| 2026-07-02 | **KRC-20 token launchpad**: Register tokens, bootstrap liquidity. `/tokens/create`. |
| 2026-07-02 | **Escrow memo field**: Notes on every escrow. |
| 2026-07-02 | **CSV export**: One-click download for tax reporting. |
| 2026-07-03 | **Email notifications**: SMTP integration, opt-in per event, `/settings` UI. |
| 2026-07-03 | **Covenant upgrades**: `daglock_advanced.sil` (extendTimeout + swap_partial), `daglock_subscription.sil` (recurring payments). Design docs ready for audit. |

> Knowledge graph last synced: 6/26/2026
> Run `/knowledge:sync` to refresh from codebase