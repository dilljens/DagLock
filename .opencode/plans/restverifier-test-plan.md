# Track A: Test RestVerifier on Testnet-11

**Goal:** Deploy updated indexer with `--kaspa-api-url https://api-tn11.kaspa.org`, create a real testnet escrow, verify UTXO check works end-to-end.

⏱ **Timebox:** 4 hours

---

## Phase A1: Build & Deploy `[ ]`

- [ ] `cargo build --release -p daglock-indexer` on dev machine
- [ ] SSH to VPS (`ssh ubuntu@40.160.241.74`)
- [ ] Copy binary to `/opt/daglock-indexer/daglock-indexer` (backup old first)
- [ ] Update systemd: `sudo systemctl edit daglock-indexer.service` — add `--kaspa-api-url https://api-tn11.kaspa.org` to ExecStart
- [ ] `sudo systemctl daemon-reload && sudo systemctl restart daglock-indexer`
- [ ] Check logs: `journalctl -u daglock-indexer -n 50 | grep -i "rest\|kaspa\|api"`
- ✅ **Checkpoint:** `journalctl -u daglock-indexer | grep "Using Kaspa REST API"` shows verifier started
- ⚙ **Fallback:** If build fails, `cargo check` first; if VPS unreachable, verify SSH key/config

---

## Phase A2: Create Testnet Escrow `[ ]`

- [ ] Ensure KasWare is on testnet-11 (Settings → Network)
- [ ] Get test KAS from faucet: `https://faucet-tn11.kaspanet.io/`
- [ ] Open `https://daglock.com` — create an offer or direct escrow
- [ ] Sign with KasWare, broadcast the lock tx
- [ ] Submit TX ID to indexer (web UI or API)
- ✅ **Checkpoint:** Escrow appears with status `pending_confirmation` or `active` in API
- ⚙ **Fallback:** If KasWare fails, use CLI + kaspawallet; if faucet down, ask in Kaspa Discord #testnet

---

## Phase A3: Verify UTXO Check `[ ]`

- [ ] Check indexer logs: `journalctl -u daglock-indexer | grep "RestVerifier"`
- [ ] Confirm `RestVerifier: UTXO found for escrow` appears
- [ ] Try settling: sign settle message in KasWare → `/submit_sig` in bot (or web)
- [ ] Confirm settle tx broadcasts and status changes to `settled`
- [ ] Check receipt: `/receipt <id>` shows valid data
- ✅ **Checkpoint:** Full lifecycle: create → verify → settle → receipt
- ⚙ **Fallback:** If UTXO not found, curl the API directly to debug format; if settle fails, check MockVerifier vs SchnorrVerifier

---

## Rollback

```bash
# If RestVerifier causes issues, revert to MockVerifier:
sudo systemctl edit daglock-indexer.service
# Remove --kaspa-api-url flag (indexer falls back to MockVerifier)
sudo systemctl daemon-reload && sudo systemctl restart daglock-indexer
```
