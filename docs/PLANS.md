# Detailed Implementation Plans

---

## 1. Atomic Swap Wizard UI

**Goal:** Guide users through the atomic swap flow step-by-step — generate secret, share hash, settle with preimage. No manual hex entry.

### Current State

- `POST /v1/swap/generate` returns `{secret, hash}` — works
- `POST /v1/escrows/:id/swap` accepts `{preimage}` — works
- Web UI has a `SwapForm` component but it's a flat text input field, not a guided flow
- Users must manually: generate hash, share it, create escrow with trade_hash, receive preimage, paste it — all with raw hex strings

### Flow Design

```
Step 1 — Initiate
  User A clicks "New Atomic Swap" on escrow create form
  → Amount, asset, counterparty fields
  → "Generate Secret" button → shows secret (warn: save this!)
  → Hash auto-filled into trade_hash field
  → Creates escrow (funds locked with trade_hash in covenant)

Step 2 — Share
  User A shares the escrow ID with User B via trade link
  → https://t.me/DagLock_bot?start=claim_<escrow_id>
  → OR sends escrow ID + hash manually

Step 3 — Claim
  User B opens the escrow, sees "Atomic Swap" badge
  → Enters their preimage (the secret shared out-of-band)
  → Submits → covenant verifies sha256(preimage) == tradeHash
  → Funds released to User B

Step 4 — Complete
  User B sees success screen with receipt link
  User A can monitor status via dashboard
```

### Web UI Changes

**Files affected:**
| File | Change |
|------|--------|
| `web/src/pages/EscrowsPage.tsx` | Add "Atomic Swap" tab + guided wizard |
| `web/src/components/escrows.tsx` | Refactor `SwapForm` → `AtomicSwapWizard` |
| `web/src/api.ts` | No changes needed (endpoints exist) |
| `web/src/__tests__/SwapForm.test.tsx` | Update tests for new wizard |

**New component: `AtomicSwapWizard`**

```typescript
// State machine for swap flow:
type SwapStep = "init" | "secrets" | "create" | "wait" | "claim" | "done";

function AtomicSwapWizard() {
  // 1. INIT: user enters amount + counterparty
  // 2. SECRETS: user clicks "Generate" → sees secret (copy to clipboard)
  //    Auto-populates trade_hash on escrow create form
  // 3. CREATE: escrow created with trade_hash embedded in covenant
  //    Shows "Share this link with your counterparty"
  // 4. WAIT: polling escrow status, waiting for counterparty to claim
  // 5. CLAIM: (counterparty's view) enter preimage → settle
  // 6. DONE: show receipt
}
```

**Key UX details:**
- Secret displayed once with a copy button + "Save this!" warning
- Trade link auto-generated: `https://t.me/DagLock_bot?start=swap_<escrow_id>`
- Countdown timer showing time remaining before refund
- Polling every 10 seconds for status changes

### Backend Changes

| File | Change |
|------|--------|
| `indexer/src/api/escrows.rs` | Add `lock-status` endpoint (already exists — `/v1/escrows/:id/lock-status`) |
| No other backend changes | All swap endpoints exist |

### Testing

| Test | Description |
|------|-------------|
| User creates escrow with trade_hash | Verify hash appears in escrow object |
| User submits wrong preimage | 403 forbidden |
| User submits correct preimage | Escrow settles |
| Double claim attempt | 409 conflict |
| Swap expiration (timeout) | Escrow becomes refundable |

### Effort: 4-5 days

| Day | Work |
|-----|------|
| 1 | Wire `AtomicSwapWizard` state machine + secret generation |
| 2 | Build create-with-hash flow + trade link generation |
| 3 | Build claim flow + preimage submission |
| 4 | Add polling, status display, countdown timer |
| 5 | Testing + edge cases |

---

## 2. Price Oracle Improvements

**Goal:** Provide on-chain price anchoring at escrow creation/settlement time, display historical prices, and show real-time KAS/USD on the dashboard.

