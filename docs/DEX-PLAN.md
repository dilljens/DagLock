# DagLock DEX — Market Orders + Limit Orders

## Overview

Build a full DEX experience with two order types:
1. **Market orders** — Price at settlement (simple, familiar)
2. **Limit orders** — Fixed price with atomic swap (price protection)

---

## Market Orders (Launch Priority)

### User Flow
```
1. Alice: "Sell 1000 KAS for NACHO at market price"
2. System: Fetches KAS/NACHO price, shows locked price
3. Alice: Confirms → creates escrow with locked price
4. Bob: Sees offer, clicks "Accept"
5. Settlement: Happens at the locked price
```

### Key Properties
- Price fetched from API at creation time
- Price is locked once escrow is created
- No preimage/secret mechanism needed
- Settlement is simple (both parties sign)

### Backend Changes
- Add `price_lock_time` to escrow (when price was fetched)
- Add `price_at_settlement` field (actual price used)
- Modify settlement to use locked price
- Add price fetch endpoint for tokens

### Frontend Changes
- CreateEscrowForm: "Market price" option → fetches price, shows locked price
- SwapForm: Simple "Accept" button (no preimage needed)
- OfferCard: Show locked price + time since lock

---

## Limit Orders (Atomic Swaps)

### User Flow
```
1. Alice: "Sell 1000 KAS for NACHO at $0.15 fixed"
2. System: Creates escrow with trade_hash
3. Alice: Shares secret with Bob off-chain
4. Bob: Submits secret → settlement
```

### Key Properties
- Price fixed at creation
- Requires preimage/secret for settlement
- Both parties know exact terms
- No price risk

### Backend Changes
- Existing atomic swap endpoint (already implemented)
- Trade hash generation endpoint (already implemented)

### Frontend Changes
- CreateEscrowForm: "Fixed price" option → manual price input
- SwapForm: Submit preimage field
- OfferCard: Show fixed price

---

## Implementation Plan

### Phase 1: Market Orders (2-3 days)

| Step | Task | Files |
|------|------|-------|
| 1 | Add price lock fields to escrow | `types.rs`, `schema.rs`, `queries.rs` |
| 2 | Fetch price at escrow creation | `escrows.rs` |
| 3 | Lock price in escrow | `escrows.rs` |
| 4 | CreateEscrowForm: market price option | `App.tsx` |
| 5 | OfferCard: show locked price | `App.tsx` |
| 6 | Simple settlement (no preimage) | `escrows.rs` |
| 7 | Testing + deploy | — |

### Phase 2: Limit Orders (1-2 days)

| Step | Task | Files |
|------|------|-------|
| 1 | Existing atomic swap endpoint | `escrows.rs`  |
| 2 | Trade hash generation | `swap.rs`  |
| 3 | CreateEscrowForm: fixed price option | `App.tsx` |
| 4 | SwapForm: submit preimage | `App.tsx`  |
| 5 | Testing + deploy | — |

---

## Database Schema

### New fields for market orders
```sql
ALTER TABLE escrows ADD COLUMN price_lock_time INTEGER;
ALTER TABLE escrows ADD COLUMN price_at_settlement REAL;
ALTER TABLE escrows ADD COLUMN price_source TEXT;  -- 'coingecko', 'manual'
```

### Existing fields (limit orders)
```sql
-- trade_hash already exists for atomic swaps
ALTER TABLE escrows ADD COLUMN trade_hash TEXT;
```

---

## API Endpoints

### Market Orders
```
POST /v1/escrows
  - price_type: "market" | "fixed"
  - For market: auto-fetches price, locks it
  - For fixed: uses provided price

POST /v1/escrows/:id/settle
  - Uses locked price for settlement
```

### Limit Orders (existing)
```
POST /v1/swap/generate
  - Generates secret + hash

POST /v1/escrows/:id/swap
  - Submits preimage to settle
```

---

## Frontend Components

### CreateEscrowForm
```
[Market price] ← toggle
  → Shows current price
  → Locks price on create

[Fixed price] ← toggle
  → Manual price input
  → Requires trade hash (generates secret)
```

### OfferCard
```
Market: "1000 KAS for NACHO @ $0.15 (locked 2m ago)"
Fixed:  "1000 KAS for NACHO @ $0.15 fixed"
```

### SwapForm (Market)
```
[Accept Offer]
  → No preimage needed
  → Settlement at locked price
```

### SwapForm (Limit)
```
[Submit Preimage]
  → Paste secret
  → Settlement via atomic swap
```

---

## Security Considerations

| Risk | Market Order | Limit Order |
|------|-------------|-------------|
| Price manipulation | Price locked at creation | Price fixed at creation |
| Front-running | Price locked, no preimage | Preimage protects |
| Sandwich attack | Price locked | Price fixed |
| Oracle manipulation | Single price source | N/A |

---

## Testing Strategy

### Market Orders
1. Create market order → verify price locked
2. Wait → verify price doesn't change
3. Settle → verify locked price used
4. Test price bounds (min/max)

### Limit Orders
1. Create limit order → verify trade hash stored
2. Wrong preimage → verify rejection
3. Correct preimage → verify settlement
4. Timeout → verify refund

---

## Success Criteria

- [ ] Market orders create escrows with locked prices
- [ ] Limit orders create escrows with trade hashes
- [ ] Settlement uses correct price for each type
- [ ] UI clearly shows order type and price
- [ ] All tests pass
- [ ] Deployed to testnet
