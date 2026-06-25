# Kaspa Ecosystem Research — June 21, 2026

## Methodology
Searched GitHub (80+ topic repos, keyword searches, individual repo commits), the official Kaspa build page, specific repos, and issue trackers. All timestamps as of June 21, 2026.

---

## Who's Building What

### L2 EVM DeFi (Complementary, Not Competing)

| Project | What | Layer | Status |
|---------|------|-------|--------|
| **KASPACOM DeFi** | Uniswap V2 fork (DEX), Aave V3 fork (lending), launchpad. WKAS token, liquidity pairs, portfolio queries. | IGRA & Kasplex EVM L2 | ✅ Active |
| **zenithlaunch** | Bonding curve launchpad (Solidity contracts). Factory + Raise + Token. | Kaspa L2 (EVM) | ✅ Active |

**Key insight:** These are EVM-based L2 solutions, NOT native Kaspa covenant-level DeFi. Different trust model, different users. Complementary — users could escrow via DagLock's L1 covenants then trade on L2 DEX.

### L1 Covenant Infrastructure (Patterns, Not Products)

| Project | What | Status |
|---------|------|--------|
| **kaspanet/silverscript** | Compiler, KCC20 token support, ternary, debugging. **No AMM/DEX patterns.** | 🔥 Very active |
| **kaspanet/vprogs** | ZK bridge/L2 settlement — RISC0, SMT, Groth16 settlement covenant. | 🔥 Very active |
| **trillskillz/OpenSilver** | 22 covenant patterns (Ownable, MultiSig, Vault, Bilateral Escrow, HTLC, Vesting, KCC20 controllers, ZK). **No AMM/CPMM/DEX.** | ✅ Active (May 25) |
| **trillskillz/KasGraph** | The Graph for Kaspa — subgraph indexer with GraphQL, MCP, WebSocket, KRC-20/KRC-721 support. | ✅ Active (Jun 2) |
| **THTProtocol/Covex** | Covenant explorer + visual Covenant Studio (Canva/Framer-like). Premium tiers. | 🔥 Active (Jun 11) |
| **trillskillz/KasBonds** | Service Bond Protocol — KAS-native bond primitive. | ✅ Active (May 22) |

### Near-Competitors (Stalled or Incomplete)

| Project | What | Why Not a Threat |
|---------|------|------------------|
| **atharaldsen/kaspa-marketplace** | Next.js marketplace with escrow (create/fund/release/refund/dispute). | Stalled Feb 2026. No Telegram bot, no KRC-20, no offer board, no reputation. Goods marketplace, not escrow-first. |
| **cliffc2/kaspa-atomic-swap-cli** | Rust CLI for HTLC covenants. Basic initiate/claim/refund. | Placeholder. Single HTLC pattern only. No product. |
| **SE-XPRT/KaspaSwapBot** | Telegram P2P swap bot. | Generic P2P, not escrow. Not updated Aug 2025. |

### Dead / Abandoned

| Project | Status |
|---------|--------|
| mahadhussaini/kaspa-dex | Empty repo |
| mirzausman371/kaspa-dex-v3 | Empty repo |
| thesheepcat/kaspa-swap | Abandoned Feb 2023 |
| sonotullio/kaspa-swap | Abandoned Jun 2023 |
| DagSwap/dagswap | Archived |
| coinoswap/coinoswap | Last commit Aug 2025 |
| Kasplex (aspectron) | Empty init only (Jun 2024) |

---

## Open Space Analysis — What Nobody Else Is Building

### ✅ Unique to DagLock (No Competition)

| Feature | Notes |
|---------|-------|
| **KAS native escrow with covenants** (trustless, L1) | OpenSilver has generic BilateralEscrow pattern but no product. KaspaMarketplace stalled. |
| **KRC-20 token escrow** | Nobody else does this. KaspaCom is EVM L2 (different paradigm). |
| **Telegram bot for escrow creation** | Only Kaspa escrow bot. KaspaSwapBot is generic P2P, not escrow. |
| **Proposal-before-commit flow** | Not found in any other project. |
| **Counterparty discovery / offer board** | Not found. KaspaMarketplace is a goods marketplace, not an escrow offer board. |
| **On-chain reputation system** | Not found anywhere in the ecosystem. |
| **Arbiter/mediation covenant** | OpenSilver has bilateral escrow but no arbitration. |
| **Settlement receipts** | Not found. |
| **Volume-based fee tiers** | Not found. |
| **Time-locked vault (productized)** | OpenSilver has vault pattern, but no service. |
| **CLI + Web + Bot (3-surface unified)** | No project covers all three surfaces. |
| **SQLite/Postgres indexer with REST API** | KasGraph is closest but it's a general subgraph indexer. |

### 🟢 Open Space — Planned Features With No Competition

| Feature | Anyone building it? |
|---------|---------------------|
| **AMM in SilverScript** | ❌ Nobody |
| **Native KAS stablecoin (vaults)** | ❌ Nobody |
| **DAO / treasury management** | ❌ Nobody |
| **KRC-20 token explorer + charts** | ❌ KRC-20 launched days ago (Toccata June 30) — too new |
| **KRC-20 launchpad / token creator** | ❌ KaspaCom has L2 launchpad, different paradigm |
| **Trading bot API** | ❌ Nobody |
| **Escrow payment widget** | ❌ WooCommerce plugin exists but basic KAS-only, no escrow |
| **Cross-chain BTC/ETH** | ❌ OpenSilver has HTLC pattern but no cross-chain implementation |
| **Memecoin platform** | ❌ Nobody |

---

## The First-Mover Window

**DagLock has approximately a 6-12 month first-mover advantage on L1 covenant DeFi products.**

The Kaspa DeFi space is bifurcated:
- **L2 EVM DeFi** (KaspaCom, zenithlaunch) — DEX, lending, launchpad on IGRA/Kasplex EVM. Complementary to DagLock.
- **Native L1 covenants** (DagLock, OpenSilver, KasBonds, Covex) — Only DagLock is a product. Everything else is infrastructure/patterns.

Toccata activates June 30. That's the starting gun. After activation, anyone can write covenant products — but no one has a head start like DagLock's existing codebase, contracts, tests, indexer, and three-surface UI.

---
*Research date: June 21, 2026*
*Sources: GitHub topic page (80 repos), kaspa.org/build, individual repo analysis*
