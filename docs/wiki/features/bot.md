# Bot

**Source**: `bot/src/`  **Updated**: `2026-06-16`  (2 files — 16 commands + 4-step /create wizard)

## What it does
Telegram bot (`@DagLock_bot`) for DagLock escrow operations. Meet Kaspa users where they are — Telegram. Uses grammY framework, communicates with indexer REST API.

## Architecture
```
index.js (bot entry + command handlers)
    
     lib/api.js     indexer REST API (HTTP)
```

## Key functions / components
| Name | Kind | File:Line | Purpose |
|------|------|-----------|---------|
| `index.js` | entry | `bot/src/index.js` | Bot initialization + command registration (16 commands) |
| `lib/api.js` | module | `bot/src/lib/api.js` | REST API client (escrows, offers, vaults, messages, swaps, reputation, receipts) |

## Commands
| Command | Description |
|---------|-------------|
| `/start` | Welcome + deep link handling |
| `/setaddress` | Set your Kaspa address |
| `/create` | 4-step wizard: amount, counterparty, timeout, dispute mode |
| `/claim <id>` | Claim a pending escrow |
| `/list` | List your escrows |
| `/offers` | Browse open offers with inline keyboard |
| `/swap <id> <hex>` | Atomic swap settle via preimage |
| `/vaults` | List your time-locked vaults |
| `/reputation <address>` | Check counterparty stats |
| `/receipt <id>` | Export settlement receipt |
| `/status <id>` | Check escrow lifecycle state |
| `/dispute <id> <reason>` | Dispute an escrow |
| `/cancel <id>` | Cancel an escrow |
| `/msg <id> <text>` | Send message on an escrow |
| `/messages <id>` | Read escrow thread |
| `/help` | All commands |

## Data flow
1. User sends command to `@DagLock_bot`
2. Bot parses command → calls indexer REST API
3. Bot responds with formatted data / inline keyboards
4. For signing: bot sends trade link deep link to KasWare/Kaspium

## Edge cases & gotchas
- Bot NEVER sees private keys — unsigned tx only
- Trade links: `https://t.me/DagLock_bot?start=claim_<escrow_id>`
- Node.js runtime — not Rust like other components
- grammY framework (not telegraf)

## Testing strategy
| Aspect | Approach |
|--------|----------|
| Unit tests | API client mocking |
| Integration tests | Telegram Bot API test mode |
| Run command | `cd bot && npm test` |

## Dependencies
| Depends on | For |
|------------|-----|
| grammY | Telegram Bot API |
| Node.js | Runtime |
| indexer REST API | Backend |

## Consumed by
| Consumer | How |
|----------|-----|
| Telegram users | Direct chat |

## Related domains
| Domain | Doc | Relationship |
|--------|-----|--------------|
| indexer | `features/indexer.md` | REST API backend |
| contracts | `features/contracts.md` | Covenant knowledge |

---

## Audit Status (2026-06-16) — All 3 bot items resolved

| ID | Issue | Status | Fix |
|----|-------|--------|-----|
| **S6** | Bot stores addresses in plaintext /tmp | ✅ Fixed | AES-256-GCM encryption with `BOT_ENCRYPTION_KEY` env var. Entire JSON encrypted before write, decrypted on read. |
| **U4** | Bot `/create` redirects to web | ✅ Fixed | 4-step grammY conversation wizard: amount → counterparty → timeout → dispute mode → unsigned tx → deep link. |
| **Q8** | Bot API no retry/backoff | ✅ Fixed | 3 attempts, exponential backoff (1s/2s/4s) on 5xx/timeout. 10s `AbortController` per attempt. |
| **A6** | Bot is Node.js while rest is Rust | ❌ Open | Long-term: rewrite in Rust (teloxide/grammy-rs). Out of scope for mainnet. |

### Dependencies

- `BOT_ENCRYPTION_KEY` must be set in production env (generate with `openssl rand -base64 32`)
- Indexer `/v1/compile` endpoint (already exists)
- KasWare/Kaspium deep link format for signing

