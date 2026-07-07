# Progress: DagLock Usability Sprint

## Baseline (June 29, 2026)
- **Quality signal:** 0.6449 (up from 0.6448)
- **Tests passing:** 77/77 (15 Rust + 40 Web + 22 Bot)
- **Deployed:** Indexer + Bot on Hetzner VPS, Web on Cloudflare Pages

## Session 2026-06-29 — Execution Complete
- **Status:** All high-priority items complete. Ready for pre-mainnet deploy.

### Completed: Track A — Trust Signals
- **A1** Fee calculator: Web `FeeCalculator` component + bot `/fee` command with USD estimates
- **A2** Explorer links: `GET /v1/network/explorer` endpoint, `ExplorerLink` components (TX/address/escrow), integrated into EscrowsPage, OffersPage, ReputationPage
- **A3** Blocklist/report: DB tables (019, 020), API endpoints (`POST/GET/DELETE /v1/blocks`, `POST/GET /v1/reports`), bot commands with web redirect
- **A4** Trade feedback: DB table (021), API endpoints (`POST/GET /v1/escrows/:id/feedback`), bot command with web redirect

### Completed: Track B — Mobile & UX
- **B1** Wallet deep links: `kaspa-deeplink.ts` utility, `kaspa:` URI support
- **B2** Onboarding wizard: 3-slide `OnboardingModal` with localStorage dismiss, "Show tour" in sidebar
- **B3** Help center: `HelpPage` with 10 FAQ accordion items, quick start guide, resource links

### Not Started: Track B4 (Email) & Track C (Covenant Design)
- Email notifications deferred (lowest priority before mainnet)
- Covenant design specs deferred (no code changes before audit)

### Files Changed/Created

#### New files (13):
- `web/src/components/FeeCalculator.tsx` — fee calculator widget
- `web/src/components/OnboardingModal.tsx` — first-visit onboarding
- `web/src/components/ExplorerLink.tsx` — block explorer link components
- `web/src/pages/HelpPage.tsx` — FAQ + quick start help page
- `web/src/kaspa-deeplink.ts` — wallet deep link utilities
- `indexer/src/api/blocks.rs` — blocklist API handlers
- `indexer/src/api/reports.rs` — user report API handlers
- `indexer/src/api/feedback.rs` — trade feedback API handlers
- `indexer/src/db/queries/blocks.rs` — blocklist DB queries
- `indexer/src/db/queries/reports.rs` — report DB queries
- `indexer/src/db/queries/feedback.rs` — feedback DB queries
- `indexer/src/db/migrations/019_create_blocked_users.sql`
- `indexer/src/db/migrations/020_create_reports.sql`
- `indexer/src/db/migrations/021_create_trade_feedback.sql`

#### Modified files (10):
- `web/src/App.tsx` — added HelpPage route, OnboardingModal
- `web/src/api.ts` — added networkPrice(), explorer() methods
- `web/src/router.tsx` — added /help route
- `web/src/layout/Sidebar.tsx` — added Help nav, Show tour button
- `web/src/pages/Dashboard.tsx` — added FeeCalculator
- `web/src/pages/EscrowsPage.tsx` — added FeeCalculator, ExplorerLinks to create + lookup
- `web/src/pages/OffersPage.tsx` — added ExplorerLinks
- `web/src/pages/ReputationPage.tsx` — added ExplorerLinks
- `web/src/styles.css` — ~249 lines of new CSS
- `bot/src/index.js` — added /fee, /block, /feedback commands + help text
- `bot/src/lib/api.js` — added fee, block, report, feedback API methods
- `indexer/src/main.rs` — added EXPLORER_BASE_URL env var support
- `indexer/src/api/mod.rs` — added blocks, reports, feedback modules + routes
- `indexer/src/api/network.rs` — added /v1/network/explorer endpoint
- `indexer/src/db/schema.rs` — added 3 new migrations
- `indexer/src/db/queries/mod.rs` — added blocks, reports, feedback modules

### Verification
- Rust indexer: 15 tests ✅
- Web UI: 40 tests ✅ (7 test files)
- Bot: 22 tests ✅
- TypeScript: type check ✅
- Vite build: production build ✅
- Sentrux quality: 0.6449 (stable)
