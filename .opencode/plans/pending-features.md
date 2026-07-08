# Plan: Pending Features — Post-OfficeForge Gap Analysis

> **Goal:** Implement the remaining high-value features identified from the OfficeForge comparison and PENDING.md backlog. Skip volume-based fee rebates (per user request).
>
> **Status:** Plan created. All 7 tracks scoped with pre-resolved decisions.

---

## Requirements

- [ ] **R1** Self-hosted Kaspa node (real UTXO verification, anchor tx broadcasting)
- [ ] **R2** Analytics dashboard (`stats.daglock.io`)
- [ ] **R3** Trading bot API (rate limit tiers)
- [ ] **R4** Price oracle improvements (alerts, charts)
- [ ] **R5** Escrow-as-a-service widget (`<daglock-pay>`)
- [ ] **R6** Interactive "Try to break" demo (marketing)
- [ ] **R7** Deal type presets (UX improvement)
- [ ] **R8** DAA-block timing for vault contracts (security improvement)

---

## Pre-resolved Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| DAA timing | Only vault contracts, not escrows | Escrows need absolute deadlines; vaults benefit from relative maturity |
| Interactivity demo | Static HTML/CSS/JS page, no backend | Covenant rules are deterministic — no DB needed |
| Stats dashboard | React page + Prometheus-like time-series | New table for daily aggregates; Grafana option for later |
| Bot API | API key tiers on existing rate limiter | Existing infrastructure (`--rate-limit` flag, api_keys table) |
| Payment for bot API | Manual invoicing (Stripe/Paddle later) | Start simple, automate when demand exists |
| Oracle | CoinGecko-only, add price alerts | Covenant can't access external data, so oracle stays off-chain |
| Escrow widget | Vanilla JS web component (`<daglock-pay>`) | No framework dependency, works on any site |
| Deal type presets | Frontend-only change | No backend changes needed |

---

## Track A: Self-Hosted Kaspa Node `[ ]`

**Description:** Provision a dedicated VPS for `kaspad` with `--utxoindex`. Wire the indexer to use real wRPC verification instead of MockVerifier. Enable anchor tx broadcasting for the chat feature.

**Timebox:** 2-3 days

### Phase A1: Provision node VPS `[ ]` [1 day]
- [ ] Provision dedicated VPS for kaspad (32 GB RAM recommended for mainnet)
  - See `docs/local-testnet-node.md` for current setup notes
- [ ] Install `kaspad` from rusty-kaspa v2.0.1 release
- [ ] Run with `--utxoindex` (required for UTXO verification)
- [ ] Expose wRPC Borsh port (17210 for testnet-12, 17210 for mainnet)
- [ ] Set up systemd service, monitoring, restart on crash
- [ ] Verify sync completes
- ✅ **Checkpoint:** `curl -s localhost:17210/health` returns OK
- ⚙ **Fallback:** Use public wRPC endpoint if someone runs one (kaspa.blue/green/red)

### Phase A2: Indexer wiring `[ ]` [1 day]
- [ ] Update indexer config: `--wrpc-url ws://your-node:17210`
- [ ] Remove `--no-wrpc` flag from production deployment
- [ ] Test `EscrowVerifier::verify_utxo_exists()` with real node (not MockVerifier)
- [ ] Test anchor tx broadcasting from `AnchorService`
- [ ] Update `get_utxos_by_outpoints` call in `listener.rs`
- ✅ **Checkpoint:** Indexer starts without `--no-wrpc`, verifies UTXOs against real node
- ⚙ **Fallback:** Keep MockVerifier as fallback with warning log

### Phase A3: Monitoring `[ ]` [4-6 hrs]
- [ ] Add DAA score + sync status to `/v1/network` endpoint
- [ ] Alert if node is behind by >10 blocks
- [ ] Uptime monitoring (systemd watchdog or healthchecks.io)
- ✅ **Checkpoint:** Dashboard shows "Node synced: yes" with current DAA score
- ⚙ **Fallback:** Simple cron job that emails on failure

---

## Track B: Analytics Dashboard `[ ]`

**Description:** Public analytics page at `stats.daglock.io` (or `/stats` route) showing total escrows, volume over time, KAS/USD price, active offers, and network health.

