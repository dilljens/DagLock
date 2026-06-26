# Kaspad: Deploy + Wire Indexer + Real UTXO Verification

**Goal:** Deploy kaspad testnet-12 node on the OVH VPS, switch daglock-indexer from `--no-wrpc` (MockVerifier) to real wRPC verification, verify everything works end-to-end, then update the testnet announcement plan.

## Requirements
- [ ] R1: kaspad testnet-12 node running on VPS with UTXO index
- [ ] R2: daglock-indexer connecting to local kaspad via `--wrpc-url ws://127.0.0.1:17210`
- [ ] R3: Health endpoint shows `node_synced: true` + real DAA score
- [ ] R4: All 19 API endpoints responding correctly after the switch
- [ ] R5: Web UI create → KasWare sign → broadcast → settle flow works with real detection
- [ ] R6: Pre-announcement checklist updated to reflect that MockVerifier is gone

## Pre-resolved Decisions
- **kaspad port**: `127.0.0.1:17210` (testnet-12 default Borsh wRPC, localhost only — no firewall changes needed)
- **UTXO index**: `--utxoindex --max-tracked-addresses 1000` for `get_utxos_by_addresses`
- **Sync strategy**: Deploy kaspad first, let it sync in background, wire indexer after sync completes
- **User**: `kaspad` system user (non-root, matching `daglock` user pattern)
- **Binary**: Pre-built locally (same glibc 2.43), 42 MB, SCP to VPS
- **No firewall changes**: kaspad only listens on 127.0.0.1, no external exposure

## Track A: Deploy & Sync kaspad `[ ]`

### Phase A1: Transfer binary & create service `[ ]` ⏱ 15 min
- [ ] SCP binary to VPS: `scp /tmp/rusty-kaspa/target/release/kaspad root@40.160.241.74:/opt/kaspad/kaspad`
- [ ] Create `kaspad` system user
- [ ] Create `/etc/systemd/system/kaspad.service`
- [ ] Start and enable kaspad
- [ ] Verify listening on port 17210
- ✅ Checkpoint: `ssh root@40.160.241.74 "ss -tlnp | grep 17210"` shows kaspad listening
- ⚙ Fallback: Check `journalctl -u kaspad -n 50` for startup errors

### Phase A2: Monitor sync progress `[ ]` ⏱ 2 hours (background)
- [ ] Tail `journalctl -u kaspad -f` for IBD headers
- [ ] Verify DAA score is increasing over time
- ✅ Checkpoint: `journalctl -u kaspad --since "5 min ago" | grep -i "ibd\|synced\|header"` shows active sync
- ⚙ Fallback: If no sync activity after 15 min, check DNS/peers: `journalctl -u kaspad -n 100 | grep -i "peer\|connect\|error"`

## Track B: Wire Indexer to kaspad `[ ]`

### Phase B1: Update service config `[ ]` ⏱ 5 min
Only start after kaspad sync is confirmed (Phase A2 complete).

- [ ] Edit `/etc/systemd/system/daglock-indexer.service`: replace `--no-wrpc` with `--wrpc-url ws://127.0.0.1:17210`
- [ ] `systemctl daemon-reload && systemctl restart daglock-indexer`
- ✅ Checkpoint: `journalctl -u daglock-indexer -n 30 | grep -i "wrpc\|verifier\|mock"` shows "wRPC verifier connected" (NOT "mock verifier")
- ⚙ Fallback: If indexer fails to connect, check that kaspad is listening: `ss -tlnp | grep 17210`. The indexer falls back to MockVerifier gracefully, so the API stays up.

### Phase B2: Verify health endpoint `[ ]` ⏱ 5 min
- [ ] `curl -s https://api.daglock.com/v1/health` — `node_synced` should be `true`, `node_daa_score` should be > 0
- [ ] `curl -s https://api.daglock.com/v1/network` — DAA score, BPS, block count all populated
- [ ] `curl -s https://api.daglock.com/v1/stats` — still works, escrow data intact
- ✅ Checkpoint: Health returns `{"status":"ok","node_synced":true,"node_daa_score":12345,...}`
- ⚙ Fallback: If node_synced is still false, wait for kaspad IBD to finish. Run `journalctl -u kaspad -f` to check.

## Track C: E2E Verification & Announcement Update `[ ]`

### Phase C1: API smoke test `[ ]` ⏱ 15 min
- [ ] `curl -s https://api.daglock.com/v1/openapi.json | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'{len(d[\"paths\"])} endpoints')"` — expect 19
- [ ] Hit a handful of key endpoints: health, status, stats, offers, escrows, network, price
- [ ] Rate limiter check: 31 rapid requests, 31st should return 429
- [ ] Offer board still serving existing offers
- ✅ Checkpoint: All key endpoints return 200 with valid JSON
- ⚙ Fallback: Check indexer logs if any endpoint errors: `journalctl -u daglock-indexer --since "5 min ago" -n 50`

### Phase C2: Manual E2E flow `[ ]` ⏱ 20 min
- [ ] Open `daglock.com` in incognito, connect KasWare (testnet) wallet
- [ ] Create escrow → sign in KasWare → submit tx_id
- [ ] Verify escrow appears in list with correct status transition
- [ ] Settle → sign → download receipt
- ✅ Checkpoint: Full round-trip create→sign→broadcast→settle→receipt works
- ⚙ Fallback: If any step fails, diagnose at the specific surface (web UI, indexer API, wallet signing)

### Phase C3: Update pre-announcement docs `[ ]` ⏱ 10 min
- [ ] Update `.opencode/plans/pre-announcement.md` — remove MockVerifier caveats, update demo video script, update VPS IP references
- [ ] Update `docs/wiki/plans/testnet-launch.md` — Phase 1/2 checkboxes populated for this deployment
- ✅ Checkpoint: No remaining references to MockVerifier in announcement/launch docs (technical docs can keep it for history)
- ⚙ Fallback: Just note the change — no need to rewrite everything
