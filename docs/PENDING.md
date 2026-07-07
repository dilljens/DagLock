# DagLock — Pending Plans

> Post-launch features, research items, and larger projects beyond the current scope.

---

## AMM (Automated Market Maker)

**Status:** Research / Pre-planning
**Dependency:** Toccata hard fork activated + KRC-20 tokens active on mainnet
**Difficulty:** Hard (2-3 months with 1 engineer)

### What It Would Be

A constant-product AMM covenant (`daglock_amm.sil`) that holds two assets (KAS + KRC-20 token) and allows trustless swaps:

```
Liquidity Pool Covenant:
  - Holds: KAS + KRC-20 tokens (via KCC-20 branch, same ICC pattern as KRC-20 escrow)
  - Pricing: x * y = k (constant product formula)
  - Actions: swap, addLiquidity, removeLiquidity
  - LP tokens: minted as KCC-20 branches to track share of pool
```

### How It Integrates With DagLock

| Existing DagLock piece | Used for AMM |
|------------------------|-------------|
| `daglock_krc20.sil` ICC pattern | LP token ownership via KCC-20 branches |
| Template hash detection | Pool UTXO identification |
| Indexer + wRPC | Pool state tracking, TVL, volume |
| Offer board (extended) | Pool discovery and liquidity opportunities |
| Reputation system | LP provider reputation |
| Jury system | Dispute resolution for pool exploits |

### What Would Need Building

| Component | Effort |
|-----------|--------|
| `daglock_amm.sil` covenant (swap, addLiquidity, removeLiquidity) | 3-4 weeks |
| LP token mint/burn logic (KCC-20 branches) | 1 week |
| Price impact + slippage protection | 1 week |
| Indexer: pool state queries, TVL, volume tracking | 1-2 weeks |
| Web UI: pool browser, swap interface, LP management | 2-3 weeks |
| Execution tests for all AMM paths | 1 week |

### Why It's Post-Launch

1. No one will use an AMM on a network without users. Escrow first.
2. KRC-20 tokens need to exist and have volume first.
3. The covenant math is complex — one bug could drain pools.
4. Requires a security audit before mainnet deployment.

---

## Atomic Swap Wizard UI

**Status:** Deferred
**Difficulty:** Medium (1 week)

The covenant supports atomic swaps via hash preimage (`swap(secret)` entrypoint). The web UI doesn't have a guided wizard for it. Users must manually enter the trade hash and preimage.

**What's needed:**
- A step-by-step swap UI: Generate secret → Share hash → Wait for counterparty → Reveal preimage → Settle
- Inline with the existing escrow creation flow
- Time-lock countdown display

---

## Price Oracle (CoinGecko KAS/USD)

**Status:** Partially done — basic CoinGecko fetch + cache exists
**Difficulty:** Easy (2-3 days for full implementation)

**What's missing:**
- On-chain recording: store price at escrow creation/settlement time (partially done)
- Integration with the covenant? No — covenants can't access external data
- Dashboard price charts for historical settlement prices
- Price alerts: notify user when KAS hits a target

---

## Analytics Dashboard

**Status:** Not started
**Difficulty:** Medium (1-2 weeks)

Public dashboard showing:
- Total escrows, volume, fees (already have `/v1/status` and `/v1/stats`)
- Settlement volume over time (daily/weekly/monthly)
- Active offer board depth
- KAS/USD price chart
- Network health (wRPC status, DAA score, uptime)

What's needed:
- Time-series data storage (separate DB table or Prometheus)
- Grafana or custom React dashboard
- Public page at `stats.daglock.io`

---

## Cross-Chain (BTC/ETH)

**Status:** Not started
**Difficulty:** Hard (2-3 months)

HTLC-based atomic swap between Kaspa and Bitcoin. Would allow trustless KAS/BTC trades without a centralized exchange.

**Why it's hard:**
- Requires running a Bitcoin node (or Lightweight client)
- HTLC covenant on Kaspa side + HTLC script on Bitcoin side
- Different timeouts, different hash functions (SHA-256 vs RIPEMD-160)
- Needs a relayer/indexer to monitor both chains
- Security audit required

---

## Volume-Based Fee Rebates

**Status:** Deferred
**Difficulty:** Easy (1-2 days)

The covenant always charges 0.5%. If a whale does 100K+ volume, they could get a rebate. Since the fee is in the covenant (non-bypassable), rebates would be off-chain — refund from the treasury address to the whale.

**When it matters:** When a single user does >500K KAS volume in a month.

---

## Over-Collateralized Stablecoin (DAI-style)

