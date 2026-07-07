# DagLock Testnet Quick Start

> **⚠️ TESTNET ONLY — Do not use real KAS**
>
> DagLock is currently running on **Kaspa Testnet-10**. All addresses, transactions, and balances on this page are for testing purposes only.
> **Never send real KAS (mainnet funds) to any address listed here.**

---

## 🚀 Quick Start (2 minutes)

### 1. Open DagLock

Visit **[https://daglock.com](https://daglock.com)**

You'll see the testnet banner at the top — that's how you know you're on testnet.

### 2. Get Test Funds

Get free testnet KAS from the Kaspa testnet faucet:

**[https://faucet-testnet.kaspanet.io](https://faucet-testnet.kaspanet.io)**

Enter a test address (see below) and request funds. They arrive instantly.

### 3. Connect With Manual Mode (No KasWare Needed)

KasWare browser extension doesn't support testnet-10. Use **Manual Mode** instead:

1. Click **"Use manual mode"** in the sidebar footer
2. Paste in one of the test addresses below
3. Done — you're connected

### 4. Create an Escrow

1. Go to **Escrows** → **Create** tab
2. Enter an amount (e.g. `100`)
3. Enter a seller address (use one of the test addresses below)
4. Click Create
5. For the TX ID, enter any hex string like `deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef`
6. The escrow will appear in "My Escrows"

### 5. Try the Telegram Bot

Open [@DagLock_bot](https://t.me/DagLock_bot) on Telegram and run:

```
/setaddress <test-address>
/fee 1000
/create    (follow the wizard)
/offers
```

---

## 🧪 Test Wallet Addresses

These are publicly shared test wallets. Anyone can use them. **Do not send real funds to them.**

### Wallet A — Buyer (has funds)

| Field | Value |
|-------|-------|
| **Address** | `kaspa:qtqwyqtmgczzjmj44vjzy` |
| **Private Key** | `2d93f2f2a4181731a682284db003140dd5ef18c868a77a685b89424788d21e73` |
| **Use case** | Create escrows, make offers, dispute |

### Wallet B — Seller

| Field | Value |
|-------|-------|
| **Address** | `kaspa:qjdpca9zm8aafdue2q0zn` |
| **Private Key** | `9e4fc9be71da065e090bb077da09b60b260898c06145b8b656cc4b873a0eaaeb` |
| **Use case** | Accept escrows, settle, receive funds |

### Wallet C — Mediator / Jury

| Field | Value |
|-------|-------|
| **Address** | `kaspa:qyp29592perates764gj8` |
| **Private Key** | `1ad52ce15703e9664c5c690640b5dc13546c5cc9199b39f93276453afa1093eb` |
| **Use case** | Test mediator dispute resolution, jury registration |

> ℹ️ **How these keys work:** The testnet indexer runs in `--mock-auth` mode, which accepts any signature as valid. You can use any address without real keys. These wallet addresses are just identifiers — no real KAS required. In production (mainnet), real Schnorr signatures from a Kaspa wallet are required.



---

## 💡 No Real KAS Needed

The testnet runs in **offline verification mode** (`MockVerifier`). This means:

- **No real testnet KAS required** — you don't need the faucet for DagLock to work
- **Any TX ID works** — for the TX ID field, just paste any 64-character hex string:
  ```
  deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef
  ```
- **Any signature works** — mock auth accepts everything
- **The full flow works** — create, settle, refund, dispute, jury, all of it

The covenants are compiled server-side and the full lifecycle is simulated. Real on-chain verification will be enabled after mainnet launch.

---

## 📱 Try the Full Flow

### Web: Create → Settle → Receipt

1. Connect Wallet A via manual mode
2. Go to **Escrows** → **Create** tab
3. Amount: `100`, Seller: Wallet B's address
4. For TX ID, paste: `deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef`
5. Click Create — escrow appears in "My Escrows"
6. Click the escrow → **Settle** → sign with mock signature → escrow settles
7. Go to **Receipt** tab → enter escrow ID → view receipt

### Bot: Create via Telegram

1. Open [@DagLock_bot](https://t.me/DagLock_bot)
2. `/setaddress kaspa:qtqwyqtmgczzjmj44vjzy`
3. `/create` — follow the 5-step wizard
4. Enter amount: `50`
5. Seller: Wallet B's address or tap Skip
6. Pick timeout + dispute mode
7. Confirm → bot shows covenant address
8. Copy the address + use `/submit_tx esc_xxx deadbeef...` to register

---

## 🔍 What to Try

| Feature | Where | How |
|---------|-------|-----|
| **Fee calculator** | Bot: `/fee 1000` or Dashboard | See 0.5% fee + USD estimate |
| **Offer board** | `/offers` or Offers tab | Browse active trade offers |
| **Create offer** | Offers tab → Create | List a trade for others |
| **Atomic swap** | Swap tab → Create Swap | Step-by-step wizard |
| **KRC-20 tokens** | Tokens tab | Browse and chart tokens |
| **Reputation** | Reputation tab → Lookup | Check an address's score |
| **Help center** | Help tab or `/help` | FAQ + quick start guide |

---

## 🐛 Found a bug?

- Open a GitHub issue: [github.com/dilljens/DagLock/issues](https://github.com/dilljens/DagLock/issues)
- Tag `@DagLock_bot` in the Kaspa Telegram group

---

## 🔒 Security Note

DagLock is **post-audit** but pre-mainnet. The covenants have been reviewed internally but may contain undiscovered bugs. Key properties:

- **No admin keys** — DagLock cannot move your funds
- **Covenant-enforced** — The SilverScript code enforces all rules
- **Open source** — All code at [github.com/dilljens/DagLock](https://github.com/dilljens/DagLock)

Testnet funds have no real value. Please report any issues you find.
