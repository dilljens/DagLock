# bot

Telegram bot (`@DagLock_bot`) for DagLock escrow operations. Meet Kaspa users where they are — Telegram. Uses grammY framework, communicates with indexer REST API.

## Rules & Conventions

- ****S6**: Bot stores addresses in plaintext /tmp**
  - Status: ✅ Fixed | Domain: bot
- ****U4**: Bot `/create` redirects to web**
  - Status: ✅ Fixed | Domain: bot
- ****Q8**: Bot API no retry/backoff**
  - Status: ✅ Fixed | Domain: bot
- ****A6**: Bot is Node.js while rest is Rust**
  - Status: ❌ Open | Domain: bot

---
*Confidence: 0.95 · Last updated: 6/17/2026*