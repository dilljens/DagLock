# DagLock Handoff

> Everything you need to deploy, test, and iterate. Last updated: June 4, 2026.

---

## 1. Quick Start — Get the Site Live

### 30 minutes, free tier

**Step 1: Push to GitHub** ✅ Done — repo is at `github.com/dilljens/DagLock`

**Step 2: Deploy Indexer on Railway** (5 min)

1. Go to [railway.com/dashboard](https://railway.com/dashboard)
2. New Project → Deploy from GitHub → `dilljens/DagLock`
3. Railway auto-detects the Dockerfile
4. Go to **Variables** tab, add:

| Variable | Value |
|----------|-------|
| `DAGLOCK_MESSAGE_KEY` | Run `openssl rand -hex 32` in your terminal |
| `PORT` | `8443` |
| `RUST_LOG` | `info` |

5. Go to **Volumes** tab → Add Volume:
   - Name: `daglock-data`
   - Mount path: `/data`
   - Size: 1GB

6. Go to **Deploy** tab → **Start Command** → Change to:
   ```
   daglock-indexer --host 0.0.0.0 --port 8443 --database-url sqlite:/data/daglock.db
   ```

7. Wait 3-4 minutes for the Docker build. Your URL will be: `https://daglock-indexer.up.railway.app`

**Step 3: Deploy Web UI on Cloudflare Pages** (5 min)

1. Go to [dash.cloudflare.com](https://dash.cloudflare.com) → **Pages**
2. Create a project → Connect Git → `dilljens/DagLock`
3. Build settings:
   - Build command: `npm run build`
   - Build output: `dist`
   - Root directory: `web`
4. Environment variables:
   - `VITE_API_URL`: `https://daglock-indexer.up.railway.app`
5. Click **Save and Deploy**

**Step 4: Custom domain** (optional, 5 min)

In Cloudflare Pages → your project → Custom domains → Add `daglock.com`

See `docs/DEPLOYMENT-RAILWAY.md` for full details with troubleshooting.

---

## 2. Testing the Web UI

Once deployed, open `https://YOUR-RAILWAY-URL` or `https://daglock.com`.

### What works (even without a Kaspa node)

The indexer runs in "offline mode" — no Kaspa node needed. Everything except on-chain UTXO detection works.

### Test Flow (5 minutes)

#### 2a. Browse the dashboard
- Open the site — you'll see the hero section with "Browse offers" and "Take action" buttons
- The 4 highlight cards show escrow counts (all zero initially)
- The Network panel shows API status, version
- The Stats panel shows escrow breakdowns

#### 2b. Check reputation
- Scroll to **Reputation** panel
- Enter `kaspa:qdyzkrhd74v6cetrv4fhv` (your test buyer wallet)
- Click **Check**
- You'll see: Score `1.0/5`, 0 trades, 0 volume, 0 days age

#### 2c. Create an offer
- Click **Actions** tab → **Create offer**
- Fill in:
  - Side: `Sell`
  - Sell asset: `KAS`
  - For asset: `KRC20:NACHO`
  - Amount: `500`
  - Expires: `7 days`
  - Address: `kaspa:qdyzkrhd74v6cetrv4fhv`
- The offer appears in **Open offers** above with a green "proposed" badge and relative time

#### 2d. Create an escrow
- Actions → **Create escrow**
- Fill in:
  - Dispute resolution: `Standard` (or `Jury` to test jury)
  - Amount: `100`
  - Buyer: `kaspa:qdyzkrhd74v6cetrv4fhv`
  - Seller: `kaspa:qg3h9mhu78cw89qyc0e42`
- Click **Create** — you'll see the escrow ID, amount, and price at creation (KAS/USD from CoinGecko)

#### 2e. Look up the escrow
- Scroll to **Escrow lookup**
- Paste the escrow ID, click **Fetch**
- You'll see: Status timeline (Locked → Active → etc), Amount, Fee (0.5%), Price, Created time

#### 2f. Send a message
- In the escrow lookup panel, enter your chat auth:
  - Address: `kaspa:qdyzkrhd74v6cetrv4fhv`
  - Signature: `abcd` (any hex works with mock verifier)
- Click **Fetch** again to load messages
- Type a message and click **Send** — it appears in the thread above

#### 2g. Dispute with jury
- Create another escrow (any amount)
- In escrow lookup, note the ID
- Actions → **Dispute**
  - Enter the escrow ID
  - Reason: "Test dispute"
  - Dispute mode: `Jury`
  - Address + Signature (mock)
- Submit — it creates a jury case and selects jurors

#### 2h. Check your history
- In the lookup section, find **My escrows**
- Enter `kaspa:qdyzkrhd74v6cetrv4fhv` → click **List**
- You'll see all escrows you're involved in

- Actions → **My offers**
- Enter `kaspa:qdyzkrhd74v6cetrv4fhv` → **List my offers**
- Shows your offers

#### 2i. Link Telegram
- Actions → **Link Telegram**
- Address: `kaspa:qdyzkrhd74v6cetrv4fhv`
- Handle: `@your_test_handle`
- Signature: `abcd`
- Click **Link** — then check Reputation again to see the handle

#### 2j. Check stats
- The Stats panel updates as you create escrows
- Total escrows, settled, disputed, fees collected

---

## 3. Testing the CLI

```bash
# From the daglock directory

# List offers
cargo run -p daglock-cli -- --api-url https://daglock-indexer.up.railway.app offer list

# Create an offer
cargo run -p daglock-cli -- --api-url https://daglock-indexer.up.railway.app offer create \
  --side sell --base KAS --quote KRC20:NACHO --amount 100

# Check reputation
cargo run -p daglock-cli -- --api-url https://daglock-indexer.up.railway.app reputation \
  kaspa:qdyzkrhd74v6cetrv4fhv

# Check escrow status
cargo run -p daglock-cli -- --api-url https://daglock-indexer.up.railway.app status esc_<id>

# Send a message
cargo run -p daglock-cli -- --api-url https://daglock-indexer.up.railway.app msg esc_<id> \
  --text "Hello from CLI" \
  --address kaspa:qdyzkrhd74v6cetrv4fhv \
  --signature abcd

# View messages
cargo run -p daglock-cli -- --api-url https://daglock-indexer.up.railway.app messages esc_<id> \
  --address kaspa:qdyzkrhd74v6cetrv4fhv \
  --signature abcd

# Get receipt
cargo run -p daglock-cli -- --api-url https://daglock-indexer.up.railway.app receipt esc_<id>
```

Note: The CLI builds from source. Installation for other users would be:
```bash
cargo install --git https://github.com/dilljens/DagLock daglock-cli
```

---

## 4. Testing the Telegram Bot

Requires a bot token from [@BotFather](https://t.me/BotFather):

```bash
cd bot
BOT_TOKEN=<your-token> INDEXER_URL=https://daglock-indexer.up.railway.app node src/index.js
```

Available commands:
```
/start            Welcome + claim handling
/create           Opens web dashboard
/claim <id>      Claim an escrow
/list             List your escrows
/offers           Browse open offers
/status <id>     Escrow details
/reputation <addr> Check stats
/receipt <id>    Settlement receipt
/dispute <id> <reason> Dispute
/cancel <id>     Cancel escrow
/msg <id> <text> Send message
/messages <id>   Read thread
/help             All commands
```

The Telegram bot is Node.js. For production, you'd deploy it on Railway too (separate service, Dockerfile included).

---

## 5. Local Development (if you want to run locally)

```bash
# Terminal 1: Start the indexer
cargo run -p daglock-indexer

# Terminal 2: Start the web UI
cd web && npm install && npm run dev
# Opens at http://localhost:5173

# Terminal 3: Run the E2E test
python3 scripts/e2e.py

# Or run the reputation simulation
python3 scripts/simulation.py --trades 20 --bots 2
```

The Vite dev server proxies `/v1/*` requests to `localhost:8443` automatically.

---

## 6. Test Wallets (for manual testing)

Generated by `python3 scripts/genkeys.py generate` and saved in `.env.testwallets` (gitignored).

| Role | Address |
|------|---------|
| **Buyer** | `kaspa:qdyzkrhd74v6cetrv4fhv` |
| **Seller** | `kaspa:qg3h9mhu78cw89qyc0e42` |

See `.env.testwallets` for the private keys. The mock verifier accepts any hex string as a signature — you can just type `abcd`.

---

## 7. What to focus on after deployment

| Priority | What | Why |
|----------|------|-----|
| 1 | **Send the link to someone** | Get real feedback — does anyone understand the UI? |
| 2 | **Create test KRC-20 tokens** | Follow `docs/KRC20-TESTNET.md` — prove the KRC-20 flow works |
| 3 | **Post in Kaspa Telegram groups** | NACHO, KASPY, general Kaspa chat |
| 4 | **Watch the logs** | Railway shows real-time logs. See what breaks |
| 5 | **Fix what breaks** | Iterate based on feedback |

---

## 8. Key commands reference

```bash
# Test
cargo test --workspace              # 107 tests
cd web && npm run build             # Web UI build

# Run
cargo run -p daglock-indexer        # Start indexer (dev)
DAGLOCK_MESSAGE_KEY=test ./target/release/daglock-indexer  # Start indexer (release)

# Deploy
git push origin main                # Triggers Railway + Cloudflare auto-deploy

# Simulate
python3 scripts/simulation.py --trades 20 --bots 2  # Generate test data
python3 scripts/e2e.py                               # Run E2E checks

# Keys
python3 scripts/genkeys.py generate  # New test wallet
```

---

## 9. Architecture in 30 seconds

```
User opens daglock.com
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
  Indexer stores everything + optionally talks to Kaspa node via wRPC
```

---

## 10. Toccata hard fork

The hard fork that enables covenants on Kaspa mainnet opens **June 5, 2026** (tomorrow). This is when DagLock's covenants become deployable on mainnet. You don't need to do anything special — the covenants compile for both testnet and mainnet. When you're ready, point the indexer at a mainnet node with `--network mainnet --allow-mainnet`.
