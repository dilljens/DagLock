# Deploy DagLock — VPS + Railway Split (Option B)

> Cost: ~$5-11/mo (VPS) + Railway free tier · Time: ~1 hour

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                        Users                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                   │
│  │ Telegram │  │   Web    │  │   CLI    │                   │
│  │   Bot    │  │ Dashboard│  │   Tool   │                   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘                   │
│       │             │             │                          │
│       └─────────────┼─────────────┘                          │
│                     │                                        │
└─────────────────────┼────────────────────────────────────────┘
                      │
         ┌────────────▼────────────┐
         │    Hetzner VPS ($5/mo)  │
         │  ┌──────────────────┐   │
         │  │ kaspad (testnet) │   │
         │  │ wRPC on :16610   │   │
         │  └────────┬─────────┘   │
         │           │ wRPC        │
         └───────────┼─────────────┘
                     │
         ┌───────────▼─────────────┐
         │   Railway (free tier)   │
         │  ┌──────────────────┐   │
         │  │ daglock-indexer  │   │
         │  │ --wrpc-url ws:// │   │
         │  │ <VPS_IP>:16610   │   │
         │  └──────────────────┘   │
         │  ┌──────────────────┐   │
         │  │ daglock-bot      │   │
         │  └──────────────────┘   │
         │  ┌──────────────────┐   │
         │  │ SQLite volume    │   │
         │  └──────────────────┘   │
         └─────────────────────────┘
```

## How It Works

1. **VPS** runs a kaspad testnet node — always online, syncing blocks
2. **Railway** runs the indexer + bot — connects to VPS via wRPC
3. The indexer's listener scans blocks from the VPS node
4. When a DagLock lock transaction is detected, the escrow activates
5. Users interact via Telegram bot, web dashboard, or CLI

## Prerequisites

| Account | Sign up at | Cost |
|---------|-----------|------|
| Hetzner | hetzner.com/cloud | $5/mo (CX22) |
| Railway | railway.com | Free tier |
| GitHub | github.com | Free |

---

## Step 1: Set Up Hetzner VPS — 15 minutes

### 1a. Create the server

1. Go to [hetzner.com/cloud](https://hetzner.com/cloud) → Sign up
2. Click **New Project** → name it "daglock"
3. Click **Add Server**:
   - **Image:** Ubuntu 24.04
   - **Location:** Closest to you
   - **Type:** CX22 (2 vCPU, 4GB RAM, 40GB SSD) — $5/mo
   - **SSH Key:** Paste your public key (`cat ~/.ssh/id_ed25519.pub`)
4. Click **Create & Buy**
5. Note the IP address (e.g., `123.45.67.89`)

### 1b. SSH into the VPS

```bash
ssh root@<VPS_IP>
```

### 1c. Install dependencies

```bash
apt update && apt install -y build-essential clang lld llvm-dev libclang-dev \
  pkg-config libssl-dev curl git
```

### 1d. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
```

### 1e. Build kaspad

```bash
git clone --depth 1 https://github.com/kaspanet/rusty-kaspa /opt/kaspad
cd /opt/kaspad
cargo build --release --bin kaspad
```

This takes ~15-20 minutes. The binary will be at `/opt/kaspad/target/release/kaspad`.

### 1f. Create kaspad systemd service

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

### 1g. Open firewall ports

```bash
# Allow wRPC connections from anywhere (for Railway indexer)
ufw allow 16610/tcp  # Borsh wRPC
ufw allow 16611/tcp  # JSON wRPC
ufw allow 22/tcp     # SSH
ufw enable
```

### 1h. Verify kaspad is running

```bash
systemctl status kaspad
# Should show "active (running)"

# Check sync progress
journalctl -u kaspad -f --tail=10
```

Kaspad will take 1-2 hours to sync testnet-12 (IBD). The wRPC endpoint is available during sync.

---

## Step 2: Update Railway Indexer — 5 minutes

### 2a. Get the VPS IP address

```bash
# On your local machine:
curl -s ifconfig.me  # or check Hetzner dashboard
```

### 2b. Update Railway environment variables

In the Railway dashboard → your indexer service → **Variables**:

| Variable | New Value |
|----------|-----------|
| `WRPC_URL` | `ws://<VPS_IP>:16610` |

### 2c. Update the start command

