# Strategic Roadmap — Research Findings

## KRC-20 Market Opportunity

Kaspa has a thriving KRC-20 token ecosystem (NACHO, GHOST, KASPY, etc.) with no trustless OTC solution. Most trades happen over Telegram with manual trust — "I'll send the KAS first, then you send the tokens." DagLock is uniquely positioned to be the default escrow layer for these communities.

## Notification Infrastructure

- **Telegram notifications** can use the existing bot infrastructure — the bot already runs on the VPS and connects to the indexer
- **Email notifications** require an SMTP service (SendGrid free tier: 100 emails/day) and a new settings page for users to opt in
- **Push notifications via PWA** require a service worker + manifest.json (~2hrs work)

## On-Chain Reputation Standard

There is no on-chain reputation standard on Kaspa yet (no ERC-like equivalent). A KIP-17/KIP-20 compatible covenant could record trade outcomes. This would:
- Make reputation portable across Kaspa dApps
- Create a protocol-level moat for DagLock
- Build network effects (more dApps using it = more data = more value)

## Cross-Chain HTLC Feasibility

Kaspa's SilverScript supports hash-locked covenants. Bitcoin's scripting supports HTLCs. The challenge is:
- Running a Bitcoin node (or using an API) to track BTC transactions
- Matching timeouts across two chains with different block times
- The relayer/indexer needs to monitor both chains

## Embeddable Widget

There's already a `widget/` directory in the repo with an unused `<daglock-escrow>` custom element. This could be resurrected and polished for distribution to partner sites.

## Fee Rebates

- 0.5% is high for whales doing 100K+ KAS trades
- Rebates can be off-chain (treasury sends refund monthly)
- Tracking: indexer already records volume per address
- Design: 30-day rolling window, automatic tiers

## CoinGecko Price Oracle

The indexer already has:
- `listener.rs` — `update_market_prices()` fetches KAS/USD every 15 min
- `offers.rs` — market price creation handler
- DB columns for price data on offers/escrows
What's missing: full wiring between creation → lock → settlement to capture price
