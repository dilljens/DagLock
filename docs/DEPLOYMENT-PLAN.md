# DagLock Deployment Plan — Testnet + Mainnet

> Dual-network deployment strategy. Toccata activates June 30, 2026. That gives ~3.5 weeks of testnet feedback before mainnet launch with shared infrastructure.
>
> **Key date:** Kaspa Toccata hard fork activates at DAA score 474,165,565 (~June 30, 16:15 UTC).
> Covenants become available on mainnet at that point. DagLock mainnet launches same day.

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

### 4-Week Timeline (June 5 → June 30)

```
Week 1 (Jun 5-11)  ████████░░░░░░░░░░░░░░  Set up infra, announce testnet
Week 2 (Jun 12-18)  ░░░░░░░░████████░░░░░░  Collect feedback, fix bugs
Week 3 (Jun 19-25)  ░░░░░░░░░░░░░░░░████░░  Iterate, prep mainnet config
Week 4 (Jun 26-30)  ░░░░░░░░░░░░░░░░░░░░██  Mainnet launch on Toccata day
```

### Week 1 — June 5 to June 11: Set Up + Announce

- [ ] Rename current Railway service to `daglock-indexer-testnet`
- [ ] Create `daglock-indexer-mainnet` Railway service (starts dormant, ready to go)
- [ ] Add custom domains (`api.daglock.com`, `test-api.daglock.com`)
- [ ] Deploy testnet web UI at `test.daglock.com`
- [ ] Update mainnet web UI env var to point at `api.daglock.com` (won't be live yet)
- [ ] Deploy `@DagLock_test_bot`
- [ ] Post on Reddit with test wallet and `test.daglock.com` link
- [ ] Verify the whole flow works end-to-end on Testnet 12

### Week 2 — June 12 to June 18: Feedback + Fixes

- [ ] Monitor testnet usage, watch logs
- [ ] Fix any bugs reported
- [ ] Fix UI friction points
- [ ] Fix telegram bot issues
- [ ] No pressure — if nobody's testing, that's fine. Keep iterating.

### Week 3 — June 19 to June 25: Iterate + Prep

- [ ] Polish mainnet config
- [ ] Test mainnet indexer against a test Kaspa mainnet node if available
- [ ] Register `@DagLock_bot` on BotFather (keep it dormant)
- [ ] Set up mainnet Cloudflare Pages project (keep it on a holding page or password)
- [ ] Write the mainnet announcement post

### Week 4 — June 26 to June 30: Mainnet Launch

- [ ] **June 28:** Flip `daglock.com` to point at mainnet API. Put up countdown page.
- [ ] **June 30 (~16:15 UTC):** Toccata activates. Covenants go live on mainnet.
- [ ] Deploy `@DagLock_bot` pointing at `api.daglock.com`
- [ ] Announce: "DagLock is live on Kaspa mainnet"
- [ ] Leave testnet infra running for ongoing debugging

**The beauty of this schedule:** If Toccata gets delayed further, you lose nothing. Testnet keeps running, mainnet infra sits ready. You flip the switch when they flip theirs.

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

Mainnet launches when Toccata activates (~June 30).
Plenty of time to test before then.
```

---

## 9. Notes

- **Testnet indexer doesn't need `--allow-mainnet`** because its network is `testnet-12`. This is a safety guard: if someone fat-fingers the config, mainnet UTXOs won't appear in the testnet database.
- **Both indexers share the same Dockerfile.** The only difference is the start command flags.
- **SQLite is fine for both.** Each service has its own volume. No shared state.
- **The testnet web UI doesn't need to be polished.** It's for debugging. A banner saying "🧪 TESTNET — No real money" at the top of the page is enough.
- **No rush.** You have 3.5 weeks before Toccata. Use the time to get real testnet feedback. If nobody uses testnet the first week, that's fine — drop another Reddit post, try Telegram groups. You have the runway.
- **If testnet volume is very low**, you can merge both indexers into one box by running two processes on different ports. But two Railway services is simpler and only costs ~$5/mo.
