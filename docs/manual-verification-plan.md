# DagLock Manual Verification Plan

**Purpose:** Pre-flight checklist to run before mainnet deploy. Catches issues that automated tests can't (HTTP integration, auth enforcement, rate limiting, wallet flows).

---

## Prerequisites

```bash
# Build everything
cargo build --workspace
cd web && npm run build
```

---

## 1. Testnet Deploy

### 1.1 Start Indexer (dev mode)
```bash
# Run in terminal 1
cargo run -p daglock-indexer -- \
  --mock-auth \
  --network testnet-11 \
  --database-url sqlite::memory: \
  --cors-origin "*"
```
> `--mock-auth` is OK for testnet — mainnet panics if it's set.

### 1.2 Start Web Dashboard
```bash
# Run in terminal 2
cd web && npm run dev
```

### 1.3 Health Check
```bash
curl -s http://localhost:8543/v1/health | python3 -m json.tool
```
**Expected:**
```json
{
  "status": "ok",
  "db_connected": true,
  "version": "0.1.0",
  "uptime_seconds": 3
}
```

---

## 2. Rate Limiting

Verify the 30 req/min per-IP rate limiter works.

```bash
# Send 31 requests — the 31st should get 429
for i in $(seq 1 31); do
  CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8543/v1/health)
  echo "Request $i: $CODE"
  if [ "$i" -eq 31 ] && [ "$CODE" != "429" ]; then
    echo "❌ FAIL: Expected 429 on request 31"
    exit 1
  fi
done
echo "✅ Rate limiter works — request 31 got 429"
```

**Expected:** Requests 1-30 return 200, request 31 returns 429.

---

## 3. Auth Enforcement

### 3.1 Dispute Without Auth → 401
```bash
# First insert an escrow so we have something to dispute
curl -s -X POST http://localhost:8543/v1/escrows \
  -H "Content-Type: application/json" \
  -d '{
    "lock_tx_id": "tx_dispute_test",
    "lock_tx_output_index": 0,
    "buyer_address": "kaspa:buyer",
    "amount_sompi": 100000000,
    "asset_type": "KAS"
  }'

# Try to dispute without auth headers
ESCROW_ID="esc_..."  # from the response above
curl -s -w "\nHTTP %{http_code}" -X POST "http://localhost:8543/v1/escrows/$ESCROW_ID/dispute" \
  -H "Content-Type: application/json" \
  -d '{"reason": "no auth"}'
```
**Expected:** HTTP 401 with `{"error": "unauthorized", ...}`

### 3.2 Offer Cancel Without Auth → 401
```bash
curl -s -w "\nHTTP %{http_code}" -X POST http://localhost:8543/v1/offers/some_id/cancel
```
**Expected:** HTTP 401

