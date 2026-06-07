# DagLock Indexer REST API

> Reference for the HTTP API exposed by the DagLock indexer service.

Base URL: `https://api.daglock.io/v1` (production) / `http://localhost:8543/v1` (development)

---

## Authentication

Lifecycle endpoints (settle, refund, dispute, cancel) require authentication headers:

| Header | Required | Description |
|--------|----------|-------------|
| `X-Daglock-Address` | Yes | Signer's Kaspa address |
| `X-Daglock-Signature` | Yes | Hex-encoded ECDSA signature |
| `X-Daglock-Message` | Yes | Signed message (format: `{action}:{escrow_id}`) |

**Message formats:**
- Settle: `settle:esc_abc123`
- Refund: `refund:esc_abc123`

**Example:**
```bash
curl -X POST http://localhost:8543/v1/escrows/esc_abc123/settle \
  -H "X-Daglock-Address: kaspa:qz2q..." \
  -H "X-Daglock-Signature: 3a4b5c..." \
  -H "X-Daglock-Message: settle:esc_abc123"
```

---

## 1. Escrow Endpoints

### 1.1 Create an Escrow Record

```
POST /v1/escrows
```

**Request body:**
```json
{
  "lock_tx_id": "ab12cd34...",
  "lock_tx_output_index": 0,
  "buyer_address": "kaspa:qz2q...",
  "seller_address": "kaspa:qz9x...",
  "amount_sompi": 500000000000,
  "expiration_daa_score": 12345678,
  "asset_type": "KAS",
  "template_hash": [1, 2, 3]
}
```

**Validation:**
- `amount_sompi` must be positive
- `buyer_address` must be valid Kaspa format (starts with `kaspa:`)
- `seller_address` must be valid Kaspa format if provided

**Response (201):**
```json
{
  "id": "esc_abc123",
  "status": "pending_confirmation",
  "template_hash": [1, 2, 3],
  "lock_tx_id": "ab12cd34...",
  "lock_tx_output_index": 0,
  "buyer_address": "kaspa:qz2q...",
  "seller_address": "kaspa:qz9x...",
  "amount_sompi": 500000000000,
  "fee_sompi": 2500000000,
  "asset_type": "KAS",
  "created_at": 1717200000,
  "settled_at": null,
  "refunded_at": null
}
```

### 1.2 Get Escrow by ID

```
GET /v1/escrows/:id
```

**Response (200):**
```json
{
  "id": "esc_abc123",
  "status": "active",
  "lock_tx_id": "ab12cd34...",
  "lock_tx_output_index": 0,
  "buyer_address": "kaspa:qz2q...",
  "seller_address": "kaspa:qz9x...",
  "amount_sompi": 500000000000,
  "fee_sompi": 2500000000,
  "asset_type": "KAS",
  "created_at": 1717200000,
  "settled_at": null,
  "refunded_at": null
}
```

### 1.3 List Escrows by Address

```
GET /v1/escrows?address=kaspa:qz2q...&role=buyer&status=active&limit=20&offset=0
```

**Query parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `address` | string | required | Kaspa address |
| `role` | string | `all` | Filter: `buyer`, `seller`, `all` |
| `status` | string | `all` | Filter: `active`, `settled`, `refunded`, `expired`, `all` |
| `limit` | int | 20 | Max results (max 100) |
| `offset` | int | 0 | Pagination offset |

**Response (200):**
```json
{
  "escrows": [ ... ],
  "total": 42,
  "limit": 20,
  "offset": 0
}
```

### 1.4 Settle Escrow

```
POST /v1/escrows/:id/settle
```

**Requires authentication headers.**

**Atomic operation:** Updates status and settled_at in a single query. Only succeeds if escrow is in `active` state.

**Response (200):**
```json
{
  "status": "settled",
  "escrow_id": "esc_abc123"
}
```

**Errors:**
- `409 escrow_already_finalized` — Escrow was already settled or is no longer active
- `403 forbidden` — Not authorized (not buyer/seller)
- `401 unauthorized` — Missing or invalid auth headers

### 1.5 Refund Escrow

```
POST /v1/escrows/:id/refund
```

**Requires authentication headers.** Only buyer can refund.

**Atomic operation:** Updates status and refunded_at in a single query.

**Response (200):**
```json
{
  "status": "refunded",
  "escrow_id": "esc_abc123"
}
```

### 1.6 Dispute Escrow

```
POST /v1/escrows/:id/dispute
```

**Request body:**
```json
{
  "reason": "Seller did not deliver"
}
```

**Response (200):**
```json
{
  "status": "disputed",
  "escrow_id": "esc_abc123"
}
```

### 1.7 Cancel Escrow

```
POST /v1/escrows/:id/cancel
```

**Response (200):**
```json
{
  "status": "cancelled",
  "escrow_id": "esc_abc123"
}
```

### 1.8 Get Statistics

```
GET /v1/stats
```

