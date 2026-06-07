# Bot

**Source**: `bot/src/`  **Updated**: `2026-06-05`  (2 files)

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
| `/create` | Open web dashboard for creation |
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

## Audit Findings (2026-06-06)

### Critical / Medium Security Issues

| ID | Finding | Location | Fix Required |
|----|---------|----------|--------------|
| **S6** | **Bot stores addresses in plaintext /tmp** — `/tmp/daglock-users.json` world-readable on shared systems. No encryption at rest. | `index.js:20-34` | Encrypt with libsodium/tweetnacl using `BOT_ENCRYPTION_KEY` env var. |
| **Q8** | **Bot API no retry/backoff** — Single `fetch()` fails under load. No resilience. | `lib/api.js` | 3 attempts, exponential backoff (1s/2s/4s) on 5xx/timeout. |

### High-Priority Usability Issues

| ID | Finding | Location | Fix Required |
|----|---------|----------|--------------|
| **U4** | **Bot `/create` redirects to web** — Opens web dashboard via inline keyboard. Telegram users expect in-chat flows. No native `/create` wizard. | `index.js:74-88` | Add grammY conversation: amount → counterparty → timeout → dispute mode → unsigned tx → deep link. |

### Structural Issues

| ID | Finding | Impact |
|----|---------|--------|
| **A6** | **Bot is Node.js while rest is Rust** — Different dep chain, no shared types, maintenance burden. | Long-term: rewrite in Rust (teloxide/grammy-rs). Short-term: harden existing. |

### Fix Plan (Phase 1 — Security + Phase 2 — Usability)

1. **Task 7 (S6):** Bot encrypt user addresses — libsodium with `BOT_ENCRYPTION_KEY` env var
   - Use `tweetnacl` (pure JS) or `libsodium-wrappers` for encryption
   - Encrypt entire JSON before write, decrypt on read
   - Key from `BOT_ENCRYPTION_KEY` (base64-encoded 32-byte key)

2. **Task 30 (Q8):** Bot API retry/backoff — wrap `fetch()` with retry logic
   - 3 attempts, exponential backoff: 1s, 2s, 4s
   - Retry on: 5xx, 429, network errors, timeout
   - Add `AbortController` timeout (10s) per attempt

3. **Task 12 (U4):** Bot native `/create` wizard — grammY conversations plugin
   - Step 1: `/create` → prompt amount (KAS)
   - Step 2: Prompt counterparty address
   - Step 3: Prompt timeout (default 24h)
   - Step 4: Prompt dispute mode (standard/mediator/jury)
   - Step 5: Call indexer `/v1/compile` for unsigned tx
   - Step 6: Return KasWare/Kaspium deep link with tx

### Dependencies

- **Indexer** must have `/v1/compile` endpoint working (already exists)
- **Indexer** must have real UTXO verification (S1) for end-to-end flow
- **KasWare/Kaspium** deep link format for signing (verify current format)
- **BOT_ENCRYPTION_KEY** must be set in production env (generate with `openssl rand -base64 32`)

### Verification

- [ ] `cd bot && npm test` passes
- [ ] Manual: `/setaddress` → `/create` → completes wizard → deep link opens KasWare → signs → broadcasts
- [ ] Manual: Restart bot → `/list` shows previously set address (decrypted correctly)
- [ ] Manual: Indexer under load (5xx) → bot retries 3x with backoff → eventually succeeds or fails gracefully
- [ ] Manual: `/tmp/daglock-users.json` is encrypted (not readable plaintext)

### Long-term: Bot Rewrite (Out of Scope for Mainnet)

Consider rewriting bot in Rust using `teloxide` or `grammy-rs` to:
- Share types with indexer (`indexer/src/types.rs` → `bot/src/types.rs` via shared crate)
- Single dependency chain (Cargo workspace)
- Better performance, type safety
- Estimated effort: 2-3 weeks

