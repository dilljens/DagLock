# Reddit Post Draft: DagLock Testnet Tutorial

> Target: r/kaspa
> Format: Text post with 3-4 screenshots
> Status: Draft — review before posting

---

## Title

**Try DagLock escrow on Kaspa testnet — 2 minutes, no wallet, no KAS needed**

---

## Body

Ever wanted to try trustless escrow on Kaspa but didn't want to install KasWare, mess with faucets, or risk real funds?

DagLock is running on testnet and the full flow works with **zero setup**. No wallet extension. No testnet KAS. No private keys to manage. Here's exactly how to test it.

---

### What is DagLock?

DagLock is a trustless escrow platform built with SilverScript covenants on Kaspa L1. Every escrow is a UTXO covenant — the rules are enforced by the blockchain, not by a company or admin keys. You can escrow KAS or KRC-20 tokens with automatic settlement, dispute resolution, and atomic swap support.

Source code: [github.com/dilljens/DagLock](https://github.com/dilljens/DagLock) (post-audit, pre-mainnet)

---

### Step 1: Open the site

Go to **[daglock.com](https://daglock.com)**.

You'll see a red testnet banner at the top — that's how you know you're on testnet. No real funds at any point.

**[SCREENSHOT: homepage with testnet banner highlighted]**

---

### Step 2: Connect manual mode

In the sidebar footer, click **"Use manual mode"**. A text input appears.

Paste in the Buyer address from the table below:

```
kaspa:qtqwyqtmgczzjmj44vjzy
```

Click "Set Address". You're now connected — the sidebar should show "Manual (testnet)".

**[SCREENSHOT: manual mode input with buyer address filled in]**

---

### Step 3: Create an escrow

Go to **Escrows → Create** tab. Fill in:

| Field | Value |
|-------|-------|
| **Amount** | `100` |
| **Seller address** | `kaspa:qjdpca9zm8aafdue2q0zn` |
| **Deal type** | Pick any (e.g. OTC Trade) |
| **Auto-settle** | Optional — toggle on if you want to see auto-settle |

Click "Create Escrow". A prompt asks for a TX ID — paste this:

```
deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef
```

Your escrow appears in **My Escrows** with status `pending_confirmation`.

**[SCREENSHOT: create form filled out + escrow appearing in list]**

---

### Step 4: Settle the escrow

Click the escrow in the list to expand it. Click the **Settle** button.

A "Sign with Wallet" prompt appears — but since you're in manual mode, you'll see a **"Mock sign (dev mode)"** button instead. Click it.

Wait 2 seconds. The escrow status changes to `settled`. You've just completed a trustless escrow transaction on Kaspa.

The 0.5% protocol fee is calculated and visible. No real funds moved — but this is exactly the same flow that will work with real KAS on mainnet.

**[SCREENSHOT: escrow showing settled status]**

---

### Step 5: Try the Telegram bot

Open **[@DagLock_bot](https://t.me/DagLock_bot)** on Telegram and try:

```
/setaddress kaspa:qtqwyqtmgczzjmj44vjzy
/fee 1000
/create
/offers
```

The `/create` command opens a native wizard: pick amount, counterparty, timeout, and dispute mode. When you're done, the bot sends a link to the web app to complete the flow.

---

### Test Wallet Addresses

These are publicly shared testnet wallets. Anyone can copy them.

| Role | Address | Private key |
|------|---------|-------------|
| **Buyer** | `kaspa:qtqwyqtmgczzjmj44vjzy` | `2d93f2f2a4181731a682284db003140dd5ef18c868a77a685b89424788d21e73` |
| **Seller** | `kaspa:qjdpca9zm8aafdue2q0zn` | `9e4fc9be71da065e090bb077da09b60b260898c06145b8b656cc4b873a0eaaeb` |
| **Mediator** | `kaspa:qyp29592perates764gj8` | `1ad52ce15703e9664c5c690640b5dc13546c5cc9199b39f93276453afa1093eb` |

Private keys are exposed because this is **testnet only** — never send real KAS to these addresses.

---

### What else to try

| Feature | Where | How |
|---------|-------|-----|
| **Offer board** | Offers tab | Browse and create trade offers |
| **Atomic swap** | Swap tab | 6-step guided swap wizard |
| **KRC-20 tokens** | Tokens tab | Token charts, prices, trading |
| **Vaults** | Vaults tab | Time-locked, multisig, password vaults |
| **Reputation** | Reputation tab | Check a Kaspa address's trade score |
| **Fee calculator** | Bot: `/fee 1000` | See 0.5% fee + USD estimate |

---

### How this works (transparency note)

The testnet indexer runs in **offline verification mode** ("MockVerifier"). This means:

- Any TX ID is accepted (no real on-chain verification needed)
- Any signature is accepted (mock auth mode)
- The covenants are compiled server-side and the full lifecycle is simulated
- **Mainnet will require real KAS, real KasWare/Kaspium signing, and real on-chain verification**

The covenants themselves are real — the same SilverScript code that will run on mainnet. The only difference is how the indexer validates UTXOs.

---

### Mainnet launch

DagLock targets **mainnet for June 30, 2026** (coinciding with the Toccata hard fork on Kaspa).

What's done:
- ✅ All 12 covenants (KAS escrow, KRC-20, vaults, arbiter, reputation, etc.)
- ✅ Internal security audit (28/30 items complete, 7 critical/high fixes deployed)
- ✅ Full lifecycle tests (241 Rust + 40 Web + 22 Bot = 303 tests passing)
- ✅ Telegram bot, web dashboard, CLI tool, REST API, embeddable widget, WASM SDK

---

### Links

- **Site**: [daglock.com](https://daglock.com)
- **Bot**: [@DagLock_bot](https://t.me/DagLock_bot)
- **API**: [api.daglock.com](https://api.daglock.com)
- **Code**: [github.com/dilljens/DagLock](https://github.com/dilljens/DagLock)
- **Report bugs**: [GitHub issues](https://github.com/dilljens/DagLock/issues) or tag @DagLock_bot in Kaspa Telegram

---

*DagLock has no admin keys, no backdoors, and cannot move your funds. The covenant enforces every rule. All code is open source. Testnet funds have no value — please report any bugs you find.*

---

## Screenshots to generate

1. `screenshots/reddit-01-dashboard.png` — Homepage with testnet banner visible
2. `screenshots/reddit-02-manual-mode.png` — Manual mode input with buyer address filled in
3. `screenshots/reddit-03-create-escrow.png` — Create escrow form filled with 100 KAS, seller address, deal type
4. `screenshots/reddit-04-settled.png` — Escrow showing settled status

See `scripts/screenshots-reddit.cjs` for the capture script.

---

## Reddit markdown notes

- Reddit uses a Markdown variant that supports `[text](url)` links, `###` headers, tables, and `code` blocks
- Screenshots are hosted on Reddit's native image upload (drag & drop into the post editor)
- Tables work but Reddit may mangle them on mobile — keep them simple
- Keep the total post under ~40000 characters (Reddit limit)