**Status:** Research / Deferred
**Dependency:** Toccata activated + KRC-20 tokens active + proven escrow usage
**Difficulty:** Hard (3-4 months with 1 engineer + security audit)

### What It Would Be

A vault covenant that lets users lock KAS as collateral and mint a KRC-20 stablecoin (e.g. KASD) at a minimum 150% collateral ratio. Price oracle provides KAS/USD. Liquidators close underwater positions.

### Components

| Component | Effort |
|-----------|--------|
| Vault covenant (lock KAS, track debt, enforce ratio) | 2-3 weeks |
| Price oracle covenant (signed KAS/USD feed) | 1 week |
| KCC20 minter integration (vault owns minter branch) | 1 week |
| Liquidation entrypoint (discount for repaying bad debt) | 1 week |
| Vault CRUD API + liquidation bot | 2-3 weeks |
| Frontend vault dashboard | 2-3 weeks |
| Execution tests | 2 weeks (parallel) |
| Security audit | 2-4 weeks (mandatory — vault holds user funds) |

### Why It's Post-Launch

1. Doubles the contract codebase and security surface area.
2. KRC-20 tokens need volume first — stablecoin is useless without demand.
3. Oracle dependency creates operational risk (liquidation bots, price feed uptime).
4. Ecosystem maturity: MakerDAO took years to secure their oracles.
5. DagLock needs escrow users first — that's the immediate priority.

### How It Fits DagLock Long-Term

- OTC traders want a stable asset to denominate trades in.
- Stability fees add a second revenue stream alongside escrow fees.
- Brand expands from "escrow" to "DeFi hub on Kaspa."

### Revisit When

- DagLock has 100+ active escrow users.
- KRC-20 token ecosystem has meaningful volume.
- A reliable KAS/USD oracle exists (either run your own or a community standard emerges).
- Someone asks: "Can I borrow KASD against my KAS?"

---

## KRC-20 Launchpad

**Status:** Research / Deferred
**Dependency:** Toccata activated + KRC-20 tokens active on mainnet
**Difficulty:** Easy-Medium (2-3 weeks)

### What It Would Be

A web UI + compile API integration that lets anyone create a KRC-20 token with a few clicks — set name, supply, mint schedule, ownership mode. Like pump.fun but on Kaspa.

### Why It Fits DagLock

| Existing piece | Used for |
|----------------|----------|
| KCC-20 contract knowledge | Token template (from `daglock_krc20.sil` ICC work) |
| `/v1/compile` API | Covenant compilation endpoint exists |
| Web UI | Existing React dashboard — add a token creation page |
| Indexer | Track token creation events, TVL, holder counts |

Every new token is a potential DagLock escrow user. No one else offers this yet on Kaspa.

### Revenue Angle

- Flat fee (50 KAS) per token creation
- Or take a small allocation in the new token

### Revisit When

- Toccata activates and users start asking "how do I create a token?"
- KRC-20 tokens exist and have trading volume

---

## Escrow-as-a-Service Widget

**Status:** Research / Deferred
**Difficulty:** Easy (2-3 weeks)

### What It Would Be

A `<daglock-pay>` web component that any website can drop in. Buyer sends KAS → escrow → seller ships goods → buyer confirms → release. Like Stripe but for crypto P2P.

### Why It Fits DagLock

| Existing piece | Used for |
|----------------|----------|
| `daglock.sil` | No new contracts needed — existing escrow covenant |
| REST API | Webhook callbacks on escrow status changes |
| Bot notifications | Notify buyer/seller of status changes |
| API key system | Already have app registration + rate limits |

Turns escrow from a destination product into an infrastructure product. Volume scales with integration.

### What Would Need Building

| Component | Effort |
|-----------|--------|
| Web component (`<daglock-pay>` as a vanilla JS custom element) | 1 week |
| Webhook delivery system (POST on escrow status change) | 3 days |
| Embedded checkout flow (inline, no redirect) | 3 days |
| Documentation + integration guide | 2 days |

### Revenue Angle

- The 0.5% protocol fee is already built into the covenant — no extra charge
- Value is in volume: more integrations = more escrows = more fees

### Revisit When

- Escrow has proven product-market fit (>50 active users)
- A merchant or marketplace asks: "can I integrate this into my site?"

---

## Self-Hosted Kaspa Node for wRPC

**Status:** Planned — see `docs/local-testnet-node.md` for setup steps
**Difficulty:** Medium (2-3 days setup + monthly hosting)
**Dependency:** RAM upgrade to 32 GB (scheduled ~July 13)

Currently the indexer runs in offline mode (MockVerifier) because the public wRPC resolvers (kaspa.red/green/blue) were taken offline during the Toccata v2 migration. A self-hosted node would:

