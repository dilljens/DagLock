# Plan #1: KRC-20 Token Dashboard

**Goal:** Build a DexScreener-like token dashboard for KRC-20 tokens on Kaspa. Show price, volume, holders, and trades for every KRC-20 token. Link every token page to the escrow creation flow so users can one-click "Buy NACHO" via escrow.

**Effort:** ~1 week

**Why this matters:** When Toccata activates June 30, KRC-20 tokens go live. There is ZERO token infrastructure in the Kaspa ecosystem — no explorer, no charts, no price feeds. Whichever product ships first becomes the default KRC-20 portal. The indexer already sees all escrow and offer activity for KRC-20 tokens — no raw blockchain scanning needed.

---

## Architecture

```
Indexer (already sees KRC-20 trades via escrows)
  │
  ├── /v1/tokens                  → list all tokens with price/volume/change
  ├── /v1/tokens/:ticker          → token detail (holders, supply, trade history)
  ├── /v1/tokens/:ticker/chart    → price history (7d/30d/all)
  └── /v1/tokens/:ticker/trades   → recent escrow trades for this token
       │
       ▼
Web (new /tokens route)
  ├── /tokens                     → token directory (sortable table)
  └── /tokens/:ticker             → token detail (chart + trades + buy button)
```

---

## Phase 1A: Indexer data pipeline `[ ]`
**⏱ Timebox:** 2 days

### Token aggregation queries
- [ ] Create `indexer/src/db/queries/tokens.rs`:
  - `list_tokens(pool)` — aggregate from offers + escrows: ticker, current price (from latest offer), 24h volume, trade count, total escrows locked
  - `get_token(pool, ticker)` — detail with trade history
  - `get_token_price_history(pool, ticker, period)` — price points from settlement history
  - `get_token_trades(pool, ticker, limit)` — recent escrows involving this token

### API endpoints
- [ ] Create `indexer/src/api/tokens.rs`:
  - `GET /v1/tokens` — list all traded KRC-20 tokens with summary stats
  - `GET /v1/tokens/:ticker` — token detail page data
  - `GET /v1/tokens/:ticker/chart?period=7d|30d|all` — price chart data
  - `GET /v1/tokens/:ticker/trades?limit=20` — recent escrow trades

### Register
- [ ] Add to `indexer/src/api/mod.rs`:
  ```rust
  pub mod tokens;
  // routes:
  .route("/v1/tokens", get(tokens::list))
  .route("/v1/tokens/:ticker", get(tokens::get))
  .route("/v1/tokens/:ticker/chart", get(tokens::chart))
  .route("/v1/tokens/:ticker/trades", get(tokens::trades))
  ```
- [ ] Add `pub mod tokens` to `indexer/src/db/queries/mod.rs`

**✅ Checkpoint:** `curl /v1/tokens` returns `["NACHO","GHOST","KASPY"]` with price/volume stats

---

## Phase 1B: Token directory page `[ ]`
**⏱ Timebox:** 1 day

- [ ] Create `web/src/pages/TokensPage.tsx`:
  - Sortable table with columns: Token, Price, 24h Change, Volume, Trades, Market Cap
  - Search/filter bar (search by ticker or name)
  - Click row → navigates to `/tokens/:ticker`
- [ ] Add /tokens route to `web/src/router.tsx`:
  ```tsx
  | "/tokens"
  ```
- [ ] Add to `web/src/App.tsx` router:
  ```tsx
  const TokensPage = lazy(() => import("./pages/TokensPage").then(m => ({ default: m.TokensPage })));
  // ...
  case "/tokens": return <TokensPage />;
  ```
- [ ] Add to sidebar `NAV_ITEMS` in `Sidebar.tsx`

**✅ Checkpoint:** Navigate to `/tokens` → see sortable table of all KRC-20 tokens

---

## Phase 1C: Token detail page `[ ]`
**⏱ Timebox:** 2 days

- [ ] Create `web/src/pages/TokenDetailPage.tsx`:
  - **Header:** Token ticker, name, current price in KAS + USD (USD from CoinGecko)
  - **Price chart:** Simple inline SVG polyline (no heavy chart library)
    - 7d / 30d / All toggle
    - Shows price_at_creation markers for each trade
  - **Stats cards:** Volume (24h), Trades (24h), Floor Price, Unique Sellers
  - **Recent trades table:** List of recent escrows involving this token
    - Columns: Time, Side (buy/sell), Amount, Price, Status, Explorer Link
  - **"Buy [TOKEN]" button:** One-click → navigates to escrow creation with token pre-selected
- [ ] Add route: `/tokens/:ticker`

**✅ Checkpoint:** Navigate to `/tokens/NACHO` → see price chart, stats, trade history

---

## Phase 1D: Escrow integration `[ ]`
**⏱ Timebox:** 1 day

- [ ] "Buy" button on token detail → pre-fills escrow creation form:
  - Asset type = `KRC20`
  - Token ticker auto-selected
  - Amount field focused
- [ ] Create offer from token page: "Sell [TOKEN]" → pre-fills offer creation
- [ ] Link existing escrow trades to token detail (each trade row links back)
- [ ] Show token badge on escrow cards (e.g., "🏷️ NACHO" badge on KRC-20 escrows)

**✅ Checkpoint:** Click "Buy NACHO" → escrow creation form with NACHO pre-selected

---

## Phase 1E: Production polish `[ ]`
**⏱ Timebox:** 1 day

- [ ] Handle empty state: "No KRC-20 tokens traded yet. Be the first!"
- [ ] Handle low-data state: only 1-2 trades → show "Just listed" instead of change %
- [ ] Price precision: KRC-20 tokens may trade at micro amounts (<0.001 KAS)
- [ ] Caching: token list uses react-query with 30s staleTime (already configured)
- [ ] Responsive: token table collapses to card layout on mobile
- [ ] Tests:
  - `web/src/__tests__/TokensPage.test.tsx` — renders token list
  - `web/src/__tests__/TokenDetailPage.test.tsx` — renders detail with mock trade data

**✅ Checkpoint:** Lighthouse score ≥ 80, mobile layout works, all tests pass

---

## Edge Cases

| Case | Handling |
|------|----------|
| No tokens traded yet | Show empty state with CTA to create first offer |
| Token with 1 trade | "Just listed" badge, no 24h change |
| Token ticker not found | 404 page with "Token not found — create it?" CTA |
| Price = 0 (no active offers) | Show "No active market" instead of price |
| Very high precision prices | Show up to 8 decimal places, with tooltip for full value |
| Indexer data stale | Token data is as fresh as the last escrow settlement — cached 30s |
