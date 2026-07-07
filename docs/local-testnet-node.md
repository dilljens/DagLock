# Plan: Local Testnet-11 Node Setup

> **Goal:** Run `kaspad` (testnet-11) on this machine with `--utxoindex`. Switch `daglock-indexer` from `--no-wrpc` (MockVerifier) to real wRPC verification. Enable anchor tx broadcasting for the on-chain chat feature.
>
> **Prerequisite:** RAM upgrade to 32 GB (scheduled ~July 13). Current 15 GB with 5.6 GB available is too tight for kaspad + indexer + bot.
>
> **Status:** Plan created. Awaiting RAM upgrade.

---

## Requirements

- [ ] **R1** `kaspad` testnet-11 node synced with `--utxoindex`
- [ ] **R2** wRPC Borsh port exposed (17210)
- [ ] **R3** Indexer connects via `--wrpc-url ws://127.0.0.1:17210` instead of `--no-wrpc`
- [ ] **R4** `EscrowVerifier::verify_utxo_exists()` uses real node data (not MockVerifier)
- [ ] **R5** `AnchorService::flush_pending()` broadcasts anchor txs through the node
- [ ] **R6** DAA score tracking for vault sweep + offer expiry uses real node
- [ ] **R7** Verification that mock auth still works alongside real wRPC (for dev/test)
- [ ] **R8** Rollback plan: `--no-wrpc` fallback if node goes down

---

## Pre-resolved Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| **Node binary** | Pre-built release from rusty-kaspa v2.0.1 GitHub | Faster than compiling from source (hours vs minutes) |
| **UTXO index** | `--utxoindex` enabled | Required for `get_utxos_by_outpoints()` verification |
| **Port** | wRPC Borsh on 17210 (testnet default) | Matches existing code defaults |
| **Data dir** | `/data/kaspad/testnet-11` (NVMe, 251 GB free) | Testnet needs ~20-50 GB |
| **Memory** | `kaspad` peaks at ~2-4 GB during IBD | After sync, idle at ~1-2 GB |
| **Pruning** | No pruning — full archive | Needed for UTXO reorg handling |
| **Indexer mode** | `--wrpc-url ws://127.0.0.1:17210` | Local connection, no auth needed |
| **MockVerifier fallback** | Keep `--no-wrpc` flag as fallback | Node could go down; indexer survives |
| **Anchor txs** | Node wallet or external hot key | Simple send-to-self with payload field |

---

## Phase 1: Install kaspad `[ ]`

**Timebox:** 1-2 hours
**Dependency:** RAM upgrade completed (16+ GB available)

### 1.1 — Download pre-built binary `[ ]` [15 min]
- [ ] Download `kaspad` binary from rusty-kaspa v2.0.1 releases:
  ```bash
  # amd64 Linux
  curl -LO https://github.com/kaspanet/rusty-kaspa/releases/download/v2.0.1/kaspad-v2.0.1-linux-amd64.tar.gz
  tar xzf kaspad-v2.0.1-linux-amd64.tar.gz
  sudo install kaspad-v2.0.1/bin/kaspad /usr/local/bin/
  ```
- [ ] Verify binary: `kaspad --version`
- ✅ **Checkpoint:** `kaspad --version` prints v2.0.1
- ⚙ **Fallback:** Build from source with `cargo install --git https://github.com/kaspanet/rusty-kaspa --tag v2.0.1 kaspad` (takes 20-40 min)

### 1.2 — Create data directory `[ ]` [5 min]
- [ ] `sudo mkdir -p /data/kaspad/testnet-11`
- [ ] `sudo chown -R $(whoami):$(whoami) /data/kaspad`
- [ ] Verify: `ls -la /data/kaspad/testnet-11`
- ✅ **Checkpoint:** Directory exists and is writable
- ⚙ **Fallback:** Use `~/.kaspad` default directory

### 1.3 — Create systemd service `[ ]` [15 min]
- [ ] Create `/etc/systemd/system/kaspad.service`:
```ini
[Unit]
Description=Kaspa testnet-11 node
After=network.target

[Service]
Type=simple
User=dillon
ExecStart=/usr/local/bin/kaspad \
    --testnet \
    --utxoindex \
    --rpclisten-borsh=0.0.0.0:17210 \
    --appdir /data/kaspad/testnet-11
Restart=always
RestartSec=30
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```
- [ ] `sudo systemctl daemon-reload`
- [ ] `sudo systemctl enable kaspad`
- ✅ **Checkpoint:** Service file exists and is valid
- ⚙ **Fallback:** Run in tmux session for initial testing: `kaspad --testnet --utxoindex --rpclisten-borsh=127.0.0.1:17210`

