# Testnet Launch Plan

**Status:** Not started

**Date:** 2026-06-16

**Target:** Fully functional testnet on Railway + Hetzner VPS, then announce to Kaspa community.

---

## Overview

```
Phase 0: Prep ──→ Phase 1: VPS ──→ Phase 2: Connect ──→ Phase 3: Verify ──→ Phase 4: Announce
  (1-2 hrs)       (1-2 hrs)         (30 min)              (1-2 hrs)            (1-2 hrs)
```

**Total effort:** ~4-8 hours over a couple days
**Monthly cost:** $5 (Hetzner CX22 VPS)
**Blockers cleared:** All 7 critical audit items fixed, 27/30 tasks done

---

## Phase 0: Prep (Before We Touch Anything)

Run these checks first so we know the starting point is solid.

### 0.1 — Verify tests pass

```bash
# All Rust tests (contracts, indexer, cli, wasm-sdk, shared)
cargo test --workspace

# Web tests
cd web && npm test

# Web build check
cd web && npm run build

# Bot tests
cd bot && npm test
```

**Expected:** All pass. If any fail, stop and fix before proceeding.

### 0.2 — Verify template hashes match Railway config

```bash
cargo test -p daglock-contracts -- --nocapture print_template_hashes
```

Then check `railway.json` start command has the same hashes:

| Covenant | Config flag | Current hash |
|----------|-------------|-------------|
| KAS | `--daglock-kas-template` | `30876e3ea42d0e23bb0980f3fd97ae8807e9c70f` |
| KRC-20 | `--daglock-krc20-template` | `8a43a8438d183a92bc7b94337c031196ff16725b` |

> If hashes differ (e.g., after contract changes), update `railway.json` and `.env.example`.

### 0.3 — Confirm Railway environment variables

Check the Railway dashboard for the indexer service:

| Variable | Needed? | Value |
|----------|---------|-------|
| `DAGLOCK_MESSAGE_KEY` | ✅ Required | 64 hex chars |
| `PORT` | ✅ Required | `8443` |
| `RUST_LOG` | Nice | `info` |
| `WRPC_URL` | Phase 2 | (will add later) |

### 0.4 — Check current Railway health

```bash
curl https://daglock-production.up.railway.app/v1/health
```

Expected: `{"status":"ok","db_connected":true,...}`

### 0.5 — Verify web UI loads

Open `https://daglock.com` in a browser. Should show the DagLock dashboard.

### 0.6 — Verify bot responds

Open Telegram, find `@DagLock_bot`, send `/start`. Should get a welcome message.

---

## Phase 1: Provision a Kaspa Node (VPS)

The indexer needs a testnet-12 node to talk to. Public resolver nodes are down (wRPC v2 migration), so we run our own on a cheap VPS.

### 1.1 — Sign up for Hetzner (if needed)

