# web

React + Vite dashboard. 17+ pages. Communicates with indexer REST API. Vitest + RTL tests, Biome lint.

## Pages

| Route | Page | Description |
|-------|------|-------------|
| `/` | Dashboard | Hero, 12 feature cards, 7 action cards, fee calculator |
| `/offers` | OffersPage | Browse, create, counter offers with deal type presets |
| `/escrows` | EscrowsPage | Escrows, milestones, multi-party, deposits, invoices, create, lookup, CSV |
| `/swap` | SwapPage | 6-step atomic swap wizard + deep link `/swap/:id` + QR code |
| `/vaults` | VaultsPage | Time-locked vaults with check-in, inheritance, early-exit |
| `/subscriptions` | SubscriptionsPage | Create, list, draw recurring subscriptions |
| `/tokens` | TokensPage | KRC-20 token directory, charts, create |
| `/reputation` | ReputationPage | On-chain scores + vouching |
| `/jury` | JuryPage | Community jury + AI mediation + evidence reveal |
| `/stats` | StatsPage | Analytics dashboard with daily volume charts |
| `/security` | SecurityPage | Covenant Security Analysis — 6 interactive attack scenarios |
| `/merchant` | MerchantPage | Escrow widget embed, API key management, webhook config |
| `/blog` | BlogPage, BlogPost | 4 blog posts (KRC-20, AI mediation, feature set, SilverScript) |
| `/docs` | DocsPage | Developer documentation, FAQ, OpenAPI spec |
| `/help` | HelpPage | FAQ + quick start guide |
| `/testnet` | TestnetPage | Testnet quickstart with test wallets |
| `/settings` | SettingsPage | Email notifications, price alerts, chat key management |

## Key Components

| Component | File | Purpose |
|-----------|------|---------|
| ChatPanel | `components/ChatPanel.tsx` | E2E encrypted messaging with key exchange + anchoring |
| MediationPanel | `components/MediationPanel.tsx` | AI + jury dispute resolution UI |
| daglock-pay | `components/daglock-pay.ts` | Vanilla JS web component for embedded payments |
| AtomicSwapWizard | `components/AtomicSwapWizard.tsx` | 6-step guided swap flow with secret safety |
| PriceChart | `components/PriceChart.tsx` | SVG line chart for KAS/USD history |
| PriceAlerts | `components/PriceAlerts.tsx` | Alert create/list/delete UI |
| FeeCalculator | `components/FeeCalculator.tsx` | Escrow fee estimator |

## Crypto Modules

| Module | File | Purpose |
|--------|------|---------|
| chat-crypto | `crypto/chat-crypto.ts` | Ed25519 keys, X25519 ECDH, NaCl secretbox encrypt/decrypt |
| chat-store | `crypto/chat-store.ts` | localStorage persistence of per-escrow keypairs |
| recovery-sheet | `crypto/recovery-sheet.ts` | .txt download + restore for chat private keys |

---
*Confidence: 0.95 · Last updated: 7/7/2026*
