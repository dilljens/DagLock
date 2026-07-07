# Plan: Price Alerts + Saved Searches (P3)

**Goal:** Let users set price targets and search filters that trigger notifications. "Notify me when NACHO reaches 0.05 KAS" or "Alert me when someone sells >1000 KAS." Keeps users coming back to DagLock instead of checking manually.

**Effort:** 2-3 days

**Why this matters:** The offer board is useful but only if you check it. Price alerts and saved searches turn it into a proactive tool — users get notified when their conditions are met without refreshing the page or polling the bot.

---

## Phase A: Price alerts `[ ]`
**⏱ Timebox:** 1 day

- [ ] New DB table `price_alerts`:
  ```sql
  CREATE TABLE price_alerts (
      id TEXT PRIMARY KEY,
      address TEXT NOT NULL,              -- who created the alert
      ticker TEXT NOT NULL,               -- "KAS" or "KRC20:NACHO"
      direction TEXT NOT NULL,            -- "above" | "below"
      target_price_sompi INTEGER NOT NULL,
      source TEXT NOT NULL DEFAULT 'coingecko',  -- price source
      is_active INTEGER NOT NULL DEFAULT 1,
      last_triggered_at INTEGER,
      created_at INTEGER NOT NULL
  );
  ```
- [ ] Price poller: background task in the indexer that checks prices every 60 seconds
  - Uses existing `fetch_kas_usd_price()` for KAS/USD
  - For KRC-20 tokens, aggregates from active offers (best bid/ask)
- [ ] When target price is met:
  - Mark alert as triggered (set `last_triggered_at`, optionally deactivate)
  - Push notification via WebSocket to connected clients
  - Dispatch via webhook for bot/dashboard
- [ ] API endpoints:
  - `POST /v1/alerts/price` — create price alert (with auth)
  - `GET /v1/alerts/price` — list user's price alerts
  - `DELETE /v1/alerts/price/:id` — delete alert
  - `PATCH /v1/alerts/price/:id` — toggle active/inactive

**✅ Checkpoint:** `POST /v1/alerts/price -d '{"ticker":"KAS","direction":"above","target_price_sompi":...}'` → alert triggers when price crosses target.

---

## Phase B: Saved searches `[ ]`
**⏱ Timebox:** 1 day

- [ ] New DB table `saved_searches`:
  ```sql
  CREATE TABLE saved_searches (
      id TEXT PRIMARY KEY,
      address TEXT NOT NULL,
      name TEXT,                           -- user-facing label ("NACHO under 0.05")
      filters TEXT NOT NULL,               -- JSON blob: {"side":"sell","base_asset":"KAS","quote_asset":"KRC20:NACHO","max_amount":...}
      notify_on_match INTEGER NOT NULL DEFAULT 1,
      last_match_at INTEGER,
      created_at INTEGER NOT NULL
  );
  ```
- [ ] Background matcher: when a new offer is created, check saved searches and notify matches
- [ ] API endpoints:
  - `POST /v1/searches` — save a search
  - `GET /v1/searches` — list saved searches
  - `DELETE /v1/searches/:id` — delete
  - `GET /v1/searches/:id/matches` — recent matches
- [ ] Offer creation event hooks into search matcher (WebSocket `offer.created` listener)

**✅ Checkpoint:** User saves a search for "NACHO under 0.05 KAS" → someone creates matching offer → user gets notified.

---

## Phase C: Bot commands + Web UI `[ ]`
**⏱ Timebox:** 1 day

- [ ] Bot commands:
  - `/alert <ticker> <above|below> <amount>` — "`/alert NACHO above 0.05`"
  - `/alerts` — list your alerts with toggle buttons
  - `/search save <name>` — save current offer board filters as a search
  - `/searches` — list + delete saved searches
- [ ] Web UI:
  - Alert creation on token detail page: "Set price alert" button → inline form
  - Saved search on offer board: "Save this filter" button → name input
  - Alert management page at `/alerts` or inline in user settings
- [ ] Notification delivery:
  - Bot: direct message when alert triggers
  - Web: toast notification + badge on bell icon
  - Email: if email notifications are configured

**✅ Checkpoint:** Bot sends "🔔 NACHO hit 0.05 KAS!" message when price target is reached.

---

## Phase D: Tests `[ ]`
**⏱ Timebox:** 4h

- [ ] Alert lifecycle: create → trigger → deactivate → re-activate
- [ ] Saved search: create → offer matches → notification fired
- [ ] Edge: duplicate alert (same user + same ticker + same direction) → 409
- [ ] Edge: alert with already-met price → immediately triggers
- [ ] Bot tests for `/alert` and `/search` commands

**✅ Checkpoint:** All tests pass.

---

## Files Changed / Created

| File | Change |
|------|--------|
| `indexer/src/db/migrations/023_price_alerts.sql` | **New** |
| `indexer/src/db/migrations/024_saved_searches.sql` | **New** |
| `indexer/src/db/queries/alerts.rs` | **New** |
| `indexer/src/db/queries/searches.rs` | **New** |
| `indexer/src/api/alerts.rs` | **New** |
| `indexer/src/api/searches.rs` | **New** |
| `indexer/src/api/mod.rs` | Register routes |
| `indexer/src/db/schema.rs` | Add migrations |
| `indexer/src/listener.rs` | Add price poller + search matcher |
| `bot/src/index.js` | Add `/alert`, `/alerts`, `/search`, `/searches` commands |
| `bot/src/lib/api.js` | Add alert/search API methods |
| `web/src/pages/TokensPage.tsx` | Add "Set alert" button |
| `web/src/pages/AlertsPage.tsx` | **New** — manage alerts |

## Edge Cases

| Case | Handling |
|------|----------|
| CoinGecko rate limited | Use cached price (15-min TTL). Show "Stale" badge |
| Token has no price yet (zero trades) | Cannot set price alert for untraded tokens |
| Too many alerts per user | Cap at 20 active alerts per address |
| Alert triggers while user is offline | Next time user opens bot/web, show pending notifications |
| Saved search matches 50 offers in 1 min | Batch notifications: "5 new matches for 'NACHO under 0.05'" |
