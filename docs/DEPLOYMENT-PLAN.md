# DagLock Deployment Plan — Testnet + Mainnet

> Dual-network deployment strategy. One week of testnet feedback, then mainnet launch with shared infrastructure.

---

## 1. Current State

| Surface | URL | Network | Mode |
|---------|-----|---------|------|
| Indexer (Railway) | `daglock-indexer.up.railway.app` | `testnet-12` (default) | Offline (no wRPC) |
| Web UI (Cloudflare) | `daglock.com` / Cloudflare Pages | Points at Railway | Live |
| Telegram bot | `@DagLock_bot` | Not deployed yet | — |

The existing Railway service uses the default `--network testnet-12` with no wRPC connection. It stores all data in a SQLite volume.

---

## 2. Target Architecture

```
                      ┌─────────────────────────────┐
                      │     Cloudflare DNS           │
                      │  daglock.com ──→ CF Pages    │
                      │  test.daglock.com ──→ CF     │
                      │  api.daglock.com ──→ Railway  │
                      │  test-api.daglock.com ──→     │
                      └─────────────────────────────┘

Mainnet:
  daglock.com ──→ api.daglock.com ──→ Railway Service A (mainnet indexer)
                                        Volume: daglock-mainnet-data
                                        Network: mainnet + wRPC
                                        --allow-mainnet

Testnet:
  test.daglock.com ──→ test-api.daglock.com ──→ Railway Service B (testnet indexer)
                                                   Volume: daglock-testnet-data
                                                   Network: testnet-12 (default)
                                                   No --allow-mainnet (safety)

Telegram:
  @DagLock_bot ──→ api.daglock.com (mainnet)
  @DagLock_test_bot ──→ test-api.daglock.com (testnet)
```

---

## 3. Step-by-Step Setup

### Step 1: Identify Current Railway Service (Day 1, 5 min)

Your existing Railway service runs the indexer in offline/testnet mode. This becomes the **testnet** service.

**Verify current config:**
- Go to Railway dashboard → your project
- Note the service name, volume mount, and start command
- Current start command: `daglock-indexer --host 0.0.0.0 --port 8443 --database-url sqlite:/data/daglock.db`

### Step 2: Rename Current Service to "testnet" (Day 1, 2 min)

In Railway:
1. Your existing project → Settings → Rename service to `daglock-indexer-testnet`
2. Note its generated URL (e.g. `daglock-indexer-testnet.up.railway.app`)
3. This is your testnet API endpoint

### Step 3: Create Mainnet Railway Service (Day 1, 5 min)

In the same Railway project:
1. **New Service** → **Deploy from GitHub repo** → same `dilljens/DagLock` repo
2. Name the service: `daglock-indexer-mainnet`
3. Railway builds from the same Dockerfile

**Environment variables:**

| Variable | Value |
|----------|-------|
| `DAGLOCK_MESSAGE_KEY` | `openssl rand -hex 32` (generate a new one) |
| `PORT` | `8443` |
| `RUST_LOG` | `info` |

**Volume:**
- Name: `daglock-mainnet-data`
- Mount path: `/data`
- Size: 1GB

**Start command:**
```bash
daglock-indexer --host 0.0.0.0 --port 8443 --database-url sqlite:/data/daglock.db --network mainnet --allow-mainnet
```

Note: The mainnet indexer works **without** `--wrpc-url` initially — it runs in offline mode, storing offers, reputation data, messages, etc. On-chain UTXO detection only activates once you connect to a Kaspa mainnet node.

### Step 4: Add Custom Domains (Day 1, 5 min)

In Railway → each service → **Settings** → **Domains**:

| Service | Custom domain |
|---------|---------------|
| `daglock-indexer-mainnet` | `api.daglock.com` |
| `daglock-indexer-testnet` | `test-api.daglock.com` |

Add CNAME records in Cloudflare DNS pointing these to the Railway-generated URLs.

### Step 5: Deploy Testnet Web UI (Day 1, 5 min)

In Cloudflare Pages:

1. **Create a new Pages project** → Connect to same `dilljens/DagLock` repo
2. Build settings:
   - Build command: `npm run build`
   - Build output: `dist`
   - Root directory: `web`
3. Environment variables:
   - `VITE_API_URL`: `https://test-api.daglock.com`
4. Custom domain: `test.daglock.com`

### Step 6: Update Mainnet Web UI (Day 1, 2 min)

In the existing Cloudflare Pages project (`daglock.com`):

1. Go to **Settings** → **Environment variables**
2. Set `VITE_API_URL`: `https://api.daglock.com`
3. Trigger a redeploy (or push a commit)

### Step 7: Telegram Bots (Day 7, after testnet week)