1. Go to [hetzner.com/cloud](https://hetzner.com/cloud)
2. Create account (email + payment)
3. Cost: ~$5/mo for CX22

### 1.2 — Create the server

1. Hetzner Cloud Console → **New Project** → name it `daglock`
2. **Add Server:**
   - Image: **Ubuntu 24.04**
   - Type: **CX22** (2 vCPU, 4GB RAM, 40GB SSD) — $5/mo
   - Location: closest to you
   - SSH Key: add your public key (`cat ~/.ssh/id_ed25519.pub`)
3. Click **Create & Buy**
4. Copy the IP address (e.g., `123.45.67.89`)

### 1.3 — SSH in and install deps

```bash
ssh root@<VPS_IP>

apt update && apt install -y build-essential clang lld llvm-dev libclang-dev \
  pkg-config libssl-dev curl git
```

### 1.4 — Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
```

### 1.5 — Build kaspad

```bash
git clone --depth 1 https://github.com/kaspanet/rusty-kaspa /opt/kaspad
cd /opt/kaspad
cargo build --release --bin kaspad
```

> Takes ~15-20 minutes. Binary goes to `/opt/kaspad/target/release/kaspad`.

### 1.6 — Create systemd service

```bash
cat > /etc/systemd/system/kaspad.service << 'EOF'
[Unit]
Description=Kaspa Node (Testnet-12)
After=network.target

[Service]
Type=simple
User=root
ExecStart=/opt/kaspad/target/release/kaspad --testnet --rpclisten-borsh=0.0.0.0:16610 --rpclisten-json=0.0.0.0:16611
Restart=always
RestartSec=10
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable kaspad
systemctl start kaspad
```

### 1.7 — Open firewall

```bash
ufw allow 16610/tcp   # Borsh wRPC (for Railway indexer)
ufw allow 16611/tcp   # JSON wRPC
ufw allow 22/tcp      # SSH
ufw enable
```

### 1.8 — Verify kaspad is running

```bash
systemctl status kaspad
# Should show "active (running)"

journalctl -u kaspad -f --tail=10
# Should show block headers syncing
```

> Sync takes 1-2 hours for testnet-12 (IBD). The wRPC endpoint is available during sync, but UTXO queries won't return useful data until fully synced.

### 1.9 — Test wRPC from your local machine

```bash
# Install websocat or use a Python wRPC client
# Quick ping test:
timeout 5 bash -c "echo > /dev/tcp/<VPS_IP>/16610" && echo "Port open" || echo "Port closed"
```

---

## Phase 2: Wire the Indexer to the Node

### 2.1 — Update Railway start command

In Railway dashboard → indexer service → **Variables**:

| Variable | Value |
|----------|-------|
| `WRPC_URL` | `ws://<VPS_IP>:16610` |

In Railway → **Deploy** → **Start Command**, change:

```diff
- daglock-indexer --host 0.0.0.0 --port 8443 --database-url sqlite:/data/daglock.db --network testnet-12 --daglock-kas-template 30876e3ea42d0e23bb0980f3fd97ae8807e9c70f --daglock-krc20-template 8a43a8438d183a92bc7b94337c031196ff16725b --cors-origin https://daglock.com --no-wrpc
+ daglock-indexer --host 0.0.0.0 --port 8443 --database-url sqlite:/data/daglock.db --network testnet-12 --daglock-kas-template 30876e3ea42d0e23bb0980f3fd97ae8807e9c70f --daglock-krc20-template 8a43a8438d183a92bc7b94337c031196ff16725b --cors-origin https://daglock.com --wrpc-url ws://<VPS_IP>:16610
```

Also update `railway.json` to match (so future deploys keep the change).

### 2.2 — Redeploy and check logs

Railway auto-rebuilds on start command change. Wait 2-3 minutes, then check logs for:

```
INFO wRPC verifier connected to ws://<VPS_IP>:16610
INFO wRPC listener starting for testnet-12 at ws://<VPS_IP>:16610
INFO Connected to Kaspa node at ws://<VPS_IP>:16610
```

### 2.3 — Verify health endpoint

```bash
curl https://daglock-production.up.railway.app/v1/health
```

Expected includes `"node_synced": true` (may be `false` until kaspad finishes IBD).

### 2.4 — Verify network info

```bash
curl https://daglock-production.up.railway.app/v1/network
```

Expected: `{"network":"testnet-12","daa_score":<number>,...}`

### 2.5 — Verify price endpoint

```bash
curl https://daglock-production.up.railway.app/v1/network/price
```

Should return KAS/USD price (from CoinGecko cache).

---

## Phase 3: Verify End-to-End

Run through every surface to confirm it all works.

### 3.1 — Health check suite

```bash
# Run the 16-item checklist
# See docs/manual-verification-plan.md for details

# Quick smoke tests:
curl -s https://daglock-production.up.railway.app/v1/health | python3 -m json.tool
curl -s https://daglock-production.up.railway.app/v1/status | python3 -m json.tool
curl -s https://daglock-production.up.railway.app/v1/stats | python3 -m json.tool
curl -s https://daglock-production.up.railway.app/v1/network | python3 -m json.tool
```

### 3.2 — Web: Create an escrow end-to-end

Requires Chrome with KasWare extension + testnet KAS.

1. Open `https://daglock.com`
2. Connect KasWare wallet
3. Go to **Escrows** → **Create**
4. Enter 10 KAS, a seller address (can be a second wallet you control)
5. Click Create
6. **Verify:** KasWare prompts to send KAS to a `kaspatest:` address
7. Approve in KasWare
8. **Verify:** Escrow appears in list with `pending_confirmation` status
9. Wait for listener to detect the UTXO → status changes to `active`
10. Click Settle → sign in KasWare → status changes to `settled`
11. **Verify:** Settlement receipt downloadable

### 3.3 — Web: Create an offer

1. Go to **Offers** → **Create Offer**
2. Enter amount, price, expiry
3. **Verify:** Offer appears on the board
4. From a different wallet → Accept the offer
5. **Verify:** Escrow created from offer

### 3.4 — Web: Atomic swap

1. Go to **Swap** → **Generate**
2. Click "Generate Secret & Hash"
3. **Verify:** Secret + hash displayed
4. Copy hash, create escrow with it
5. Go to **Submit** → enter preimage
6. **Verify:** Swap settles via hash preimage

### 3.5 — Web: Jury + dispute

1. Create an escrow with dispute mode = jury
2. Dispute it
3. **Verify:** Available jurors shown
4. Vote as a juror

### 3.6 — CLI: Create an escrow

```bash
# Requires kaspawallet installed
cargo run -p daglock-cli -- create \
  --amount 100 \
  --counterparty kaspa:<partner_address>
```

**Verify:** Covenant address printed, unsigned tx assembled.

### 3.7 — CLI: Check reputation

```bash
cargo run -p daglock-cli -- reputation <address>
```

### 3.8 — Bot: /create wizard

1. In Telegram, send `/create` to `@DagLock_bot`
2. Follow the 4-step wizard
3. **Verify:** Deep link opens wallet for signing

### 3.9 — Bot: Check commands

```bash
/start
/list
/status <id>
/help
```

### 3.10 — API key registration

```bash
# Register an app
curl -X POST https://daglock-production.up.railway.app/v1/apps/register \
  -H "Content-Type: application/json" \
  -d '{"name": "TestApp", "owner_address": "kaspa:test"}'

# Use the returned key to access app details
curl https://daglock-production.up.railway.app/v1/apps/<app_id> \
  -H "X-Daglock-Api-Key: <api_key>"
```

### 3.11 — Rate limiting

```bash
# Send 31 rapid requests — the 31st should get 429
for i in $(seq 1 31); do
  curl -s -o /dev/null -w "%{http_code}\n" \
    https://daglock-production.up.railway.app/v1/health
done | tail -5
```

### 3.12 — OpenAPI spec

```bash
curl -s https://daglock-production.up.railway.app/v1/openapi.json | python3 -c \
  "import sys,json; d=json.load(sys.stdin); print(f'{len(d[\"paths\"])} endpoints')"
```

---

## Phase 4: Announce

Only start this after Phase 3 passes. If the flow breaks, fix first.

### 4.1 — Pre-announcement checks

- [ ] Phase 1-3 all pass
- [ ] `cargo test --workspace` passes
- [ ] `cd web && npm test && npm run build` passes
- [ ] Manual end-to-end flow works (web → KasWare → broadcast → settle)
- [ ] Bot responds and wizard works
- [ ] Rate limiter protecting API
- [ ] Error messages are user-friendly (not stack traces)

### 4.2 — Demo video

Record a 30-60 second screen capture showing:

1. Open `daglock.com` → Connect KasWare wallet
2. Create escrow → Sign in KasWare
3. Escrow appears in list
4. Settle → Signature → Done

Post to Twitter/X, embed in Telegram announcement.

### 4.3 — Telegram announcements

Post in these channels (ask permission first where needed):

- **Kaspa main chat** — "Hey, we built trustless escrow for Kaspa. Testnet is live. Here's how it works…"
- **KRC-20 token groups** (GHOST, NACHO, KASPY, etc.) — "Escrow for your KRC-20 trades. Try it on testnet: @DagLock_bot"
- **Kaspa DeFi / builders** — "We're open source, audited, and looking for feedback on testnet"

### 4.4 — Twitter/X

- Tag @KaspaCurrency, @KaspaCommunity
- Include demo video
- Link to `daglock.com`

### 4.5 — Monitor feedback

After announcing:
- Watch bot DMs for bug reports
- Check indexer logs for errors
- Track escrow creation volume (via `/v1/stats`)
- Fix issues as they come up

---

## Rollback Plan

If something breaks:

1. **wRPC connection lost:** Railway auto-reconnects. If persistent, SSH into VPS and restart kaspad (`systemctl restart kaspad`).
2. **Indexer crash:** Railway auto-restarts. Check logs for the cause.
3. **Web UI broken:** Cloudflare can redeploy a previous version.
4. **Worst case:** Revert Railway start command to `--no-wrpc` (offline mode), fix the issue, then re-enable wRPC.

---

## Checklist

### Phase 0: Prep
- [ ] 0.1 — `cargo test --workspace` passes
- [ ] 0.1 — `cd web && npm test && npm run build` passes
- [ ] 0.1 — `cd bot && npm test` passes
- [ ] 0.2 — Template hashes match Railway config
- [ ] 0.3 — Railway env vars are set
- [ ] 0.4 — Railway health endpoint responds
- [ ] 0.5 — Web UI loads at daglock.com
- [ ] 0.6 — Bot responds at @DagLock_bot

### Phase 1: VPS
- [ ] 1.1 — Hetzner account created
- [ ] 1.2 — CX22 server provisioned
- [ ] 1.3 — Dependencies installed
- [ ] 1.4 — Rust installed
- [ ] 1.5 — kaspad compiled
- [ ] 1.6 — kaspad systemd service running
- [ ] 1.7 — Firewall open (16610, 16611, 22)
- [ ] 1.8 — kaspad syncing (may take 1-2 hrs)
- [ ] 1.9 — wRPC port reachable from local machine

### Phase 2: Connect
- [ ] 2.1 — Railway start command updated (no more `--no-wrpc`)
- [ ] 2.1 — `railway.json` updated in repo
- [ ] 2.2 — Railway logs show "Connected to Kaspa node"
- [ ] 2.3 — `/v1/health` shows `node_synced`
- [ ] 2.4 — `/v1/network` returns DAA score + network info
- [ ] 2.5 — `/v1/network/price` returns KAS/USD

### Phase 3: Verify
- [ ] 3.1 — Health check suite passes
- [ ] 3.2 — Web: Create escrow → KasWare sign → settle → receipt
- [ ] 3.3 — Web: Create offer → accept → escrow created
- [ ] 3.4 — Web: Atomic swap (generate → submit preimage)
- [ ] 3.5 — Web: Dispute → jury → vote
- [ ] 3.6 — CLI: Create escrow with kaspawallet
- [ ] 3.7 — CLI: Check reputation
- [ ] 3.8 — Bot: /create wizard
- [ ] 3.9 — Bot: Basic commands work
- [ ] 3.10 — API key registration + access
- [ ] 3.11 — Rate limiter kicks in at 31 requests
- [ ] 3.12 — OpenAPI spec returns

### Phase 4: Announce
- [ ] 4.1 — Pre-announcement checks done
- [ ] 4.2 — Demo video recorded
- [ ] 4.3 — Telegram announcements posted
- [ ] 4.4 — Twitter/X post made
- [ ] 4.5 — Monitoring active (logs, stats, bot DMs)
