# indexer

Rust backend serving the DagLock REST API. 60+ endpoints across escrows, offers, vaults, jury, AI mediation, E2E chat, tokens, subscriptions, milestones, multi-party escrow, deposits, invoices, price oracle, analytics, webhooks, and WebSocket real-time updates. SQLite/PostgreSQL via SQLx. MockVerifier (no wRPC node available).

## REST API Endpoints

### Core (6)
`GET /v1/health`, `/v1/network`, `/v1/network/price`, `/v1/network/explorer`, `/v1/fees/estimate`, `/v1/stats`

### Escrows (15)
`GET/POST /v1/escrows`, `GET /v1/escrows/:id`, `GET /v1/escrows/:id/lock-status`, `POST /v1/escrows/:id/settle`, `POST /v1/escrows/:id/refund`, `POST /v1/escrows/:id/dispute`, `POST /v1/escrows/:id/cancel`, `POST /v1/escrows/:id/swap`, `POST /v1/escrows/:id/auto-settle`, `GET/POST /v1/escrows/:id/evidence`, `GET /v1/escrows/export`, `GET/POST /v1/escrows/:id/feedback`

### Offers + Negotiation (7)
`GET/POST /v1/offers`, `POST /v1/offers/:id/accept`, `POST /v1/offers/:id/cancel`, `POST /v1/offers/:id/counter`, `GET /v1/offers/:id/counters`, `POST /v1/counteroffers/:id/accept`, `POST /v1/counteroffers/:id/decline`

### Vaults (9)
`GET/POST /v1/vaults`, `GET /v1/vaults/:id`, `POST /v1/vaults/:id/withdraw`, `POST /v1/vaults/:id/transfer`, `POST /v1/vaults/:id/sweep`, `POST /v1/vaults/:id/relock`, `POST /v1/vaults/:id/early-exit`, `POST /v1/vaults/:id/heir-withdraw`

### Subscriptions (5)
`GET/POST /v1/subscriptions`, `GET /v1/subscriptions/:id`, `POST /v1/subscriptions/:id/cancel`, `POST /v1/subscriptions/:id/draw`

### Milestones (8)
`GET/POST /v1/milestones`, `GET /v1/milestones/:id`, `POST /v1/milestones/:id/release`, `POST /v1/milestones/:id/approve`, `POST /v1/milestones/:id/dispute`, `POST /v1/milestones/:id/refund`, `POST /v1/milestones/:id/complete`

### Multi-Party Escrow (6)
`GET/POST /v1/multi-escrows`, `GET /v1/multi-escrows/:id`, `POST /v1/multi-escrows/:id/sign`, `POST /v1/multi-escrows/:id/refund`, `POST /v1/multi-escrows/:id/swap`

### Jury (9)
`POST /v1/jury/register`, `POST /v1/jury/unregister`, `GET /v1/jury/cases`, `GET /v1/jury/cases/active/:address`, `GET /v1/jury/cases/:id`, `POST /v1/jury/cases/:id/vote`, `GET /v1/jury/candidates`, `GET /v1/jury/cases/:id/evidence`, `POST /v1/jury/cases/:id/evidence/clear`

### AI Mediation (3)
`POST /v1/escrows/:id/mediate`, `GET /v1/escrows/:id/mediate`, `POST /v1/escrows/:id/mediate/:party/accept`

### Messaging + Reveal (4)
`GET/POST /v1/escrows/:id/messages`, `POST /v1/escrows/:id/messages/reveal`, `GET /v1/escrows/:id/messages/anchors`

### Tokens (7)
`GET /v1/tokens`, `GET /v1/tokens/registered`, `POST /v1/tokens/deploy`, `GET /v1/tokens/:ticker`, `PATCH /v1/tokens/:ticker`, `GET /v1/tokens/:ticker/chart`

### Invoices (3)
`GET/POST /v1/invoices`, `GET /v1/invoices/:id`

### Payments (3)
`POST /v1/pay`, `GET /v1/pay/:session_id`, `POST /v1/pay/:session_id/fund`

### Deposits (5)
`POST /v1/escrows/:id/deposit`, `GET /v1/escrows/:id/deposit`, `POST /v1/escrows/:id/deposit/release`, `POST /v1/escrows/:id/deposit/forfeit`, `POST /v1/deposits/sweep`

### Analytics (2)
`GET /v1/stats/daily`, `GET /v1/stats/summary`

### Price (3)
`GET /v1/network/price/history`, `GET/POST /v1/price-alerts`, `DELETE /v1/price-alerts/:id`

### Other — Reports, Blocks, Identity, Notifications, Apps, Webhooks, Compile, Receipts, Vouches, Reputation.

## Key Services
| Service | Module | Purpose |
|---------|--------|---------|
| EscrowService | `services/escrow_service.rs` | Escrow lifecycle with auth + verification |
| EscrowVerifier | `verification.rs` | UTXO verification (WrpcVerifier/MockVerifier) |
| AiMediator | `services/ai_mediator.rs` | DeepSeek V4 Flash API integration |
| AnchorService | `services/anchor.rs` | On-chain message hash anchoring |
| EmailService | `services/email.rs` | SMTP notifications |
| PriceOracle | `services/price_oracle.rs` | CoinGecko KAS/USD polling |
| PriceAlerts | `services/price_alerts.rs` | Threshold-based price alerts |
| WebhookService | `services/webhooks.rs` | Event dispatch with retry |

## Background Tasks (spawned from `main.rs`)

11 background tasks: anchor flush, wRPC listener, offer reconciliation, mediation escalation, dispute escalation, evidence wipe, auto-settle, daily stats, price oracle, price alerts, deposit sweep.

## Config Flags
`--no-wrpc`, `--wrpc-url`, `--mock-auth`, `--mock-chat-sig`, `--network`, `--treasury-pubkey`, `--admin-token`, `--ai-mediator-api-key`, `--ai-mediator-model`, `--anchor-wallet-key`, `--auto-sweep-vaults`, `--auto-settle-escrows`, `--auto-escalate-disputes`, `--auto-sweep-deposits`, `--price-alerts-enabled`, `--stats-interval-seconds`, `--evidence-auto-wipe-hours`

---
*Confidence: 0.95 · Last updated: 7/7/2026*
