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

## Self-Hosted Kaspa Node for wRPC

**Status:** Not started
**Difficulty:** Medium (2-3 days setup + monthly hosting)

Currently the indexer uses the Public Node Network (PNN) Resolver to discover a Kaspa node, or runs in offline mode (MockVerifier). A self-hosted node would:

- Eliminate resolver dependency
- Guarantee wRPC availability
- Enable UTXO index for faster verification
- Cost: ~$50-100/month for a VPS (8GB RAM, 200GB SSD)

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

## Implementation Priority

```
Priority 1 (Post-Launch, Q3 2026):
  └─ Self-hosted Kaspa node (enables real UTXO verification)

Priority 2 (After traction):
  └─ Atomic swap wizard UI
  └─ Price oracle improvements
  └─ Analytics dashboard

Priority 3 (After users ask):
  └─ Volume-based fee rebates
  └─ Cross-chain BTC

Priority 4 (If Kaspa DeFi grows):
  └─ AMM
```