### Current State

- `fetch_kas_usd_price()` fetches from `api.coingecko.com/api/v3/simple/price?ids=kaspa&vs_currencies=usd`
- 5-minute in-memory TTL cache (just added)
- Price stored on escrow creation (`price_at_creation`) and at settlement (`price_at_settlement`)
- Market-price offers update every 15 minutes via listener
- No historical price tracking
- No fiat price displayed on the dashboard
- No price oracle webhook for real-time updates

### What's Missing

| Gap | Priority | User Impact |
|-----|----------|-------------|
| Price displayed on dashboard | High | Users don't know KAS value in USD |
| Escrow create shows USD equivalent | High | Users creating 5000 KAS escrow see "~$750 USD" |
| Settlement price history | Medium | Tax reporting, trade analytics |
| Price chart on escrow detail | Low | Nice-to-have visualization |
| WebSocket price updates | Low | Real-time dashboard updates |

### Phase 1: Dashboard Price Display (1-2 days)

**Files affected:**
| File | Change |
|------|--------|
| `indexer/src/api/network.rs` | Add KAS/USD price to `/v1/network` response |
| `web/src/pages/Dashboard.tsx` | Add price card showing KAS/USD |
| `web/src/api.ts` | No changes (network endpoint already returns price) |

**New dashboard card:**
```
┌──────────────────────┐
│  KAS Price (USD)     │
│                      │
│  $0.0425             │
│  ↑ 2.3% (24h)        │
│                      │
│  Updated 2m ago      │
└──────────────────────┘
```

### Phase 2: USD Equivalent on Escrow Creation (1 day)

**Files affected:**
| File | Change |
|------|--------|
| `web/src/pages/EscrowsPage.tsx` | Show "≈ $X USD" below amount input |
| `web/src/pages/OffersPage.tsx` | Show USD equivalent on offer cards |

```
Amount (KAS): [5000] → ≈ $212.50 USD (at $0.0425/KAS)
```

### Phase 3: Historical Price Storage (2-3 days)

**Files affected:**
| File | Change |
|------|--------|
| `indexer/src/db/migrations/018_price_history.sql` | **New** — price_history table |
| `indexer/src/db/schema.rs` | Add migration 018 |
| `indexer/src/db/queries.rs` | Add `record_price()`, `get_price_history()` |
| `indexer/src/listener.rs` | Record price every 15 minutes in DB |
| `indexer/src/api/network.rs` | Add `/v1/network/price/history` endpoint |

**Migration SQL:**
```sql
CREATE TABLE IF NOT EXISTS price_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    price_usd REAL NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_price_history_created
    ON price_history(created_at);
```

### Phase 4: Price Chart on Escrow Detail (2-3 days)

**Files affected:**
| File | Change |
|------|--------|
| `web/src/pages/EscrowsPage.tsx` | Add price chart to escrow detail view |
| `web/src/components/PriceChart.tsx` | **New** — simple SVG price chart |

Simple inline SVG chart (no heavy charting library — just raw SVG paths):

```typescript
function PriceChart({ history }: { history: { time: number; price: number }[] }) {
  // Draw a simple polyline showing price over time
  // Show price_at_creation marker, price_at_settlement marker
  // Height: 80px, responsive width
}
```

### Total Effort: 5-8 days (can be done incrementally)

| Phase | Effort | Ships |
|-------|--------|-------|
| 1. Dashboard price | 1-2 days | Part of main UI |
| 2. USD equivalent | 1 day | Part of escrow/offer forms |
| 3. Historical storage | 2-3 days | Backend |
| 4. Price chart | 2-3 days | UI polish |

### Edge Cases

- CoinGecko rate-limited: cache serves stale price, show "Stale" badge
- No internet: show "Offline" with last known price
- Price = 0 or negative: hide USD estimates, show "Price unavailable"
- Decimal precision: show 4 decimal places for sub-cent KAS prices
