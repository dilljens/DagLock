# DagLock Handoff

> Everything you need to deploy, test, and iterate. Last updated: June 5, 2026.

---

## 1. Quick Start — Get the Site Live

### 30 minutes, free tier

**Step 1: Push to GitHub** ✅ Done — repo is at `github.com/dilljens/DagLock`

**Step 2: Deploy Indexer on Railway** ✅ Done
- URL: `https://daglock-production.up.railway.app`
- Start command: `daglock-indexer --host 0.0.0.0 --port 8443 --database-url sqlite:/data/daglock.db`

**Step 3: Deploy Web UI on Cloudflare Pages** ✅ Done
- URL: `https://daglock.com`
- Build command: `npm install --legacy-peer-deps && npm run build`
- Build output: `dist`
- Root directory: `web`
- Environment: `VITE_API_URL` = `https://daglock-production.up.railway.app`

**Step 4: Deploy Telegram Bot on Railway** ✅ Done
- Bot: `@DagLock_bot`
- Env: `BOT_TOKEN`, `INDEXER_URL`

---

## 2. Testing the Web UI

### What works ✅

| Feature | Status |
|---------|--------|
| Dashboard loads | ✅ |
| Network panel | ✅ |
| Stats panel | ✅ |
| Open offers | ✅ |
| Create escrow | ✅ |
| Create offer | ✅ |
| Market price offers | ✅ |
| Vault creation | ✅ |
| Vault lookup | ✅ |
| Escrow lookup | ✅ |
| Reputation lookup | ✅ |
| Receipt lookup | ✅ |
| Jury system | ✅ |
| Telegram linking | ✅ |

### Test Flow

1. Open `https://daglock.com`
2. Connect wallet (top right)
3. Create offer → Actions → Offer
4. Create escrow → Actions → Escrow
5. Create vault → Actions → Vault
6. Lookup escrow → Paste ID, click Fetch
7. Check reputation → Enter address, click Check

---

## 3. Testing the CLI

```bash
# Health check
cargo run -p daglock-cli -- --api-url https://daglock-production.up.railway.app health

# List offers
cargo run -p daglock-cli -- --api-url https://daglock-production.up.railway.app offer list

# Create offer
cargo run -p daglock-cli -- --api-url https://daglock-production.up.railway.app offer create \
  --side sell --base KAS --quote KRC20:NACHO --amount 100

# Check reputation
cargo run -p daglock-cli -- --api-url https://daglock-production.up.railway.app reputation \
  kaspa:qdyzkrhd74v6cetrv4fhv
```

---

## 4. Testing the Telegram Bot

```bash
cd bot
BOT_TOKEN=your-token INDEXER_URL=https://daglock-production.up.railway.app node src/index.js
```

Available commands:
```
/start            Welcome + claim handling
/setaddress       Set your Kaspa address
/create           Opens web dashboard
/claim <id>       Claim an escrow
/list             List your escrows
/offers           Browse open offers
/status <id>      Escrow details
/reputation <addr> Check stats
/dispute <id> <reason> Dispute
/cancel <id>      Cancel escrow
/msg <id> <text>  Send message
/messages <id>    Read thread
/help             All commands
```

---

## 5. Local Development

```bash
# Terminal 1: Start the indexer
cargo run -p daglock-indexer

# Terminal 2: Start the web UI
cd web && npm install && npm run dev
# Opens at http://localhost:5173

# Terminal 3: Run the CLI
cargo run -p daglock-cli -- --api-url http://localhost:8543 health

# Terminal 4: Run the bot
cd bot && BOT_TOKEN=xxx INDEXER_URL=http://localhost:8543 node src/index.js
```

---

## 6. Test Wallets (Testnet)

| Role | Address |
|------|---------|
| **Buyer** | `kaspa:qdyzkrhd74v6cetrv4fhv` |
| **Seller** | `kaspa:qg3h9mhu78cw89qyc0e42` |

Get testnet KAS from: https://faucet-tn10.kaspanet.io/

The mock verifier accepts any hex string as a signature.

---

## 7. Key Commands

```bash
# Test
cargo test --workspace              # 95 tests
cd web && npm run build             # Web UI build

# Run
cargo run -p daglock-indexer        # Start indexer (dev)
cd web && npm run dev               # Start web UI (dev)
cd bot && node src/index.js         # Start bot (dev)

# Deploy
git push origin main                # Triggers Railway + Cloudflare auto-deploy
bash scripts/deploy-web.sh          # Manual Cloudflare deploy

# Build
cargo build --release               # Release build
```

---

## 8. Architecture

```
User visits daglock.com
        │
        ▼
  Cloudflare Pages serves the React SPA
        │
        ▼
  Web UI calls api.daglock.io (Railway)
        │
        ▼
  Railway runs the indexer (Rust + SQLite)
        │
        ▼
  Indexer stores everything + optionally talks to Kaspa node
```

---

## 9. What's Built (Feature List)

### Core
- ✅ REST API (Rust/Axum)
- ✅ SQLite database
- ✅ Web dashboard (React/Vite)
- ✅ Telegram bot (Node.js)
- ✅ CLI tool (Rust)
- ✅ Price-locked offers (15-min CoinGecko)
- ✅ Market price orders
- ✅ Atomic swap (preimage mechanism)
- ✅ KRC-20 token support (covenant + UI)
- ✅ Vault system (time-locked)
- ✅ Jury system
- ✅ Vouching / reputation
- ✅ Escrow messaging
- ✅ Settlement receipts

### Security
- ✅ Schnorr signature verification (Secp256k1Verifier)
- ✅ CORS hardened
- ✅ No unwrap() in production
- ✅ Atomic database operations
- ✅ MockVerifier only for dev

### Infrastructure
- ✅ GitHub Actions CI (fmt, clippy, test, build)
- ✅ Railway deployment
- ✅ Cloudflare Pages deployment
- ✅ Telegram bot deployment
- ✅ Docker support

---

## 10. What's NOT Built Yet

| Priority | Feature | Effort |
|----------|---------|--------|
| P0 | On-chain UTXO verification (wRPC) | 3-5 days |
| P0 | Remove mock mode | 1-2 days |
| P1 | Offer history timeline | 2-3 days |
| P1 | Mobile responsive polish | 2-3 days |
| P2 | Rate limiting | 1 day |
| P2 | Webhooks | 3 days |
| P2 | Tax export | 2 days |

---

## 11. Environment Variables

### Railway (Indexer)
```
PORT=8443
RUST_LOG=info
DAGLOCK_MESSAGE_KEY=<hex>
```

### Railway (Bot)
```
BOT_TOKEN=<token from BotFather>
INDEXER_URL=https://daglock-production.up.railway.app
```

### Cloudflare Pages
```
VITE_API_URL=https://daglock-production.up.railway.app
```

---

## 12. Known Issues

| Issue | Status |
|-------|--------|
| Market price offers: price_type not stored | 🔴 Known bug |
| Mock verifier accepts any hex | ⚠️ Dev only |
| No on-chain verification | ⚠️ Dev only |

---

## 13. Security Notes

- `DAGLOCK_MESSAGE_KEY` — must be set in Railway
- `.env.cloudflare` — gitignored (has API token)
- `.env.testwallets` — gitignored (has private keys)
- MockVerifier is for dev only — never in production
