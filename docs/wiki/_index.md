# DagLock — Architecture Overview

**Key Rules:** Rule #1: Never `.unwrap()` outside `#[cfg(test)]` · Rule #2: Never hardcode addresses/keys in covenant source · Rule #3: Never skip fee validation in release/swap paths · Rule #4: Never expose private keys in bot/CLI/WASM · Rule #5: Never change the fee denominator (200) without updating all paths · Rule #6: Never use non-atomic updates for lifecycle transitions · Rule #7: Never skip address validation on create

## Domains
| Domain | Description |
|--------|-------------|
| contracts | 12 SilverScript covenants (KAS, KRC-20, Arbiter, Advanced, Vault, VaultMultisig, VaultSoftlock, Subscription, Milestone, Multi, Deposit, Reputation) compiled via `silverscript-lang`. Rusty-kaspa SDK v2.0.1 (Toccata). |
| indexer | Rust backend with 60+ REST endpoints. Handles escrow lifecycle, offers, reputation, vaults, jury, AI mediation, E2E chat, anchor service, payment sessions, price oracle, analytics, webhooks. SQLite/PostgreSQL via SQLx. |
| cli | Command-line tool for escrow operations. Connects to indexer API, assembles unsigned txs for kaspawallet/KasWare signing. |
| wasm-sdk | Browser WASM SDK for covenant compilation and tx assembly. |
| web | React + Vite dashboard at `/`. 17+ pages: escrows, vaults, swap wizard, tokens, subscriptions, stats, security deep-dive, merchant, blog, jury, settings, help. Vitest + RTL tests. |
| bot | Telegram bot (`@DagLock_bot`) with 50+ commands using grammY. Full lifecycle: create, settle, refund, swap, dispute, invoices, subscriptions, KRC-20, deposits. |
| simulation | `scripts/simulation.py` — Mass trade generation + reputation testing |

## For humans
| I want to... | Read |
|-------------|------|
| Know what NOT to do | `_standards.md` § Rules |
| Understand a module | `features/<domain>.md` |
| Find a term | `_glossary.md` |

## Key Documents
| Document | What it covers |
|----------|---------------|
| `_glossary.md` | Project vocabulary |
| `_standards.md` | Rules, conventions, patterns |
| `docs/PENDING.md` | Planned features with effort estimates |
| `docs/security-audit.md` | Full security audit results (July 2026) |
| `docs/local-testnet-node.md` | Local node setup plan (after RAM upgrade) |
| `.opencode/plans/cross-chain-btc-eth-detailed.md` | Cross-chain swap architecture (deferred) |
| `.opencode/plans/marketing-plan.md` | Community + YouTuber outreach plan |

## Audit Log

| Date | Change |
|------|--------|
| 2026-07-07 | **Full feature completion**: Subscription web UI, KRC-20 bot commands, vault API endpoints, MediationPanel wiring, deal type backend, on-chain anchoring UI, DeepSeek AI switch |
| 2026-07-07 | **Feature completeness audit**: 20 features checked across 4 layers. 12 partial/missing features completed. |
| 2026-07-07 | **CI Green + E2E**: 40 Playwright tests, 300+ Rust, 44 web, 22 bot. All passing. |
| 2026-07-07 | **KRC-20 launchpad**: Compile endpoint fixed (was 501), deploy tx verification, proper TS types |
| 2026-07-07 | **Blog infrastructure**: 4 blog posts at /blog, sidebar nav, content CSS |
| 2026-07-07 | **Security audit**: 3 critical + 2 high vulnerabilities found and fixed. Report at docs/security-audit.md |
| 2026-07-07 | **Marketing plan**: 4-phase plan (community → YouTubers → ecosystem → launch) at .opencode/plans/marketing-plan.md |
| 2026-07-06 | **8 new feature tracks**: Auto-release, milestones, subscriptions, multi-party, deposits, dispute escalation, AI mediator, covenant upgrades (dust/split/emergency/vault). All fully implemented. |
| 2026-07-06 | **Kasia chat**: E2E encrypted messaging (Ed25519 keys, X25519 ECDH, NaCl secretbox), on-chain hash anchoring, dispute reveal flow, recovery sheets |
| 2026-07-06 | **OfficeForge gap analysis**: DAA vault timing, MIN_OUT dust protection, AI mediator, emergency signatureless timeout, dispute split, interactive demo, deal presets |
| 2026-07-06 | **Analytics dashboard**: /stats page, daily stats table, price history, price alerts |
| 2026-07-06 | **Trading bot API**: Rate limit tiers (Free/Pro/Whale), admin upgrade endpoint |
| 2026-07-06 | **Testnet fixes**: testnet-10/12 → testnet-11 (testnet-12 never deployed) |
| 2026-07-03 | **Covenant upgrades**: `daglock_advanced.sil`, `daglock_subscription.sil` |
| 2026-07-02 | **KRC-20 launchpad**: Token registration UI at /tokens/create |
| 2026-07-01 | **KRC-20 token dashboard**: /tokens with price charts, trade history |
| 2026-06-30 | **Atomic swap wizard**: 6-step guided UI with deep links |
| 2026-06-29 | **Usability sprint**: Fee calculator, onboarding, blocklist, testnet-10 migration |
| 2026-06-26 | **Invoice feature**: Escrow-based invoicing |
| 2026-06-23 | **S3 ICC fix**: KCC-20 input ownership validated |
| 2026-06-18 | **SDK migration**: rusty-kaspa tn12 → v2.0.1 |

> Knowledge graph last synced: 7/7/2026