**Timebox:** 1-2 weeks

### Phase B1: Time-series data `[ ]` [2-3 days]
- [ ] New DB table `daily_stats`:
  ```sql
  CREATE TABLE IF NOT EXISTS daily_stats (
      date TEXT PRIMARY KEY,           -- "2026-07-06"
      escrows_created INTEGER DEFAULT 0,
      escrows_settled INTEGER DEFAULT 0,
      volume_sompi INTEGER DEFAULT 0,
      fees_sompi INTEGER DEFAULT 0,
      active_escrows INTEGER DEFAULT 0,
      open_offers INTEGER DEFAULT 0,
      kas_usd_price REAL,
      daa_score INTEGER
  );
  ```
- [ ] Background task: compute today's stats every hour (upsert into daily_stats)
- [ ] Include milestone/completed, subscription/completed, multi/settled in volume
- [ ] Backfill: compute stats from all existing data for past dates
- ✅ **Checkpoint:** 30 days of daily stats available in DB
- ⚙ **Fallback:** Compute on-the-fly from escrows table (slower but no new table)

### Phase B2: Stats API `[ ]` [1-2 days]
- [ ] `GET /v1/stats/daily?days=30` — returns array of daily snapshots
- [ ] `GET /v1/stats/summary` — live totals (total escrows, total volume, all-time fees, active users)

Read `/home/dillon/_code/DagLock/indexer/src/api/status.rs` for existing stats endpoint pattern.

