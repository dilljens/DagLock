# Plan: Discord Bot (P5)

**Goal:** Bring DagLock escrow to Discord — the other half of the Kaspa community. Mirror the Telegram bot's functionality in a Discord bot using discord.js.

**Effort:** 3-5 days

**Why this matters:** The Kaspa community is split between Telegram (OTC trading, R&D discussion) and Discord (mining, technical support, project announcements). A Discord bot captures the second half of the addressable market. Current flow: users see a DagLock mention on Discord → switch to Telegram → switch to web → give up.

---

## Phase A: Core bot framework `[ ]`
**⏱ Timebox:** 1 day

- [ ] Create `bot-discord/` directory with Node.js + discord.js v14
- [ ] Register slash commands:
  - `/create` — 6-step modal wizard (same flow as Telegram bot)
  - `/claim <id>` — claim an escrow
  - `/list` — list your escrows
  - `/status <id>` — check escrow status
  - `/settle <id>` — settle an escrow (prompts for signature)
  - `/refund <id>` — refund an escrow
  - `/cancel <id>` — cancel an escrow
  - `/dispute <id> <reason>` — dispute an escrow
  - `/offer create [side] [amount] [asset]` — create an offer
  - `/offer list` — browse offers
  - `/reputation <address>` — check reputation
  - `/fee <amount>` — calculate escrow fee
  - `/help` — all commands
- [ ] Share the same `ApiClient` from the Telegram bot (same REST API)
- [ ] Durable user storage: address linked to Discord ID (encrypted at rest)
- [ ] Use Discord's interaction modals for multi-step flows

**✅ Checkpoint:** `/fee 1000` returns "0.5% = 5 KAS fee" in a Discord channel.

---

## Phase B: Escrow creation wizard `[ ]`
**⏱ Timebox:** 1 day

- [ ] `/create` opens a Discord modal (pop-up form) with fields:
  - Amount (number input)
  - Counterparty address (text input)
  - Timeout (select: 1h/24h/3d/7d)
  - Dispute mode (select: standard/mediator/jury)
- [ ] After submission, generate unsigned TX and respond with:
  - Transaction summary embed
  - "Connect your wallet to sign" button
  - "Copy transaction data" fallback
- [ ] Ephemeral responses for sensitive info (addresses, amounts)

**✅ Checkpoint:** User runs `/create`, fills modal, gets a summary with signing instructions.

---

## Phase C: Embedded offer browsing `[ ]`
**⏱ Timebox:** 1 day

- [ ] `/offer list` shows a rich embed with 5 offers per page:
  ```
  📋 Open Offers — Page 1/3
  ┌──────────────────────────────────┐
  │ 🔴 SELL 1000 KAS → KRC20:NACHO  │
  │  Creator: kaspa:q...            │
  │  Status: proposed                │
  │  2h ago · [View on Explorer]     │
  └──────────────────────────────────┘
  [← Prev] [1] [2] [3] [Next →] [Refresh]
  ```
- [ ] Pagination via button components (ephemeral)
- [ ] `/offer create` — slash command with option parameters
- [ ] `/offer accept <id>` — accept an offer, create escrow

**✅ Checkpoint:** Discord user browses offers with pagination, accepts one.

---

## Phase D: Notifications + server integration `[ ]`
**⏱ Timebox:** 1 day

- [ ] Escrow status notifications via webhook (indexer sends POST to bot)
- [ ] Users can opt-in: `/notify on` → receive DM when escrow status changes
- [ ] Server admins can configure a channel for public escrow announcements
- [ ] Auto-thread: when a dispute is created, auto-create a Discord thread for moderated discussion
- [ ] Reputation lookup in-channel: `/reputation <address>` posts a public embed

**✅ Checkpoint:** User receives "Your escrow has been claimed" DM in Discord.

---

## Phase E: Tests + deploy `[ ]`
**⏱ Timebox:** 1 day

- [ ] Unit tests: command parsing, modal validation, API responses
- [ ] Manual test: full lifecycle in a Discord server
- [ ] Deploy as systemd service on VPS: `daglock-discord-bot.service`
- [ ] Register slash commands with Discord API on startup
- [ ] Add to `AGENTS.md` and `VPS.md` docs

**✅ Checkpoint:** Discord bot slash commands respond correctly in a test server.

---

## Files Changed / Created

| File | Change |
|------|--------|
| `bot-discord/package.json` | **New** — discord.js, api client |
| `bot-discord/src/index.js` | **New** — main bot entry |
| `bot-discord/src/commands/` | **New** — slash command handlers |
| `bot-discord/src/lib/api.js` | **New** — shared REST client (mirrors bot/src/lib/api.js) |
| `bot-discord/src/crypto.js` | **New** — address encryption (copy from bot/) |
| `bot-discord/Dockerfile` | **New** — container for VPS deployment |
| `docs/DISCORD-BOT.md` | **New** — setup + usage guide |
| `indexer/src/api/webhooks.rs` | Extend to support Discord bot webhook target |

## Shared code with Telegram bot

| Module | Reuse strategy |
|--------|---------------|
| `ApiClient` | Copy (same REST API, different bot framework) |
| Address encryption | Copy |
| Fee calculation | Copy (reuse `FEEDENOMINATOR` constant) |
| Covenant compilation | Same `/v1/compile` endpoint |

## Edge Cases

| Case | Handling |
|------|----------|
| Discord rate limits (5 requests per guild per 5s) | Implement a simple queue/bucket |
| User runs command in DMs vs guild channel | Both supported, guild commands have ephemeral fallback for private data |
| Discord slash command permissions | Require "Use External Apps" permission in guild |
| Multiple instances (Telegram + Discord) | Separate services, same indexer |
| User links same address to Telegram and Discord | Independent — each platform has its own address mapping |