**Testnet bot (Day 1 or Day 7):**
1. Go to [@BotFather](https://t.me/BotFather)
2. `/newbot` → name: `DagLock Test`, username: `DagLock_test_bot`
3. Deploy on Railway as a third service (or locally):
   ```bash
   BOT_TOKEN=<test-token> INDEXER_URL=https://test-api.daglock.com node bot/src/index.js
   ```
4. Add bot description: "🧪 TESTNET — DagLock escrow on Kaspa Testnet 12. No real money."

**Mainnet bot (Day 7):**
1. `/newbot` → name: `DagLock`, username: `DagLock_bot`
2. Same code, same deployment method, different env vars:
   ```bash
   BOT_TOKEN=<mainnet-token> INDEXER_URL=https://api.daglock.com node bot/src/index.js
   ```

---

## 4. Summary of Services

| Service | Railway | Domain | Purpose |
|---------|---------|--------|---------|
| Indexer (testnet) | `daglock-indexer-testnet` | `test-api.daglock.com` | Testnet API |
| Indexer (mainnet) | `daglock-indexer-mainnet` | `api.daglock.com` | Mainnet API |
| Web (testnet) | Cloudflare Pages | `test.daglock.com` | Testnet UI |
| Web (mainnet) | Cloudflare Pages | `daglock.com` | Mainnet UI |
| Bot (testnet) | Railway or other | `@DagLock_test_bot` | Testnet Telegram |
| Bot (mainnet) | Railway or other | `@DagLock_bot` | Mainnet Telegram |

### Ongoing Costs

| Item | Cost |
|---|---|
| Railway Service A (mainnet indexer) | Included in Railway plan |
| Railway Service B (testnet indexer) | ~$5/mo (smaller tier) |
| Cloudflare Pages (2 projects) | Free |
| Custom domains (2 subdomains) | Free |
| Telegram bots (2) | Free |
| **Total** | **~$5-10/mo** |

---

## 5. Deployment Calendar

```
Day 1  ████████░░░░░░░░░░░░░░░░░░░░░░  Set up testnet + mainnet infra
Day 1  ░░░░░░░░████████░░░░░░░░░░░░░░  Deploy testnet, announce on Reddit
Day 2  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  Watch testnet feedback, fix bugs
Day 3  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  Iterate on feedback
Day 4  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  Fix issues found
Day 5  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  Verify fixes on testnet
Day 6  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  Final checks
Day 7  ░░░░░░░░░░░░░░░░░░░░░░░████████  Mainnet launch + mainnet bot live
```

**Day 1 tasks:**
- [ ] Rename current Railway service to `daglock-indexer-testnet`
- [ ] Create `daglock-indexer-mainnet` Railway service
- [ ] Add custom domains (`api.daglock.com`, `test-api.daglock.com`)
- [ ] Deploy testnet web UI at `test.daglock.com`
- [ ] Update mainnet web UI env var to point at `api.daglock.com`
- [ ] Post on Reddit with test wallet and `test.daglock.com` link
- [ ] (Optional) Deploy `@DagLock_test_bot`

**Day 7 tasks:**
- [ ] Verify no critical bugs found during testnet week
- [ ] Announce DagLock mainnet is live
- [ ] Deploy `@DagLock_bot` pointing at mainnet
- [ ] Leave testnet infra running for ongoing testing

### Early Exit (if nobody uses testnet)

If after Day 3-4 there's zero testnet activity, just launch mainnet early. The infra stays up for CI/testing regardless.

---

## 6. Code Changes Required

### None to the indexer

The `--network` and `--allow-mainnet` flags already exist. The same binary handles both networks.

### None to the web UI

The API URL is an environment variable (`VITE_API_URL`). Two Cloudflare Pages projects with different env vars, same build.

### .env.example — add network clarification

```env
# Network (default: testnet-12). Set to "mainnet" for production.
# Requires --allow-mainnet flag when network is mainnet.
# DAGLOCK_NETWORK=mainnet
```

Already there. No changes needed.

---

## 7. Connecting to a Real Kaspa Node (Post-Launch)

To enable on-chain UTXO detection (automatic escrow state tracking), either service can be pointed at a Kaspa node:

**Option A: Public kaspa-fy node**
```bash
daglock-indexer --wrpc-url wss://mainnet.kaspa-fy.com:17110 --network mainnet --allow-mainnet ...
```

**Option B: Run your own kaspad**
```bash
# On a separate VPS or Railway sidecar
kaspad --utxoindex --rpclisten=0.0.0.0

# Point indexer at it
daglock-indexer --wrpc-url ws://kaspad:17110 --network mainnet --allow-mainnet ...
```

**Option C: Keep offline mode**

The indexer works fully without a Kaspa node. Offers, reputation, messaging, jury — all stored in SQLite. On-chain UTXO detection is optional. For testnet, offline mode is fine. For mainnet, consider adding node connectivity within the first month so escrows auto-detect on-chain state.

---

## 8. Test Wallet for Reddit Post

Generate a throwaway test wallet:

```bash
python3 scripts/genkeys.py generate
```

Post format:

```
🧪 DagLock testnet is live on Kaspa Testnet 12

Try it at: https://test.daglock.com
API/CLI: https://test-api.daglock.com

Test wallet address: kaspa:qdyzkrhd74v6cetrv4fhv
(It's testnet KAS with no value — DM me for the private key if you want to sign transactions)

Just create an offer, check reputation, or browse escrows.
Tell me what breaks or what's confusing.

Mainnet launch planned in about a week.
```

---

## 9. Notes

- **Testnet indexer doesn't need `--allow-mainnet`** because its network is `testnet-12`. This is a safety guard: if someone fat-fingers the config, mainnet UTXOs won't appear in the testnet database.
- **Both indexers share the same Dockerfile.** The only difference is the start command flags.
- **SQLite is fine for both.** Each service has its own volume. No shared state.
- **The testnet web UI doesn't need to be polished.** It's for debugging. A banner saying "🧪 TESTNET — No real money" at the top of the page is enough.
- **If testnet volume is very low**, you can merge both indexers into one box by running two processes on different ports. But two Railway services is simpler and only costs ~$5/mo.
