# Roadmap

Forward-looking plan for DagLock. Timelines are approximate and subject to change.

## Current Milestone

**v0.8 — Mainnet Hardening** (target: July 2026)

Preparing for mainnet launch on Kaspa Toccata hard fork. Focus on security hardening, infrastructure reliability, and deployment automation. All pre-mainnet verification gates must pass.

### This milestone

| Feature / Task | Priority | Status | Notes |
|----------------|----------|--------|-------|
| Rotate Cloudflare API token + encryption keys | P1 | ❌ blocked | Needs Cloudflare dashboard access |
| Bot persistent storage (SQLite) | P1 | ✅ done | Replaced /tmp JSON with better-sqlite3 |
| Covenant test coverage (sub, milestone, advanced) | P1 | ✅ done | 18 new execution tests |
| Bot test suite | P2 | ✅ done | 17 new tests (CRUD + commands + migration) |
| Bot `/accept`, `/createoffer`, `/canceloffer` commands | P1 | ✅ done | Full Telegram offer lifecycle |
| Offer memo/deal_type + web UI polish | P2 | ✅ done | Memo field, deal type dropdown, expiry display |
| WebSocket + nginx fixes | P1 | ✅ done | WS upgrade headers, CORS on 429 |
| Rate limiter: 10→60/min + cleanup | P1 | ✅ done | Scalability for many users |
| SQLite optimizations + indexes | P1 | ✅ done | synchronous=NORMAL, busy_timeout, cache_size, 6 new indexes |
| Offers API pagination | P1 | ✅ done | limit/offset, default 50 max 200 |
| Structured logging + TraceLayer | P2 | ✅ done | --log-json flag, request instrumentation |
| Snapshot + property-based tests | P2 | ✅ done | insta, proptest, concurrent access tests |
| Chat signature verification | P1 | ❌ blocked | Needs wRPC via PNN (TN10 wRPC is live) |
| Web UI deploy (Cloudflare token) | P1 | ❌ blocking offers | Token exists in GitHub secrets, deploy ran |
| Secrets rotation | P1 | ❌ blocked | Needs Cloudflare dashboard access |

## Future Milestones

| Milestone | Target | Summary |
|-----------|--------|---------|
| v0.9 — Cross-chain HTLC | August 2026 | BTC/LTC atomic swaps via hash time-locked contracts |
| v1.0 — Mainnet Launch | TBD | Production mainnet deployment with full feature set |
| v1.1 — On-chain Reputation | TBD | Trade history derived from covenant executions |

## Icebox

- **Volume-based fee tiers** — Off-chain rebates for high-volume traders. Covenant stays simple.
- **Counterparty discovery board** — Public listing of open escrow offers within the bot.
- **Mobile app** — Kaspium integration for mobile escrow creation.
- **Analytics dashboard v2** — Advanced charts, volume tracking, whale alerts.
- **Arbitration marketplace** — Let users stake on jury duty for fee rewards.