### 1.4 — Start and monitor sync `[ ]` [2-24 hours IBD]
- [ ] `sudo systemctl start kaspad`
- [ ] Check logs: `journalctl -u kaspad -f`
- [ ] Monitor sync progress:
  ```bash
  # Check if port is listening
  ss -tlnp | grep 17210
  
  # Query DAA score via wRPC (once connected)
  # Use kaspawallet or a simple RPC call
  ```
- [ ] Estimated IBD time:
  - testnet-11: 2-6 hours (smaller chain)
  - Mainnet: 24-48 hours
- [ ] **Stop the indexer during IBD** to free RAM:
  ```bash
  sudo systemctl stop daglock-indexer
  sudo systemctl stop daglock-bot
  ```
- ✅ **Checkpoint:** Node reports `"synced": true` or DAA score advances consistently
- ⚙ **Fallback:** If testnet-11 IBD fails, try testnet-10 (smaller chain, faster sync)

---

## Phase 2: Wire Indexer to Local Node `[ ]`

**Timebox:** 1-2 hours
**Dependency:** Phase 1 (kaspad synced)

### 2.1 — Update indexer args `[ ]` [10 min]
- [ ] Edit indexer systemd service or env:
  ```bash
  # Remove --no-wrpc, add --wrpc-url
  # Before:
  --no-wrpc --network testnet-11
  
  # After:
  --wrpc-url ws://127.0.0.1:17210 --network testnet-11
  ```
- [ ] Add `--anchor-wallet-key` if anchor tx broadcasting is desired:
  ```bash
  # Generate a hot wallet key for anchor txs
  openssl rand -hex 32 > /etc/daglock/anchor-key.txt
  
  # Add flag:
  --anchor-wallet-key $(cat /etc/daglock/anchor-key.txt)
  ```
- ✅ **Checkpoint:** Indexer starts without `--no-wrpc`, connects to local node
- ⚙ **Fallback:** Keep `--no-wrpc --wrpc-url ws://127.0.0.1:17210` — indexer uses wRPC if available, falls back to mock

### 2.2 — Verify UTXO verification `[ ]` [30 min]
- [ ] Run the e2e test script:
  ```bash
  # Create an escrow via API
  curl -X POST http://localhost:8443/v1/escrows \
    -H "Content-Type: application/json" \
    -d '{
      "lock_tx_id": "test_tx_1",
      "buyer_address": "kaspatest:...",
      "amount_sompi": 100000000
    }'
  ```
- [ ] Check indexer logs for: `"verified UTXO"` (not `"mock verification"`)
- [ ] Check `GET /v1/escrows/:id/lock-status` returns `{"confirmed": true/false}`
- ✅ **Checkpoint:** Lock-status reflects real node data, not mock
- ⚙ **Fallback:** MockVerifier remains as fallback — no data loss, just less trust

### 2.3 — Verify anchor tx broadcasting `[ ]` [30 min]
- [ ] Send a chat message on an escrow
- [ ] Check indexer logs for: `"Broadcasting anchor tx"` instead of `"No wRPC client — logging payload"`
- [ ] Check `explorer.kaspa.org` for the anchor transaction (search by tx ID)
- ✅ **Checkpoint:** Anchor tx appears on explorer with readable DLAH payload
- ⚙ **Fallback:** Anchor service logs payload hex — can be manually broadcast

### 2.4 — Verify DAA score tracking `[ ]` [15 min]
- [ ] Check `GET /v1/network` returns real DAA score (not 0)
- [ ] Check vault sweeper logs show correct DAA-based expiration
- ✅ **Checkpoint:** `/v1/network` shows `daa_score: <non-zero>`
- ⚙ **Fallback:** DAA score shows 0 — listener may not have processed first block yet

---

## Phase 3: Restart Bot + Monitoring `[ ]`

**Timebox:** 30 min
**Dependency:** Phase 2

### 3.1 — Restart bot services `[ ]` [5 min]
- [ ] `sudo systemctl start daglock-bot`
- [ ] Verify bot connects: `journalctl -u daglock-bot -f`
- ✅ **Checkpoint:** Bot responds to `/start`

### 3.2 — Resource monitoring `[ ]` [15 min]
- [ ] Set up monitoring:
  ```bash
  # Check kaspad mem usage
  ps aux | grep kaspad | awk '{print $6/1024 " MB"}'
  
  # Check total system mem
  free -h
  
  # Wire up a simple alert: if kaspad crashes, restart it
  # (already in systemd Restart=always)
  ```