**Response (200):**
```json
{
  "total_escrows": 1250,
  "active_escrows": 340,
  "disputed_escrows": 10,
  "settled_escrows": 890,
  "refunded_escrows": 20,
  "cancelled_escrows": 5,
  "total_volume_kas": "12500000",
  "total_fees_collected_kas": "62500",
  "unique_buyers": 450,
  "unique_sellers": 380
}
```

---

## 2. Health & Status

### 2.1 Health Check

```
GET /v1/health
```

**Response (200):**
```json
{
  "status": "ok",
  "version": "0.1.0",
  "node_synced": true,
  "node_daa_score": 0,
  "uptime_seconds": 86400
}
```

### 2.2 Network Info

```
GET /v1/network
```

**Response (200):**
```json
{
  "network": "mainnet",
  "daa_score": 12500100,
  "block_count": 12500000,
  "difficulty": 123456789.0,
  "bps": 10.0,
  "daglock_kas_template_hash": "d1e2f3...",
  "daglock_krc20_template_hash": "a1b2c3..."
}
```

---

## 3. Fee Endpoints

### 3.1 Fee Estimate

```
GET /v1/fees/estimate?amount_kas=5000
```

**Response (200):**
```json
{
  "amount_kas": "5000",
  "fee_kas": "25",
  "fee_percentage": 0.5,
  "network_fee_estimate": "0.00001",
  "miner_fee_budget": "0.00001"
}
```

---

## 4. Offer Endpoints

### 4.1 List Offers

```
GET /v1/offers?asset=KAS&side=buy&status=proposed
```

**Query parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `asset` | string | `all` | Filter by asset |
| `side` | string | `all` | Filter: `buy`, `sell`, `all` |
| `status` | string | `proposed` | Filter: `proposed`, `accepted`, `locked`, `settled`, `cancelled` |

### 4.2 Create Offer

```
POST /v1/offers
```

**Request body:**
```json
{
  "creator_address": "kaspa:qz2q...",
  "side": "buy",
  "base_asset": "KAS",
  "quote_asset": "KRC20:NACHO",
  "amount_sompi": 500000000000
}
```

### 4.3 Accept Offer

```
POST /v1/offers/:id/accept
```

**Request body:**
```json
{
  "counterparty_address": "kaspa:qz9x..."
}
```

### 4.4 Cancel Offer

```
POST /v1/offers/:id/cancel
```

---

## 5. Reputation Endpoint

### 5.1 Get Reputation

```
GET /v1/reputation/:address
```

**Response (200):**
```json
{
  "address": "kaspa:qz2q...",
  "trade_count": 15,
  "total_volume_sompi": 5000000000000,
  "settled_count": 14,
  "refunded_count": 1,
  "disputed_count": 0,
  "first_trade_at": 1717200000,
  "age_days": 30,
  "dispute_rate": 0.0,
  "refund_rate": 0.067,
  "score": 3.45
}
```

---

## 6. Receipt Endpoint

### 6.1 Get Receipt

```
GET /v1/receipts/:id
```

**Response (200):**
```json
{
  "receipt_id": "rct_abc123",
  "escrow_id": "esc_abc123",
  "status": "settled",
  "asset": "KAS",
  "amount_sompi": 500000000000,
  "fee_sompi": 2500000000,
  "buyer_address": "kaspa:qz2q...",
  "seller_address": "kaspa:qz9x...",
  "lock_tx_id": "ab12cd34...",
  "lock_tx_output_index": 0,
  "settled_at": 1717203600,
  "verification": {
    "covenant_verified": true,
    "signatures_verified": true,
    "fee_compliant": true
  }
}
```

---

## 7. Error Format

All errors return:

```json
{
  "error": {
    "code": "escrow_not_found",
    "message": "No escrow found with id 'esc_invalid'"
  }
}
```

| HTTP Code | Error Code | Meaning |
|-----------|------------|---------|
| 400 | `invalid_address` | Invalid Kaspa address format |
| 400 | `invalid_amount` | Amount must be positive |
| 401 | `unauthorized` | Missing or invalid auth headers |
| 403 | `forbidden` | Not authorized for this action |
| 404 | `escrow_not_found` | Escrow ID does not exist |
| 409 | `escrow_already_finalized` | Escrow already settled/refunded/cancelled |
| 409 | `verification_failed` | UTXO verification failed |
| 500 | `internal_error` | Indexer or DB failure |

---

## 8. CORS

The API supports CORS for browser access. All origins are allowed by default.

---

## 9. Rate Limiting

Rate limiting is handled at the reverse proxy level (nginx/caddy). Default: 100 requests/second per IP.

---

## 10. WebSocket Events (Future)

The indexer will optionally expose a WebSocket endpoint for real-time updates:

```
WS /v1/ws?address=kaspa:qz2q...
```

**Event types:**
```json
{"event": "escrow_created", "data": { ... }}
{"event": "escrow_settled", "data": { ... }}
{"event": "escrow_refunded", "data": { ... }}
{"event": "escrow_expired", "data": { ... }}
```
