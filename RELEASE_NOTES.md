# Release Notes

## v0.1.0 — 2026-06-17 — Testnet Launch

**Architecture:**
- Indexer, kaspad, Telegram bot, and trade bot on single OVHcloud VPS
- Web UI served via Cloudflare Pages at daglock.com
- API at api.daglock.com (Cloudflare-proxied)
- All services migrated off Railway

**Covenants (6 templates):**
- KAS Escrow (daglock.sil) — release, swap, refund entrypoints
- KRC-20 Escrow (daglock_krc20.sil) — token escrow with ICC pattern
- Arbiter (daglock_arbiter.sil) — mediator/jury dispute resolution
- Vault Standard (daglock_vault.sil) — time-locked self-custody
- Vault Softlock (daglock_vault_softlock.sil) — password-recoverable
- Vault Multisig (daglock_vault_multisig.sil) — 2-of-3 multisig

**Indexer:**
- 16 REST API handler modules (apps, compile, escrows, evidence, identity, jury, messages, network, offers, receipts, reputation, status, swap, vaults, vouches, webhooks)
- Schnorr signature verification with replay protection
- Rate limiting: 30 req/min default, 300 req/min with API key
- Daily creation caps: max 50 escrows/offers per address
- WebSocket real-time event broadcast
- SQLite (dev) / Postgres (prod) via sqlx

**Web UI (8 pages):**
- Dashboard with feature cards, How It Works, stats, quick actions
- Offers board with type badges (KAS Escrow / Atomic Swap)
- Escrow lifecycle (create, settle, refund, dispute, cancel)
- Atomic swap wizard (generate hash, submit preimage)
- Vault management with type selector (time-locked, beneficiary, multisig)
- On-chain reputation display with explanation
- Jury system (register, vote, candidates)
- Developer docs page (API, CLI, Bot, Integrate)
- Skeleton loading, page transitions, Radix UI, TanStack Query, WebSocket

**Telegram Bot (16 commands):**
- /create 4-step wizard, /swap, /vaults, /reputation, /dispute, /evidence, /msg
- AES-256-GCM encrypted user data storage
- 3-attempt retry with exponential backoff

**Security:**
- All 7 critical/high audit findings fixed
- 0.5% escrow fee and 0.1% vault fee enforced by covenants
- Production hardening: request body limits, rate tiers, SQLite pool tuning
- Error boundaries on every web page

**Tests:** 211 Rust + 40 Web = 251 total
**CI:** 5 jobs (Check & Lint, Test, Build, Web, Bot)
