# bot

Telegram bot (`@DagLock_bot`). 50+ commands using grammY framework. Communicates with indexer REST API.

## Storage

- **July 8, 2026:** Migrated from `/tmp/daglock-users.json` to SQLite via `better-sqlite3`.
- **Module:** `bot/src/db.js` — wraps `better-sqlite3` with ESM `createRequire`.
- **Tables:** `users` (telegram_id, address, updated_at).
- **Migration:** Auto-migrates legacy `/tmp/daglock-users.json` on first run (backup as `.bak`).
- **Encryption:** Addresses still encrypted at rest via `BOT_ENCRYPTION_KEY` (caller's responsibility).
- **Env:** `BOT_DB_PATH` (default: `bot/bot.db`).

## Tests

- `tests/unit/commands.test.js` — command handler logic (10 tests)
- `tests/unit/db.test.js` — SQLite CRUD + migration (7 tests)
- `src/lib/api.test.js` — API client with mocked fetch (12 tests)
- `src/crypto.test.js` — encryption/decryption round-trips (10 tests)
- Run: `cd bot && npm test` (39 tests)

## All Commands

| Command | Category | Purpose |
|---------|----------|---------|
| `/start` | General | Welcome + deep link handling |
| `/help` | General | Full command list |
| `/create` | Escrow | 6-step native wizard (amount, counterparty, timeout, presets) |
| `/settle` | Escrow | Settle an active escrow |
| `/refund` | Escrow | Refund an escrow |
| `/cancel` | Escrow | Cancel an escrow or wizard |
| `/dispute` | Escrow | Dispute an escrow |
| `/status` | Escrow | Check escrow lifecycle state |
| `/list` | Escrow | List your escrows |
| `/claim` | Escrow | Claim a pending escrow |
| `/submit_tx` | Escrow | Submit TX ID after broadcasting |
| `/submit_sig` | Escrow | Submit signature for settle/refund |
| `/swap` | Swap | Create/claim/status atomic swaps |
| `/invoice` | Invoice | 3-step invoice wizard |
| `/offers` | Offers | Browse open offers |
| `/counter` | Offers | Counter an offer |
| `/counters` | Offers | List counter-offers |
| `/fee` | Utility | Calculate escrow fee + USD conversion |
| `/receipt` | Utility | Export settlement receipt |
| `/reputation` | Reputation | Check address reputation |
| `/feedback` | Reputation | Leave trade feedback |
| `/block` | Safety | Block a user |
| `/krc20_list` | KRC-20 | List your tokens |
| `/krc20` | KRC-20 | Show token detail |
| `/krc20_create` | KRC-20 | 5-step token creation wizard |
| `/subscriptions` | Subscriptions | List your subscriptions |
| `/sub_create` | Subscriptions | Create subscription escrow |
| `/sub_draw` | Subscriptions | Draw current installment |
| `/sub_cancel` | Subscriptions | Cancel a subscription |
| `/milestones` | Milestone | List milestone escrows |
| `/create_milestone` | Milestone | Wizard with up to 5 stages |
| `/release_milestone` | Milestone | Release current milestone |
| `/milestone_approve` | Milestone | Approve milestone as buyer |
| `/milestone_dispute` | Milestone | Dispute milestone |
| `/milestone_refund` | Milestone | Refund remaining funds |
| `/milestone_complete` | Milestone | Mutual complete |
| `/multi_escrows` | Multi-Party | List multi-party escrows |
| `/create_multi` | Multi-Party | Create with party/shares wizard |
| `/sign` | Multi-Party | Sign release as a party |
| `/deposit` | Deposits | Create security deposit |
| `/deposit_status` | Deposits | Check deposit status |
| `/deposit_release` | Deposits | Mutual release |
| `/vaults` | Vaults | List your vaults |
| `/vault_create` | Vaults | Create time-locked vault |
| `/vault_withdraw` | Vaults | Withdraw from vault |
| `/msg` | Chat | Send message on escrow |
| `/messages` | Chat | Read escrow thread |
| `/reveal` | Chat | Reveal chat key to jury |
| `/evidence` | Chat | View dispute evidence |
| `/mediate` | Mediation | Start AI mediation |
| `/mediate_accept` | Mediation | Accept mediation outcome |
| `/mediate_status` | Mediation | Check mediation status |
| `/stats` | Info | Platform statistics |
| `/anchors` | Info | Message anchor status |

---
*Confidence: 0.95 · Last updated: 7/7/2026*
