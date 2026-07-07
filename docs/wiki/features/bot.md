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

## Commands

| Command | Purpose | Status |
|---------|---------|--------|
| `/start` | Welcome + deep link handling | ✅ |
| `/setaddress` | Set your Kaspa address | ✅ |
| `/create` | **6-step native wizard** (no web redirect) | ✅ **New** |
| `/invoice` | 3-step invoice wizard | ✅ |
| `/claim` | Claim a pending escrow | ✅ |
| `/settle` | Settle an active escrow | ✅ **New** |
| `/refund` | Refund an escrow | ✅ **New** |
| `/list` | List your escrows | ✅ |
| `/offers` | Browse open offers | ✅ |
| `/counter` | Counter an offer with a different amount | ✅ **New** |
| `/counters` | List counter-offers on an offer | ✅ **New** |
| `/status` | Check escrow lifecycle state | ✅ |
| `/receipt` | Export settlement receipt | ✅ |
| `/swap` | Atomic swap via preimage | ✅ |
| `/dispute` | Dispute an escrow | ✅ |
| `/cancel` | Cancel an escrow or wizard | ✅ |
| `/submit_tx` | Submit TX ID after broadcasting | ✅ **New** |
| `/submit_sig` | Submit signature for settle/refund | ✅ **New** |
| `/reputation` | Check counterparty stats | ✅ |
| `/fee` | Calculate escrow fee | ✅ **New** |
| `/block` | Block a user | ✅ **New** |
| `/feedback` | Leave trade feedback | ✅ **New** |
| `/vaults` | List your vaults | ✅ |
| `/msg` | Send message on escrow | ✅ |
| `/messages` | Read escrow thread | ✅ |
| `/evidence` | List dispute evidence | ✅ |
| `/help` | All commands | ✅ |

## Deep Links

- `https://t.me/DagLock_bot?start=claim_<id>` — claim an escrow
- `https://t.me/DagLock_bot?start=swap_<id>` — open swap claim page
- `https://daglock.com/swap/<id>` — web deep link for swaps

---

*Confidence: 0.95 · Last updated: 7/3/2026*