### Phase B3: Stats page `[ ]` [3-5 days]
- [ ] New page at `/stats` (or `stats.daglock.com` subdomain)
- [ ] Chart: escrows created/settled per day (last 30d)
- [ ] Chart: volume per day (KAS)
- [ ] Chart: KAS/USD price overlay
- [ ] Cards: all-time totals, active escrows, open offers, total fees collected
- [ ] "What is DagLock?" summary box for new visitors
- [ ] Use a simple charting library (Chart.js or recharts — check what's already in package.json)
- ✅ **Checkpoint:** `/stats` shows 4 charts + summary cards, all data loading from API
- ⚙ **Fallback:** Static page, skip charts, just show numbers

### Phase B4: Network health widget `[ ]` [1 day]
- [ ] Node sync status (synced / behind N blocks)
- [ ] wRPC connection status
- [ ] Uptime
- [ ] Last anchor tx DAA score (for chat anchoring)
- ✅ **Checkpoint:** Health widgets update in real-time on the stats page
- ⚙ **Fallback:** Just show last updated timestamp

---

## Track C: Trading Bot API `[ ]`

**Description:** Sell API key tiers with rate limits. Free (10 req/min), Pro ($10/mo, 100 req/min + webhooks), Whale ($100/mo, 1000 req/min + priority). Infrastructure already exists (rate limiter, api_keys table, webhook dispatch).

**Timebox:** 1 week

### Phase C1: Rate limit tiers `[ ]` [2-3 days]
- [ ] Read `/home/dillon/_code/DagLock/indexer/src/ratelimit.rs` — understand current rate limiter
- [ ] Add `tier` field to `api_keys` table: "free" | "pro" | "whale"
- [ ] Update rate limiter to check API key tier:
  - Free: 10 req/min (current default)
  - Pro: 100 req/min
  - Whale: 1000 req/min
- [ ] Rate limit key lookup: `api_key → tier` (cached with TTL)
- ✅ **Checkpoint:** API key with tier="pro" gets 100 req/min, "free" gets 10
- ⚙ **Fallback:** Hardcoded tiers, no DB lookup

### Phase C2: Webhook tier gating `[ ]` [1 day]
- [ ] Only Pro+ tiers can register webhooks
- [ ] Whale tier gets priority delivery (dedicated queue)
- [ ] Update `services/webhooks.rs` to check key tier
- ✅ **Checkpoint:** Free tier can't register webhook, Pro can
- ⚙ **Fallback:** All tiers can use webhooks (no gating)

### Phase C3: Billing UI `[ ]` [2-3 days]
- [ ] Web page `/pricing` showing tier comparison table
- [ ] "Buy Pro" / "Buy Whale" buttons
- [ ] For now: manual process (email daglock@ to upgrade → admin changes tier in DB)
- [ ] Web UI shows current tier on API keys page
- [ ] Later: Stripe/Paddle integration (deferred)
- ✅ **Checkpoint:** `/pricing` page exists, admin can set tier via API
- ⚙ **Fallback:** No billing UI, just API-level tier enforcement

### Phase C4: API key management `[ ]` [1-2 days]
- [ ] Read existing `/home/dillon/_code/DagLock/indexer/src/api/apps.rs`
- [ ] Add tier display to key list
- [ ] Admin endpoint: `POST /v1/admin/keys/:key_id/upgrade` (requires admin auth)
- ✅ **Checkpoint:** Can create key, see tier, upgrade via admin endpoint
- ⚙ **Fallback:** Manual DB update

---

## Track D: Price Oracle Improvements `[ ]`

**Description:** CoinGecko KAS/USD fetch and cache already works. Add price alerts (notify user when KAS hits a target), historical settlement price charts, and price-at-creation display improvements.

**Timebox:** 3-5 days

### Phase D1: Price alerts `[ ]` [2-3 days]
- [ ] New DB table `price_alerts`: id, address, target_price, direction (above/below), created_at, triggered_at
- [ ] `POST /v1/price-alerts` — create alert
- [ ] `GET /v1/price-alerts` — list alerts
- [ ] `DELETE /v1/price-alerts/:id` — delete alert
- [ ] Background task: every 5 min check CoinGecko price, compare against active alerts
- [ ] On trigger: send email notification + WebSocket event
- [ ] Mark alert as triggered (one-shot)
- ✅ **Checkpoint:** Price alert fires email when KAS crosses target
- ⚙ **Fallback:** No alerts, just display current price (already done)

### Phase D2: Settlement price chart `[ ]` [1-2 days]
- [ ] Web UI: chart showing escrow creation price vs settlement price
- [ ] Scatter plot: each settled escrow as a dot (creation time × settlement profit/loss)
- [ ] Aggregate chart: average settlement price per day overlay on market price
- ✅ **Checkpoint:** Settlement price chart on `/escrows` page
- ⚙ **Fallback:** Table view of prices, no chart

### Phase D3: Price display improvements `[ ]` [1 day]
- [ ] Escrow detail: show KAS amount + USD value at creation
- [ ] Offer board: show KAS amount + USD value
- [ ] Add `price_currency` to stats endpoint
- ✅ **Checkpoint:** USD values shown alongside KAS on all escrow/offer pages
- ⚙ **Fallback:** KAS-only display (current state)

---

## Track E: Escrow-as-a-Service Widget `[ ]`

**Description:** `<daglock-pay>` web component that any website can embed. Buyer sends KAS → escrow → seller ships → buyer confirms → release. Like Stripe but for crypto P2P.

**Timebox:** 2-3 weeks

### Phase E1: Web component `[ ]` [1 week]
- [ ] Create `web/src/components/daglock-pay.ts` — vanilla JS custom element
  - `<daglock-pay amount="100" seller="kaspa:..." memo="Widget design" oncomplete="callback">`
- [ ] API key auth: widget registers as an app, uses API key
- [ ] Inline checkout flow: shows escrow terms, generates tx, links to KasWare/kaspium
- [ ] Status polling: shows "Waiting for confirmation..." → "Funds locked!" → "Waiting for release..." → "Complete!"
- [ ] No redirect — embedded in the host page
- ✅ **Checkpoint:** `<daglock-pay>` renders on a raw HTML page, completes a full escrow cycle
- ⚙ **Fallback:** Redirect to `daglock.com/pay/:id` instead of inline component

### Phase E2: Webhook delivery system `[ ]` [3-5 days]
- [ ] Read `/home/dillon/_code/DagLock/indexer/src/services/webhooks.rs`
- [ ] Ensure webhooks fire on all escrow lifecycle events (created, funded, settled, refunded, disputed)
- [ ] Webhook payload: `{ event, escrow_id, status, amount, buyer, seller, timestamp }`
- [ ] Retry: 3 attempts with exponential backoff (1min, 5min, 30min)
- **Checkpoint:** Merchant receives webhook POST when escrow is settled
- ⚙ **Fallback:** Merchant polls status endpoint

### Phase E3: Documentation + demo `[ ]` [2-3 days]
- [ ] Integration guide at `/docs/widget`
- [ ] Live demo page with working `<daglock-pay>` instance
- [ ] Copy-paste code snippet
- ✅ **Checkpoint:** `/docs/widget` shows "Add this to your site:" with ready-to-use HTML
- ⚙ **Fallback:** Just the component, no docs page

### Phase E4: Embedded checkout API `[ ]` [2-3 days]
- [ ] `POST /v1/pay` — create a checkout session (returns embed URL + metadata)
- [ ] `GET /v1/pay/:id` — checkout session status
- [ ] Checkout flow: embedded iframe or redirect
- ✅ **Checkpoint:** Can create a pay link and complete checkout
- ⚙ **Fallback:** Skip checkout API, just offer the web component

---

## Track F: Interactive "Try to Break" Demo `[ ]`

**Description:** Educational page where users click attack types (steal funds, redirect payout, forge evidence, timeout attack) and see the covenant reject each one. OfficeForge has this and it's their most persuasive marketing element.

**Timebox:** 2-3 days

### Phase F1: Attack scenarios `[ ]` [1 day]
- Define attack scenarios with clear explanations:
  1. "Arbiter tries to steal" — covenant only allows buyer/seller/fee as destinations
  2. "Server changes fee" — fee percentage is hardcoded in covenant
  3. "Seller ships nothing" — buyer disputes within window
  4. "Buyer ghosts after receiving" — auto-settle protects seller
  5. "Arbiter disappears" — emergency timeout returns funds
  6. "Chat evidence forged" — E2E encryption + on-chain anchoring prevents this

### Phase F2: Interactive page `[ ]` [1-2 days]
- [ ] New page at `/security` (or `/demo`)
- [ ] Six cards, one per attack, each with:
  - Attack name and description
  - "Try it" button
  - Animation/result showing the covenant rejecting the attack
  - "Why it works" explanation
- [ ] Simple CSS animations (covenant as a shield, attack bounces off)
- [ ] Mobile-responsive
- ✅ **Checkpoint:** `/security` page shows all 6 attack scenarios with working interactive elements
- ⚙ **Fallback:** Static page with text explanations only (no animations)

### Phase F3: Link from landing page `[ ]` [2-3 hrs]
- [ ] Add "🔒 Try to break the escrow" CTA button on the landing page
- [ ] Link to `/security`
- ✅ **Checkpoint:** Landing page has trust-building CTA linking to interactive demo
- ⚙ **Fallback:** Simple text link

---

## Track G: Deal Type Presets `[ ]`

**Description:** Add preset deal types (Goods, OTC, Service) to the escrow creation flow. Each type has sensible defaults for dispute window duration.

**Timebox:** 1 day (frontend-only)

### Phase G1: Preset definitions `[ ]` [2-3 hrs]
- Define presets:
  - **Goods**: dispute window 72h (3 days), auto-settle 72h, suggested for physical items
  - **OTC**: dispute window 24h (1 day), auto-settle 24h, suggested for KAS/KRC-20 trades
  - **Service**: dispute window 120h (5 days), auto-settle 120h, suggested for freelance work
  - **Custom**: user sets their own values (current behavior)

### Phase G2: UI changes `[ ]` [3-5 hrs]
- Read `/home/dillon/_code/DagLock/web/src/pages/EscrowsPage.tsx` (CreateEscrow component)
- Add radio button group at top of create form: Goods / OTC / Service / Custom
- On selection, auto-fill:
  - Dispute mode → set to "standard" (or "jury" for high-value?)
  - Auto-settle timeout → set to preset value
  - Memo → pre-fill with deal type (editable)
- Show a brief description under each preset ("Recommended for physical goods — 72h dispute window")
- ✅ **Checkpoint:** Selecting "Goods" auto-fills dispute timeout to 72h
- ⚙ **Fallback:** Only show presets, don't auto-fill (user still picks values)

### Phase G3: Offer board integration `[ ]` [1-2 hrs]
- [ ] Show deal type badge on offer cards (🛒 Goods / 🤝 OTC / 🛠️ Service)
- [ ] Filter offers by deal type
- ✅ **Checkpoint:** Offer board shows type badges + filter
- ⚙ **Fallback:** No type filter, just badges

---

## Track H: DAA-Block Timing for Vault Contracts `[ ]`

**Description:** Convert vault contracts from absolute `tx.time` (Unix timestamp) to relative `this.age` (DAA block count). This is a more covenant-native semantic for vaults, which enforce minimum holding periods.

**Timebox:** 1 week

### Phase H1: Convert vault contracts `[ ]` [2-3 days]
- [ ] `daglock_vault.sil`: 
  - Rename `timeout` → `lockDuration` (blocks)
  - Rename `heirTimeout` → `inheritLockDuration` (blocks)
  - Change `require(tx.time >= timeout)` → `require(this.age >= lockDuration)`
  - Same for `sweep`, `heir_withdraw`, `early_exit`
- [ ] `daglock_vault_softlock.sil`: same pattern
- [ ] `daglock_vault_multisig.sil`: same pattern
- [ ] Template hashes will change — update `AGENTS.md` and config defaults
- [ ] Read `/home/dillon/_code/DagLock/contracts/src/daglock_vault.sil` and sibling vault files
- ✅ **Checkpoint:** `cargo test -p daglock-contracts` passes with all vault tests
- ⚙ **Fallback:** Keep both params (timeout + lockDuration) for backward compat

### Phase H2: Indexer alignment `[ ]` [1 day]
- [ ] Update `expiration_daa_score` to match vault's new `lockDuration` semantic
- [ ] When vault is created: `expiration_daa_score = current_daa + lockDuration`
- [ ] Indexer sweeper already uses DAA score — will work naturally
- [ ] Read `/home/dillon/_code/DagLock/indexer/src/listener.rs` (vault sweep logic)
- ✅ **Checkpoint:** Vault sweep triggers at correct DAA score, matching on-chain covenant
- ⚙ **Fallback:** Keep old timestamp-based expiration alongside new block-based

### Phase H3: API conversion layer `[ ]` [1-2 days]
- [ ] Web UI: accept "1 day" / "7 days" / "30 days", convert to block count (1 block ≈ 1 sec)
- [ ] CLI: accept human-readable durations, convert
- [ ] Display: show "Locks for ~N days" (convert blocks back to days for display)
- [ ] Read `/home/dillon/_code/DagLock/cli/src/commands/vaults.rs` if it exists
- ✅ **Checkpoint:** User sets "7 days", vault covenant enforces `this.age >= 604800`
- ⚙ **Fallback:** Developer enters raw block count, no conversion

### Phase H4: Docs update `[ ]` [4-6 hrs]
- [ ] Update template hashes in `AGENTS.md`
- [ ] Update vault feature docs in `docs/wiki/`
- [ ] Add note about DAA-block timing to changelog
- ✅ **Checkpoint:** All docs reference block counts, not timestamps, for vaults
- ⚙ **Fallback:** Keep old docs, add migration note

---

## Execution Strategy

```
Priority 1 (Infrastructure — unlocks everything else):
  Track A — Self-hosted Kaspa node (2-3 days)

Priority 2 (Revenue + trust):
  Track C — Trading bot API (1 week)          ← first revenue
  Track F — Interactive demo (2-3 days)       ← trust building

Priority 3 (User-facing improvements):
  Track G — Deal type presets (1 day)          ← quick UX win
  Track H — DAA vault timing (1 week)          ← security + alignment
  Track D — Price oracle (3-5 days)

Priority 4 (Growth):
  Track B — Analytics dashboard (1-2 weeks)
  Track E — Escrow widget (2-3 weeks)
```

Tracks within the same priority can run in parallel. Track A is the foundation — everything else depends on having a real node.

---

## Anti-scope (not included)

- Volume-based fee rebates (explicitly excluded by user)
- Cross-chain BTC/ETH (already in PENDING.md, 2-3 months)
- AMM (already deferred, 2-3 months)
- Over-collateralized stablecoin (3-4 months)
- KRC-20 launchpad (already built!)
- Full KRC-20 token explorer (deferred)
- Mobile native app
