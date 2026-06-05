# DagLock — Product Roadmap

## Current State

DagLock is a testnet escrow platform with:
-  REST API (Rust backend)
-  Web dashboard (React)
-  Telegram bot
-  CLI tool
-  Basic escrow/offer functionality
-  No real KRC-20 trading
-  No on-chain verification
-  No wallet integration
-  No market price feeds

---

## What Makes DagLock Actually Useful

### Must-Have Features (Phase 1-2)

| Feature | Why It's Needed | Effort |
|---------|-----------------|--------|
| **wRPC Listener** | Detect UTXOs on-chain | 2-3 days |
| **Wallet Integration** | Sign transactions with KasWare | 3-5 days |
| **Real KRC-20 Trading** | Trade tokens, not just KAS | 5-7 days |
| **Market Price Oracle** | Real-time pricing | 1-2 days |
| **Price-Locked Offers** | Set price at market rate | 2-3 days |

### Nice-to-Have Features (Phase 3-4)

| Feature | Why It's Needed | Effort |
|---------|-----------------|--------|
| **Limit Orders** | Set price, auto-execute | 3-5 days |
| **Market Orders** | Instant trade | 2-3 days |
| **DCA** | Recurring purchases | 3-5 days |
| **Tax Reporting** | Export history | 2-3 days |
| **Mobile App** | Wider reach | 4-6 weeks |

---

## Market Price Integration — How Hard?

**Current state:** Offers have fixed prices set by creator.

**Desired state:** Offers automatically adjust to market price.

**Implementation:**

```
Current: User creates offer → sets price → price is fixed
Desired: User creates offer → price updates with market → trades at current price
```

**What's needed:**

1. **Price Feed** (already have CoinGecko integration)
   - `GET /v1/network/price` returns `kas_usd`
   - Can extend for KRC-20 tokens

2. **Price-Locked Offers**
   - Store `price_type: "fixed" | "market"`
   - Store `price_offset: 0` (optional +/- from market)
   - Auto-update offer price when market changes

3. **Price Protection**
   - Set `min_price` and `max_price` bounds
   - Prevents flash crash exploitation
   - User controls slippage tolerance

**Example flow:**
```
User: "Sell 1000 KAS at market price"
System: Fetches current KAS/USD ($0.15)
        Creates offer: 1000 KAS @ $0.15
        If market moves to $0.16 → offer updates
        If market moves to $0.14 → offer updates
        Counterparty sees current price when accepting
```

**Effort: 2-3 days**

**Code changes:**
1. Add `price_type` and `price_offset` to offers table
2. Add price update logic to reconciliation loop
3. Update offer display to show "market price"
4. Add price protection bounds

---

## What's Actually Useful?

The **minimum viable product** for real trading needs:

| Priority | Feature | Status |
|----------|---------|--------|
| **P0** | wRPC listener (on-chain detection) |  Missing |
| **P0** | Wallet integration (signing) |  Missing |
| **P0** | Market price oracle |  Built |
| **P1** | Price-locked offers |  Missing |
| **P1** | KRC-20 token support |  Built (covenant) |
| **P2** | Limit orders |  Missing |
| **P2** | Tax reporting |  Missing |

**Without wRPC and wallet integration, the system is a simulation.** These are the critical missing pieces.

**Want me to create a detailed implementation plan for the P0 features?**
