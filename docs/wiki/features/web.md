# web

React + Vite dashboard for browser-based users. Provides escrow creation, claiming, offer board, and reputation views. Communicates with the indexer REST API. Uses Vitest + React Testing Library for component tests, Biome for lint.

## Rules & Conventions

- ****U2**: Web CreateEscrowForm generates fake `lock_tx_id`**
  - Status: ❌ Open | Domain: web
- ****U7**: No web onboarding for first-time users**
  - Status: ✅ Fixed | Domain: web
- ****Q7**: Web API no request timeout**
  - Status: ✅ Fixed | Domain: web
- ****Q1**: `.expect()` on UUID in code**
  - Status: ✅ Fixed | Domain: web
- ****Q2/Q3**: Magic number `200` in fee calc**
  - Status: ✅ Fixed | Domain: web

## Features

| Feature | Status | Notes |
|---------|--------|-------|
| Escrow CRUD (create, settle, refund, dispute) | ✅ | Core flow |
| Offer board (browse, create, accept, counter) | ✅ | In-app negotiation with counter-offers |
| Vaults (time-locked, beneficiary, multisig) | ✅ | Create, list, lookup |
| Jury system (cases, voting, registration) | ✅ | Community dispute resolution |
| Reputation lookup + vouching | ✅ | On-chain scores |
| Receipt lookup | ✅ | Settlement receipts |
| **Invoice creation** | ✅ **New** | Escrow-based invoice form |
| **Invoice payment page** | ✅ **New** | `daglock.com/pay/:id` |
| Manual wallet mode (testnet) | ✅ | Enter address + mock sign |
| Swap wizard (atomic) | ✅ **New** | Step-by-step 6-step guided UI at `/swap` |
| **Fee calculator** | ✅ **New** | Widget on dashboard + create form |
| **Block explorer links** | ✅ **New** | TX and address links on every page |
| **Onboarding wizard** | ✅ **New** | 3-slide first-visit modal |
| **Help center / FAQ** | ✅ **New** | `/help` page with accordion FAQ |
| **Testnet quickstart** | ✅ **New** | `/testnet` page with test wallets |
| **Settings / notifications** | ✅ **New** | `/settings` — email subscription + preferences |
| **KRC-20 token dashboard** | ✅ **New** | `/tokens` — price charts, trades, directory |
| **KRC-20 token creation** | ✅ **New** | `/tokens/create` — register new tokens |
| **Escrow memo field** | ✅ **New** | Notes on every escrow |
| **CSV export** | ✅ **New** | Download button on My Escrows |
| **Counter-offers** | ✅ **New** | Inline counter form on offer cards |

## Pages

| Route | Page | Description |
|-------|------|-------------|
| `/` | Dashboard | Stats, quick actions, fee calculator |
| `/offers` | OffersPage | Browse, create, counter offers |
| `/escrows` | EscrowsPage | My escrows, create, lookup, CSV export |
| `/swap` | SwapPage | 6-step atomic swap wizard |
| `/vaults` | VaultsPage | Time-locked vaults |
| `/reputation` | ReputationPage | On-chain scores + vouching |
| `/jury` | JuryPage | Dispute resolution system |
| `/docs` | DocsPage | Developer documentation |
| `/help` | HelpPage | FAQ + quick start guide |
| `/testnet` | TestnetPage | Testnet quickstart with test wallets |
| `/tokens` | TokensPage | KRC-20 token directory + charts |
| `/tokens/create` | CreateTokenPage | Register new tokens |
| `/settings` | SettingsPage | Email notification preferences |

---

*Confidence: 0.95 · Last updated: 7/3/2026*