In Railway → **Deploy** → **Settings** → **Deploy Command**:

```
daglock-indexer --host 0.0.0.0 --port $PORT --database-url sqlite:/data/daglock.db --network testnet-12 --daglock-kas-template 30876e3ea42d0e23bb0980f3fd97ae8807e9c70f --daglock-krc20-template 8a43a8438d183a92bc7b94337c031196ff16725b --cors-origin https://daglock.com --wrpc-url $WRPC_URL
```

**Key change:** Removed `--no-wrpc`, added `--wrpc-url $WRPC_URL`.

### 2d. Verify the connection

After Railway redeploys (~2-3 minutes), check the logs:

1. Railway → your service → **Deployments** → **View Logs**
2. You should see:
   ```
   INFO wRPC verifier connected to ws://<VPS_IP>:16610
   INFO wRPC listener starting for testnet-12 at ws://<VPS_IP>:16610
   INFO Connected to Kaspa node at ws://<VPS_IP>:16610
   INFO REST API listening on http://0.0.0.0:8443
   ```

### 2e. Test the API

```bash
curl https://daglock-indexer.up.railway.app/v1/health
# Should show: {"status":"ok","node_synced":true,...}

curl https://daglock-indexer.up.railway.app/v1/network
# Should show: {"network":"testnet-12","daa_score":...}
```

---

## Step 3: Verify End-to-End — 5 minutes

### 3a. Create an escrow

```bash
curl -X POST https://daglock-indexer.up.railway.app/v1/escrows \
  -H "Content-Type: application/json" \
  -d '{
    "lock_tx_id": "tx_test_deploy",
    "lock_tx_output_index": 0,
    "buyer_address": "kaspa:qzyqpzry9x8gf2tvdw0s3jn54khce6mua7l",
    "seller_address": "kaspa:qz8gf2tvdw0s3jn54khce6mua7lqzyqpy3",
    "amount_sompi": 100000000
  }'
```

### 3b. Check the status

```bash
curl https://daglock-indexer.up.railway.app/v1/escrows/<escrow_id>
# Status should be "pending_confirmation"
```

### 3c. Monitor for activation

Once kaspad finishes syncing and the listener detects the lock transaction (if it exists on-chain), the escrow will transition to `active`.

---

## Step 4: Update Web UI (Optional)

If you're using Cloudflare Pages for the web dashboard, update the API URL:

1. Cloudflare Pages → your project → **Settings** → **Environment variables**
2. Set `VITE_API_URL` to `https://daglock-indexer.up.railway.app`
3. Redeploy

---

## Monitoring

### Check VPS status
```bash
ssh root@<VPS_IP>
systemctl status kaspad
journalctl -u kaspad -f --tail=20
```

### Check Railway status
```bash
curl https://daglock-indexer.up.railway.app/v1/health
curl https://daglock-indexer.up.railway.app/v1/network
```

### Check listener activity
```bash
# On Railway, view logs for:
# - "DAA progressed:" — block scanning active
# - "Activated escrow" — lock transaction detected
# - "Scanned N block(s)" — scanning working
```

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Indexer shows `node_synced: false` | VPS not reachable | Check firewall, verify port 16610 is open |
| "wRPC connection lost" in logs | kaspad restarted | Railway auto-reconnects in 30s |
| Escrows stay `pending_confirmation` | kaspad still syncing | Wait for IBD to complete (~1-2 hours) |
| "Resolver connection timed out" | Old resolver nodes down | Ignore — we use direct wRPC, not resolver |
| High VPS CPU during IBD | Normal during sync | Drops to idle after sync completes |

---

## Cost Summary

| Component | Monthly Cost | Notes |
|-----------|-------------|-------|
| Hetzner CX22 VPS | $5 | 2 vCPU, 4GB RAM, 40GB SSD |
| Railway (free tier) | $0 | 500 hours/mo, 1GB RAM |
| Cloudflare Pages | $0 | Free tier |
| **Total** | **$5/mo** | |

## Scaling for Mainnet

When moving to mainnet:
1. Upgrade VPS to CX32 (4 vCPU, 8GB RAM, 80GB SSD) — $11/mo
2. Change `--network testnet-12` to `--network mainnet`
3. Add `--allow-mainnet` flag
4. Set `DAGLOCK_MESSAGE_KEY` env var
5. Update template hashes if contracts change
