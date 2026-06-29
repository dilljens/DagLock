# Kaspa Adoption Features — What DagLock Could Build Next

**Goal:** Identify and plan the highest-impact features DagLock could build to solve real problems preventing Kaspa from being useful to average people.

**Status:** Research complete. Plan for prioritization.

---

## Research Summary

Kaspa has excellent infrastructure (10 BPS, DAG, SilverScript covenants, $765M market cap) but **zero DeFi, zero token trading, zero reputation, zero multi-surface products**. The Toccata hard fork (June 30, 2026) enables KRC-20 tokens and L1 covenants — the starting gun for the ecosystem.

### What DagLock Already Covers

| Need | Status |
|------|--------|
| Trustless P2P escrow (KAS + KRC-20) | ✅ Built |
| Offer board / counterparty discovery | ✅ Built |
| On-chain reputation | ✅ Built |
| Telegram bot + Web UI + CLI | ✅ Built |
| Dispute resolution (arbiter/jury) | ✅ Built |
| Settlement receipts | ✅ Built |
| Time-locked vaults | ✅ Built |

### What's Still Missing (Opportunities for DagLock)

| # | Gap | Impact | Feasibility | Description |
|---|-----|--------|-------------|-------------|
| 1 | **KRC-20 token explorer + charts** | High — first mover | Medium | DexScreener-like dashboard for KRC-20 tokens. Price, volume, holders, transfers. Indexer already sees all activity. |
| 2 | **KRC-20 token launchpad** | High — drives ecosystem | Medium | Web UI + compile API integration to create KRC-20 tokens in a few clicks. Like pump.fun for Kaspa. |
| 3 | **Atomic swap wizard UI** | Medium — improves existing | Low | Step-by-step guided swap UI for the existing atomic swap covenant. Generate secret → share hash → counterparty locks → reveal preimage. |
| 4 | **Mobile PWA improvements** | Medium — mobile users | Low | The web UI works on mobile but isn't great. Better responsive design + PWA install prompts. |
| 5 | **Multi-wallet support (Kaspium)** | Medium — broader reach | Low | Currently KasWare only. Add Kaspium wallet connection for mobile users. |
| 6 | **Analytics dashboard** | Medium — community tool | Medium | Public stats: total escrows, volume, fees collected, active users, network health. Already have `/v1/stats`. |
| 7 | **Price oracle + alerts** | Low — nice-to-have | Low | CoinGecko feed with configurable price alerts (Telegram notification when KAS hits a target). |
| 8 | **KAS/USD fiat on-ramp** | High — biggest barrier | High | Let users buy KAS with fiat inside the app. Requires partnership with a fiat gateway (MoonPay, Onramp, etc.) — weeks of integration work. |

---

## Track A: KRC-20 Token Dashboard (Explorer + Charts) `[ ]`

**Why this matters most:** When KRC-20 tokens launch (June 30), the first thing users will want is to **see** them — prices, volume, holders. If DagLock is the place to do that, it becomes the default KRC-20 portal. No one else is building this.

**⏱ Timebox:** 1 week

### Phase A1: Indexer data pipeline `[ ]`
- [ ] Add KRC-20 token metadata tracking to indexer (token supply, holders via ICC events)
- [ ] Add price tracking table (trade price history from escrow settlements)
- [ ] Create API endpoints: `/v1/tokens`, `/v1/tokens/:ticker`, `/v1/tokens/:ticker/chart`
- ✅ Checkpoint: `curl /v1/tokens` returns list of tokens with price/volume
- ⚙ Fallback: Start with minimal fields (ticker, supply, trades count)

### Phase A2: Web UI `[ ]`
- [ ] Token directory page: sortable table of all KRC-20 tokens (price, volume, change, holders)
- [ ] Token detail page: price chart (7d/30d/all), recent trades, holder stats
- [ ] Link token pages to existing escrow/offer creation (one-click "Buy NACHO")
- ✅ Checkpoint: daglock.com/tokens renders with mock data
- ⚙ Fallback: Simpler version — just a list page without charts

---

## Track B: KRC-20 Token Launchpad `[ ]`

**Why this matters:** Every new KRC-20 token is a potential DagLock escrow user. If DagLock is the easiest place to create a token, it gets first access to every project. No one else offers this yet.

**⏱ Timebox:** 3-4 days

### Phase B1: API integration `[ ]`
- [ ] The `/v1/compile` endpoint already exists — wire it to a new "deploy token" endpoint
- [ ] Create `/v1/tokens/deploy` that: compiles the KRC-20 minter covenant → returns address
- ✅ Checkpoint: `curl /v1/tokens/deploy` returns a covenant address
- ⚙ Fallback: Compile-only without on-chain deployment (user broadcasts manually)

### Phase B2: Web UI `[ ]`
- [ ] Token creation form: name, ticker, supply, mint schedule, ownership mode
- [ ] Show deployment status (compiling → address → pending broadcast)
- [ ] Link to token dashboard after creation
- ✅ Checkpoint: Web UI creates a KRC-20 token record in the indexer
- ⚙ Fallback: Basic form without mint schedule options

---

## Track C: Atomic Swap Wizard `[ ]`

**Why this matters:** Atomic swaps are powerful but confusing. A step-by-step wizard demystifies them and makes the covenant's swap entrypoint actually usable.

**⏱ Timebox:** 1-2 days

### Phase C1: Swap wizard UI `[ ]`
- [ ] Step 1: Generate secret → compute SHA-256 hash → display both
- [ ] Step 2: Share hash with counterparty (copy link / QR)
- [ ] Step 3: Wait for counterparty to lock funds with the hash
- [ ] Step 4: Submit preimage → settle → receipt
- ✅ Checkpoint: Full wizard flow working on testnet
- ⚙ Fallback: Inline with existing SwapPage — add guided mode toggle

---

## Track D: Mobile & Wallet Improvements `[ ]`

**Why this matters:** Kaspa's community lives on Telegram (mobile). If the web UI doesn't work well on phones, you lose most of your audience.

**⏱ Timebox:** 2-3 days

### Phase D1: PWA & responsive fixes `[ ]`
- [ ] Audit all pages for mobile breakpoints (sidebar off-canvas, form widths, button sizing)
- [ ] Add PWA install prompt for returning visitors
- [ ] Fix known mobile issues (overlapping buttons, truncated text)
- ✅ Checkpoint: Lighthouse mobile score passes minimum
- ⚙ Fallback: Focus only on the most-used pages (Dashboard, Escrows, Offers)

### Phase D2: Multi-wallet support `[ ]`
- [ ] Research Kaspium wallet connect protocol (deep link / WebSocket)
- [ ] Add Kaspium as a connect option alongside KasWare
- [ ] Test on mobile: connect → create escrow → sign → broadcast
- ✅ Checkpoint: Kaspium wallet flow works end-to-end
- ⚙ Fallback: Manual mode with copy-paste for mobile (already works)

---

## Prioritization

| Priority | Feature | Effort | Impact | When |
|----------|---------|--------|--------|------|
| **P0** | Token dashboard | 1 week | 🔴 High — first-mover advantage for KRC-20 | Week of June 30 (Toccata) |
| **P1** | Token launchpad | 3-4 days | 🔴 High — drives token creation → escrow usage | After dashboard |
| **P2** | Atomic swap wizard | 1-2 days | 🟡 Medium — unlocks existing feature | After launchpad |
| **P3** | Mobile/PWA | 2-3 days | 🟡 Medium — mobile users | After swap wizard |
| **P4** | Kaspium support | 2-3 days | 🟢 Low — broader wallet access | Post-launch |