- [ ] Target: kaspad idle at ~1-2 GB, indexer at ~200 MB, bot at ~100 MB
- ✅ **Checkpoint:** Total memory usage < 12 GB (leaving room for OS + swap)
- ⚙ **Fallback:** If kaspad uses too much RAM, add `--max-invocations` or prune settings

### 3.3 — Rollback plan `[ ]` [10 min]
- [ ] If node causes issues, rollback is safe:
  ```bash
  sudo systemctl stop kaspad
  # Restart indexer with --no-wrpc
  # (edit service file to add --no-wrpc back)
  sudo systemctl restart daglock-indexer
  ```
- [ ] No data loss — indexer DB is separate from kaspad data
- ✅ **Checkpoint:** Rollback tested and verified
- ⚙ **Fallback:** If rollback fails, `cargo run -p daglock-indexer -- --no-wrpc --mock-auth` works in terminal

---

## Phase 4: Update Config + Docs `[ ]`

**Timebox:** 1 hour
**Dependency:** Phase 2 working

### 4.1 — Update systemd service files `[ ]` [15 min]
- [ ] Update `/etc/systemd/system/daglock-indexer.service`:
  ```
  ExecStart=/usr/local/bin/daglock-indexer \
      --host 0.0.0.0 --port 8443 \
      --network testnet-11 \
      --wrpc-url ws://127.0.0.1:17210 \
      --database-url sqlite:/data/daglock/daglock.db \
      --daglock-kas-template <hash> \
      --daglock-krc20-template <hash> \
      --daglock-vault-softlock-template <hash> \
      --daglock-vault-multisig-template <hash> \
      ...
  ```
- [ ] Remove `--no-wrpc` from all service configs
- ✅ **Checkpoint:** `sudo systemctl daemon-reload && sudo systemctl restart daglock-indexer`
- ⚙ **Fallback:** Keep old service file as backup

### 4.2 — Update deployment docs `[ ]` [15 min]
- [ ] Update `docs/DEPLOYMENT.md`:
  - Default `--wrpc-url ws://127.0.0.1:17210` instead of `--no-wrpc`
  - Add kaspad systemd service info
  - Note resource requirements (16+ GB RAM recommended)
- ✅ **Checkpoint:** DEPLOYMENT.md reflects local node setup

### 4.3 — Update wiki docs `[ ]` [15 min]
- [ ] Update `docs/wiki/features/indexer.md`:
  - Change: `--no-wrpc` → ws://127.0.0.1:17210
  - Remove MockVerifier note
- ✅ **Checkpoint:** Wiki docs updated

---

## Appendix: Resource Estimates

| Component | RAM (IBD) | RAM (Idle) | Disk | CPU |
|-----------|:---------:|:----------:|:----:|:---:|
| kaspad (testnet-11) | 2-4 GB | 1-2 GB | 20-50 GB | 2-4 cores |
| daglock-indexer | — | 200-400 MB | 1 GB | 1 core |
| daglock-bot | — | 100 MB | 100 MB | <1 core |
| **Total** | **3-5 GB** | **1.5-3 GB** | **25-55 GB** | **4-6 cores** |

**After RAM upgrade to 32 GB:**
- Total system RAM: 32 GB
- kaspad: 2-4 GB (6-12%)
- Indexer + bot: 0.5 GB (1.5%)
- Free for OS + other: ~27 GB (84%)
- Comfortable margin ✅

## Appendix: Verification Commands

```bash
# Check kaspad is running
curl -s http://127.0.0.1:17210/health 2>/dev/null || ss -tlnp | grep 17210

# Check DAA score
curl -s http://localhost:8443/v1/network | jq .daa_score

# Check UTXO verification
curl -s http://localhost:8443/v1/escrows/some-id/lock-status | jq .

# Check anchor broadcast
curl -s http://localhost:8443/v1/escrows/some-id/messages | jq '.messages[0].anchor_tx_id'

# Check indexer logs for real verification
journalctl -u daglock-indexer --since "5 min ago" | grep -i "verify\|utxo\|anchor\|mock"

# Check memory usage
ps aux | grep -E "kaspad|daglock" | awk '{printf "%-20s %s MB\n", $11, $6/1024}'
```

## Appendix: ANTI-SCOPE

- Mainnet node (needs 100+ GB disk, 4-8 GB RAM — separate plan)
- Pruning configuration (testnet is small enough for full archive)
- Grafana dashboard (would be nice, not required)
- Public wRPC endpoint (this node is local-only)
- TLS/wSS for wRPC (localhost connection doesn't need encryption)