- Eliminate resolver dependency (resolvers are gone — kaspa.red/green/blue NXDOMAIN)
- Guarantee wRPC verification availability (move off MockVerifier)
- Enable UTXO index for faster verification
- Cost: CPX42 (~€21/mo) is the cheapest benchmark-validated tier for mainnet

**What's needed:**
- Spin up `kaspad` with `--utxoindex` on a VPS
- Expose wRPC Borsh port (17210 for testnet)
- Point the indexer at it with `--wrpc-url ws://your-node:17210`
- Monitoring + restart on crash

---

## Volume-Based Fee Rebates

**Status:** Deferred until there's volume

Same as above — not relevant until users exist.

---

---

## Trading Bot API (with Rate Limit Tiers)

**Status:** Not started
**Difficulty:** Easy (1 week)
**Revenue:** High — recurring subscription, high-margin, low maintenance

### What It Is

Sell API keys with rate limit tiers for automated trading. Bot operators use DagLock's existing REST API (escrows, offers, reputation) programmatically.

The infrastructure already exists:
- REST API with 19+ endpoints
- App registration + API key system (`/v1/apps/register`)
- Rate limiter already implemented (30 req/min per IP)
- Manual wallet mode + mock auth for dev testing

### What Would Need Building

| Component | Effort |
|-----------|--------|
| Rate limit tiers (free/pro/whale) in config | 1 day |
| API key → tier mapping in `api_keys` table | 1 day |
| Webhook system for escrow status events | 2 days |
| Billing integration (bot payment or manual invoicing) | 2 days |
| Docs page for bot developers | 1 day |

### Tiers

| Tier | Rate limit | Webhooks | Monthly |
|------|-----------|----------|---------|
| Free | 10 req/min | No | $0 |
| Pro | 100 req/min | Yes | $10 |
| Whale | 1000 req/min | Yes + priority | $100 |

### Why It Fits DagLock

- Bot users are power users — they run 24/7 and will pay for reliability
- Low support overhead (bots don't file UX complaints)
- The offer board + escrow endpoints are already designed for programmatic use
- Existing rate limiter infrastructure just needs tier wiring

### Revisit When

- Rate limiter and API key system are live (already done)
- Offer board has 10+ active offers
- Someone asks: "Can I automate this?"

---

## KRC-20 Token Explorer + Charts

**Status:** Not started
**Difficulty:** Medium (2-3 weeks)
**Revenue:** Medium — premium API access, token listing fees

### What It Is

A token-focused explorer at `tokens.daglock.com` or integrated into the existing web UI. Shows every KRC-20 token with price, volume, holders, transfers, and charts. Like DexScreener but for Kaspa.

### What Would Need Building

| Component | Effort |
|-----------|--------|
| Token data aggregation (offers, trades, volume) | 3-5 days |
| Price history storage (time-series for charting) | 2-3 days |
| Charting UI (price chart, volume chart) | 3-5 days |
| Token directory page (sort by volume, age, etc.) | 2 days |
| Premium API tier (real-time alerts, data export) | 2 days |

### Revenue Model

- Free: basic token data, current price, 24h volume
- Premium ($50/mo): real-time price alerts, historical trade data export
- Listing fee ($100): featured token in directory with logo+description

### Why It Fits DagLock

- The indexer already sees all escrow and offer activity for KRC-20 tokens
- Every token listed is a potential DagLock escrow user
- Becomes the go-to destination for KRC-20 research → drives traffic to escrow
- Data moat — once you index all KRC-20 activity, competitors can't easily replicate

### Revisit When

- Toccata activates and KRC-20 tokens start trading
- At least 10 unique KRC-20 tokens have been traded on the offer board

---

## Implementation Priority

```
Priority 1 (Post-Launch, Q3 2026):
  └─ Self-hosted Kaspa node (enables real UTXO verification)

Priority 2 (After traction, Q3-Q4 2026):
  └─ Trading bot API + rate limit tiers   ← NEW (highest ROI)
  └─ KRC-20 launchpad (when Toccata activates, tokens need to exist)
  └─ Escrow-as-a-service widget (infrastructure play)
  └─ Atomic swap wizard UI
  └─ Price oracle improvements
  └─ Analytics dashboard

Priority 3 (After users ask):
  └─ KRC-20 token explorer + charts   ← NEW (brand + data moat)
  └─ Volume-based fee rebates
  └─ Cross-chain BTC

Priority 4 (If Kaspa DeFi grows):
  └─ Over-collateralized stablecoin
  └─ AMM
  └─ Treasury management (niche)     ← NEW
```
