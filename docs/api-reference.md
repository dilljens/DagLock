# DagLock API Reference

**Base URL:** `https://daglock-production.up.railway.app`

**Version:** 1.0.0

---

## Authentication

Most endpoints don't require authentication. For authenticated operations (settle, refund, cancel), provide these headers:

```
X-Daglock-Address: <your kaspa address>
X-Daglock-Signature: <hex signature>
X-Daglock-Message: <signed message>
```

---

## Endpoints

### System

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/health` | Health check |
| GET | `/v1/network` | Network information |
| GET | `/v1/network/price` | KAS/USD price |
| GET | `/v1/stats` | Platform statistics |

### Escrows

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/escrows` | List escrows |
| POST | `/v1/escrows` | Create escrow |
| GET | `/v1/escrows/:id` | Get escrow by ID |
| POST | `/v1/escrows/:id/settle` | Settle escrow |
| POST | `/v1/escrows/:id/refund` | Refund escrow |
| POST | `/v1/escrows/:id/dispute` | Dispute escrow |
| POST | `/v1/escrows/:id/cancel` | Cancel escrow |

### Offers

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/offers` | List offers |
| POST | `/v1/offers` | Create offer |
| POST | `/v1/offers/:id/accept` | Accept offer |
| POST | `/v1/offers/:id/cancel` | Cancel offer |

### Reputation

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/reputation/:address` | Get reputation |

### Receipts

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/receipts/:id` | Get receipt |

### Vaults

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/vaults` | List vaults |
| POST | `/v1/vaults` | Create vault |
| GET | `/v1/vaults/:id` | Get vault by ID |
| POST | `/v1/vaults/:id/withdraw` | Withdraw from vault |

---

## Examples

### Health Check

```bash
curl https://daglock-production.up.railway.app/v1/health
```

Response:
```json
{
  "status": "ok",
  "version": "0.1.0",
  "node_synced": false,
  "node_daa_score": 0,
  "uptime_seconds": 12345
}
```

### Create Escrow

```bash
curl -X POST https://daglock-production.up.railway.app/v1/escrows \
  -H "Content-Type: application/json" \
  -d '{
    "lock_tx_id": "abc123",
    "lock_tx_output_index": 0,
    "buyer_address": "kaspa:qdyzkrhd74v6cetrv4fhv",
    "seller_address": "kaspa:qg3h9mhu78cw89qyc0e42",
    "amount_sompi": 100000000,
    "asset_type": "KAS"
  }'
```

Response:
```json
{
  "id": "esc_abc123",
  "status": "pending_confirmation",
  "amount_sompi": 100000000,
  "fee_sompi": 500000,
  "created_at": 1700000000
}
```

### Create Offer

```bash
curl -X POST https://daglock-production.up.railway.app/v1/offers \
  -H "Content-Type: application/json" \
  -d '{
    "creator_address": "kaspa:qdyzkrhd74v6cetrv4fhv",
    "side": "sell",
    "base_asset": "KAS",
    "quote_asset": "KRC20:NACHO",
    "amount_sompi": 50000000
  }'
```

Response:
```json
{
  "id": "off_xyz789",
  "status": "proposed",
  "amount_sompi": 50000000,
  "created_at": 1700000000
}
```

### Get Reputation

```bash
curl https://daglock-production.up.railway.app/v1/reputation/kaspa:qdyzkrhd74v6cetrv4fhv
```

Response:
```json
{
  "address": "kaspa:qdyzkrhd74v6cetrv4fhv",
  "score": 3.5,
  "trade_count": 5,
  "settled_count": 4,
  "disputed_count": 0,
  "total_volume_sompi": 500000000
}
```

---

## Error Responses

All errors return:

```json
{
  "error": "error_code",
  "message": "Human-readable error message"
}
```

Common error codes:
- `not_found` — Resource not found
- `invalid_address` — Invalid Kaspa address
- `insufficient_funds` — Not enough funds
- `timeout_not_reached` — Vault timeout not reached
- `forbidden` — Not authorized

---

## Rate Limits

No rate limits currently. Please be respectful of the API.

---

## Changelog

### v1.0.0 (2026-06-05)
- Initial API release
- Escrow CRUD operations
- Offer management
- Reputation system
- Vault support