### 3.3 Escrow Create With Wrong Auth → 403
```bash
# Auth headers claim address A but body has address B
curl -s -w "\nHTTP %{http_code}" -X POST http://localhost:8543/v1/escrows \
  -H "Content-Type: application/json" \
  -H "X-Daglock-Address: kaspa:alice" \
  -H "X-Daglock-Signature: aa" \
  -H "X-Daglock-Message: create:esc_test" \
  -d '{
    "lock_tx_id": "tx_auth_mismatch",
    "lock_tx_output_index": 0,
    "buyer_address": "kaspa:bob",
    "amount_sompi": 100000000
  }'
```
**Expected:** HTTP 403 (signed address doesn't match buyer)

---

## 4. App Registration + API Key Flow

### 4.1 Register an App
```bash
REG=$(curl -s -X POST http://localhost:8543/v1/apps/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "ManualTest",
    "owner_address": "kaspa:testuser"
  }')
echo "$REG" | python3 -m json.tool

# Extract values
API_KEY=$(echo "$REG" | python3 -c "import sys,json; print(json.load(sys.stdin)['api_key'])")
APP_ID=$(echo "$REG" | python3 -c "import sys,json; print(json.load(sys.stdin)['app']['id'])")
```
**Expected:** HTTP 201 with `api_key` (starts with `dl_sk_`) and `app` object.

### 4.2 Access App With Valid Key
```bash
curl -s -w "\nHTTP %{http_code}" "http://localhost:8543/v1/apps/$APP_ID" \
  -H "X-Daglock-Api-Key: $API_KEY"
```
**Expected:** HTTP 200 with app details. Name is "ManualTest".

### 4.3 Access App Without Key
```bash
curl -s -w "\nHTTP %{http_code}" "http://localhost:8543/v1/apps/$APP_ID"
```
**Expected:** HTTP 401

### 4.4 Access App With Wrong Key
```bash
curl -s -w "\nHTTP %{http_code}" "http://localhost:8543/v1/apps/$APP_ID" \
  -H "X-Daglock-Api-Key: dl_sk_wrong_key_12345"
```
**Expected:** HTTP 401

### 4.5 Cross-App Access Forbidden
```bash
# Register a second app
REG2=$(curl -s -X POST http://localhost:8543/v1/apps/register \
  -H "Content-Type: application/json" \
  -d '{"name": "SecondApp", "owner_address": "kaspa:user2"}')
KEY2=$(echo "$REG2" | python3 -c "import sys,json; print(json.load(sys.stdin)['api_key'])")

# Use key2 to access app1
curl -s -w "\nHTTP %{http_code}" "http://localhost:8543/v1/apps/$APP_ID" \
  -H "X-Daglock-Api-Key: $KEY2"
```
**Expected:** HTTP 403 (key from app2 can't access app1)

---

## 5. Web End-to-End (Browser Required)

### 5.1 Prerequisites
- Chrome/Brave with KasWare extension installed
- Testnet KAS from faucet
- Indexer running with `--cors-origin *`

### 5.2 Flow
1. Open `http://localhost:5173` in browser
2. Click "Connect Wallet" → approve in KasWare
3. Navigate to **Escrows** → **Create**
4. Enter amount (e.g., `10` KAS)
5. Click "Create Escrow"
6. **Verify:** KasWare prompts to send KAS to a `kaspatest:p...` address
7. **Verify:** After broadcast, escrow appears with `pending_confirmation` status
8. Navigate to **Swap** → **Generate Swap**
9. Click "Generate Secret & Hash"
10. **Verify:** Secret + hash displayed with orange warning box

### 5.3 If KasWare Not Available
The form falls back to a manual prompt for tx_id. Enter any hex string to test the indexer path.

---

## 6. CLI Flow (Optional — Requires kaspawallet)

```bash
# Run from within the DagLock repo
cargo run -p daglock-cli -- create \
  --amount 100 \
  --counterparty kaspa:partner_address

# Expected output:
#   Covenant address: kaspatest:p...
#   Send funds to this address using:
#      kaspawallet send --to kaspatest:p... --amount 100 --priority normal
```

---

## 7. Bot Wizard (Optional)

```bash
cd bot
BOT_TOKEN=your_test_token node src/index.js
```

In Telegram:
1. `/create`
2. Enter amount `1`
3. Enter counterparty or `skip`
4. Pick timeout `1`
5. Pick dispute mode `standard`
6. **Verify:** Bot replies with web app link, NOT an escrow ID

---

## Full Deploy Checklist

| # | Check | Pass |
|---|-------|------|
| 1 | `cargo test --workspace` — all 176 pass | ☐ |
| 2 | `cd web && npm test && npm run build` — all 36 pass | ☐ |
| 3 | Health endpoint returns `db_connected: true` | ☐ |
| 4 | Rate limiter: 31st request → 429 | ☐ |
| 5 | Dispute without auth → 401 | ☐ |
| 6 | Offer cancel without auth → 401 | ☐ |
| 7 | Escrow create with mismatched auth → 403 | ☐ |
| 8 | App register → key → access → 200 | ☐ |
| 9 | App access without key → 401 | ☐ |
| 10 | App access with wrong key → 401 | ☐ |
| 11 | Cross-app key access → 403 | ☐ |
| 12 | Web: KasWare connects | ☐ |
| 13 | Web: Create escrow → covenant address → broadcast | ☐ |
| 14 | Web: Swap → generate secret/hash | ☐ |
| 15 | Telegram: /create wizard → web link | ☐ |
