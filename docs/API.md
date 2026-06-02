# DagLock Indexer REST API

> Reference for the HTTP API exposed by the DagLock indexer service.

Base URL: `https://api.daglock.io/v1` (production) / `http://localhost:8443/v1` (development)

---

## 1. Escrow Endpoints

### 1.1 Create an Escrow Record

Notifies the indexer that a lock transaction has been broadcast. The indexer will verify it on-chain.

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
  "amount": "500000000000",           // sompi
  "expiration_daa_score": 12345678,
  "treasury_address": "kaspa:tr3a..."
}
```

**Response (201):**
```json
{
  "id": "esc_abc123",
  "status": "pending_confirmation",
  "template_hash": "d1e2f3...",
  "lock_tx_id": "ab12cd34...",
  "lock_tx_output_index": 0,
  "buyer_address": "kaspa:qz2q...",
  "seller_address": "kaspa:qz9x...",
  "amount_sompi": "500000000000",
  "amount_kas": "5000",
  "fee_sompi": "2500000000",
  "fee_kas": "25",
  "expiration_daa_score": 12345678,
  "treasury_address": "kaspa:tr3a...",
  "created_at": "2026-06-01T12:00:00Z",
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
  "status": "active",          // pending_confirmation | active | settled | refunded | expired
  "template_hash": "d1e2f3...",
  "lock_tx_id": "ab12cd34...",
  "lock_tx_output_index": 0,
  "buyer_address": "kaspa:qz2q...",
  "seller_address": "kaspa:qz9x...",
  "amount_sompi": "500000000000",
  "amount_kas": "5000",
  "fee_sompi": "2500000000",
  "expiration_daa_score": 12345678,
  "current_daa_score": 12500100,
  "created_at": "2026-06-01T12:00:00Z",
  "settled_at": null,
  "refunded_at": null,
  "claim_link": "https://daglock.io/claim/esc_abc123"
}
```

### 1.3 List Escrows by Address

```
GET /v1/escrows?address=kaspa:qz2q...&role=buyer&status=active&limit=20&offset=0
```

**Query parameters:**
| Param | Type | Default | Description |
|---|---|---|---|
| `address` | string | required | Kaspa address |
| `role` | string | `all` | Filter: `buyer`, `seller`, `all` |
| `status` | string | `all` | Filter: `active`, `settled`, `refunded`, `expired`, `all` |
| `limit` | int | 20 | Max results |
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

### 1.4 Get Escrow Statistics

```
GET /v1/stats
```

**Response (200):**
```json
{
  "total_escrows": 1250,
  "active_escrows": 340,
  "settled_escrows": 890,
  "refunded_escrows": 20,
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
  "chain": "mainnet",
  "node_synced": true,
  "node_daa_score": 12500100,
  "indexer_block_height": 12500000,
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
  "daglock_template_hash": "d1e2f3..."
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

## 4. Error Format

All errors return:

```json
{
  "error": {
    "code": "escrow_not_found",
    "message": "No escrow found with id 'esc_invalid'",
    "details": {}
  }
}
```

| HTTP Code | Error Code | Meaning |
|---|---|---|
| 400 | `invalid_address` | Address failed Kaspa checksum validation |
| 400 | `invalid_amount` | Amount below minimum or malformed |
| 404 | `escrow_not_found` | Escrow ID does not exist |
| 409 | `escrow_already_settled` | Attempted to claim a settled escrow |
| 409 | `escrow_expired` | Timeout refund already available |
| 429 | `rate_limited` | Too many requests |
| 500 | `internal_error` | Indexer or DB failure |

---

## 5. WebSocket Events (Future)

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
