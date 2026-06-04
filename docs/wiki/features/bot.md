# Bot

**Source**: `bot/src/`  **Updated**: `2026-06-04`  (2 files)

## What it does
Telegram bot (`@DagLockBot`) for DagLock escrow operations. Meet Kaspa users where they are — Telegram. Uses grammY framework, communicates with indexer REST API.

## Architecture
```
index.js (bot entry + command handlers)
    │
    └── lib/api.js    ──▶ indexer REST API (HTTP)
```

## Key functions / components
| Name | Kind | File:Line | Purpose |
|------|------|-----------|---------|
| `index.js` | entry | `bot/src/index.js` | Bot initialization + command registration |
| `lib/api.js` | module | `bot/src/lib/api.js` | REST API client functions |

## Commands
| Command | Description |
|---------|-------------|
| `/start` | Welcome + deep link handling |
| `/create` | Open web dashboard for creation |
| `/claim <id>` | Claim a pending escrow |
| `/list` | List your escrows |
| `/offers` | Browse open offers with inline keyboard |
| `/reputation <address>` | Check counterparty stats |
| `/receipt <id>` | Export settlement receipt |
| `/status <id>` | Check escrow lifecycle state |
| `/dispute <id> <reason>` | Dispute an escrow |
| `/cancel <id>` | Cancel an escrow |
| `/msg <id> <text>` | Send message on an escrow |
| `/messages <id>` | Read escrow thread |
| `/help` | All commands |

## Data flow
1. User sends command to `@DagLockBot`
2. Bot parses command → calls indexer REST API
3. Bot responds with formatted data / inline keyboards
4. For signing: bot sends trade link deep link to KasWare/Kaspium

## Edge cases & gotchas
- Bot NEVER sees private keys — unsigned tx only
- Trade links: `https://t.me/DagLockBot?start=claim_<escrow_id>`
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
