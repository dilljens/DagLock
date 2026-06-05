# P0 Features Implementation Plan

## Goal
Make DagLock useful for real trading by adding:
1. API documentation (OpenAPI spec)
2. Price-locked offers (market price trading)
3. wRPC listener enhancement (on-chain detection)

---

## Phase 1: API Documentation (1-2 days)

### What
Create OpenAPI/Swagger specification for all API endpoints.

### Why
Developers need documentation to integrate with DagLock.

### Files
- `docs/openapi.yaml` — OpenAPI 3.0 spec
- `docs/api-reference.md` — Human-readable API docs

### Endpoints to Document
```
GET  /v1/health
GET  /v1/network
GET  /v1/network/price
GET  /v1/stats
POST /v1/compile
GET  /v1/escrows
POST /v1/escrows
GET  /v1/escrows/:id
POST /v1/escrows/:id/settle
POST /v1/escrows/:id/refund
POST /v1/escrows/:id/dispute
POST /v1/escrows/:id/cancel
GET  /v1/offers
POST /v1/offers
POST /v1/offers/:id/accept
POST /v1/offers/:id/cancel
GET  /v1/reputation/:address
GET  /v1/receipts/:id
POST /v1/vaults
GET  /v1/vaults
GET  /v1/vaults/:id
POST /v1/vaults/:id/withdraw
```

---

## Phase 2: Price-Locked Offers (2-3 days)

### What
Allow offers to be priced at market rate instead of fixed price.

### Why
Users want to trade at market price without worrying about price fluctuations.

### Database Changes
```sql
ALTER TABLE offers ADD COLUMN price_type TEXT DEFAULT 'fixed';
ALTER TABLE offers ADD COLUMN price_offset REAL DEFAULT 0.0;
ALTER TABLE offers ADD COLUMN min_price REAL;
ALTER TABLE offers ADD COLUMN max_price REAL;
```

### API Changes
```json
POST /v1/offers
{
  "creator_address": "kaspa:...",
  "side": "sell",
  "base_asset": "KAS",
  "quote_asset": "USD",
  "amount_sompi": 100000000,
  "price_type": "market",
  "price_offset": 0.0,
  "min_price": 0.10,
  "max_price": 0.20
}
```

### Logic
1. Fetch current price from CoinGecko
2. Calculate offer price: `market_price + price_offset`
3. Check bounds: `min_price <= calculated_price <= max_price`
4. Store offer with current price
5. Reconciliation loop updates prices periodically

---

## Phase 3: wRPC Listener Enhancement (2-3 days)

### What
Detect DagLock UTXOs on-chain via wRPC.

### Why
Need to verify escrows exist on-chain before settlement.

### Files
- `indexer/src/listener.rs` — Already exists, needs enhancement

### Logic
1. Connect to Kaspa node via wRPC
2. Subscribe to new blocks
3. Scan transactions for DagLock template hashes
4. Update escrow status based on on-chain state
5. Handle confirmations and reorganizations

---

## Execution Order
1. **API Documentation** — Start immediately
2. **Price-Locked Offers** — Database + API changes
3. **wRPC Enhancement** — On-chain detection

---

## Success Criteria
- [ ] OpenAPI spec covers all endpoints
- [ ] Price-locked offers work end-to-end
- [ ] wRPC listener detects DagLock UTXOs
- [ ] All tests pass
- [ ] Documentation is clear and complete
