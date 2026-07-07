# indexer

Rust backend serving the DagLock REST API. Handles escrow lifecycle (create, settle, refund, dispute), offer board, reputation, vaults, jury, encrypted messaging, app registration, webhook dispatch, and WebSocket real-time updates. Uses SQLite or PostgreSQL via SQLx.

## Infrastructure

| Component | Location | Details |
|-----------|----------|---------|
| Indexer | OVH VPS-2 (8GB RAM) | `testnet-11`, `--no-wrpc` (MockVerifier) |
| Bot | Same VPS | `@DagLock_bot` on Telegram |
| Web UI | Cloudflare Pages | `daglock.com` → `api.daglock.com` |
| kaspad | ❌ Not on VPS | Brief test failed, used `--no-wrpc` instead |
| Gitpod / Laptop | — | kaspad failed to sync behind NAT |

## REST API Endpoints (35+)

### Core
| Method | Route | Description |
|--------|-------|-------------|
| GET | `/v1/health` | Health check |
| GET | `/v1/network` | Network info |
| GET | `/v1/network/price` | KAS/USD price |
| GET | `/v1/network/explorer` | Explorer base URL |
| GET | `/v1/fees/estimate` | Fee estimate |
| GET | `/v1/stats` | Escrow statistics |

### Escrows
| Method | Route | Description |
|--------|-------|-------------|
| GET/POST | `/v1/escrows` | List/create escrows |
| GET | `/v1/escrows/:id` | Get escrow by ID |
| GET | `/v1/escrows/:id/lock-status` | Lock status |
| POST | `/v1/escrows/:id/settle` | Settle |
| POST | `/v1/escrows/:id/refund` | Refund |
| POST | `/v1/escrows/:id/dispute` | Dispute |
| POST | `/v1/escrows/:id/cancel` | Cancel |
| POST | `/v1/escrows/:id/swap` | Atomic swap |
| GET | `/v1/escrows/:id/evidence` | List evidence |
| POST | `/v1/escrows/:id/evidence` | Submit evidence |
| GET | `/v1/escrows/export` | **CSV export** (new) |
| GET/POST | `/v1/escrows/:id/feedback` | **Trade feedback** (new) |

### Offers + Negotiation
| Method | Route | Description |
|--------|-------|-------------|
| GET/POST | `/v1/offers` | List/create offers |
| POST | `/v1/offers/:id/accept` | Accept offer |
| POST | `/v1/offers/:id/cancel` | Cancel offer |
| POST | `/v1/offers/:id/counter` | **Counter-offer** (new) |
| GET | `/v1/offers/:id/counters` | **List counters** (new) |
| POST | `/v1/counteroffers/:id/accept` | **Accept counter** (new) |
| POST | `/v1/counteroffers/:id/decline` | **Decline counter** (new) |

### Notifications
| Method | Route | Description |
|--------|-------|-------------|
| GET/POST | `/v1/notifications` | **Email subscription** (new) |
| POST | `/v1/notifications/verify` | **Verify email** (new) |
| POST | `/v1/notifications/preferences` | **Update prefs** (new) |

### Tokens
| Method | Route | Description |
|--------|-------|-------------|
| GET | `/v1/tokens` | List tokens |
| GET | `/v1/tokens/:ticker` | Token detail |
| GET | `/v1/tokens/:ticker/chart` | Price chart |
| POST | `/v1/tokens/deploy` | **Register token** (new) |
| PATCH | `/v1/tokens/:ticker` | **Update token status** (new) |

### Blocklist
| Method | Route | Description |
|--------|-------|-------------|
| GET/POST | `/v1/blocks` | List/block users |
| POST | `/v1/blocks/:id` | Unblock |

### Vaults, Jury, Messaging, Identity, Apps, Webhooks — unchanged.

## New Schema Migrations
- 019: blocked_users
- 020: user_reports
- 021: trade_feedback
- 022: counteroffers
- 025: token_registry
- 026: email_subscriptions

## Email Service

SMTP-based email notifications for escrow events. Configured via env vars:
- `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASS`, `NOTIFICATION_FROM`

Currently uses a debug SMTP server on 127.0.0.1:1025 that logs to `/var/log/daglock-emails.log`.

---

*Confidence: 0.95 · Last updated: 7/3/2026*
