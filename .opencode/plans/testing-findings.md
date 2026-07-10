# Findings: E2E Testing Improvement

## Discovery Answers
- **Goal:** Add comprehensive E2E tests for wallet, offers, escrows
- **Tracks:** A) Wallet errors, B) Offer lifecycle, C) Escrow actions, D) CI
- **Wallet mock:** addInitScript (no real KasWare) — already implemented
- **Test runner:** Playwright v1.61 — already configured
- **CI:** GitHub Actions — needs new job

## Existing E2E Infrastructure
- `web/playwright.config.ts` — 3 projects (manual-wallet, kasware-wallet, mobile)
- `web/e2e/fixtures.ts` — auto mocks KasWare + API + dismisses onboarding
- `web/e2e/helpers/kasware.ts` — mock KasWare provider factory + injector
- `web/e2e/helpers/api.ts` — route mocks for all major endpoints
- Existing tests: 8 wallet, 5 escrow-create, 3 error-state = 16 tests

## Key Selectors for E2E Tests
- Connect button: `.sidebar-connect`
- Wallet info: `.sidebar-wallet`
- Address display: `.sidebar-wallet-addr`
- Network badge: `.sidebar-network`
- Sidebar links: `.sidebar-link` with text filter
- Tab bar: `.tab-bar`
- Offer cards: `.offer`
- Toast notifications: `.Toastify__toast` or custom toast selector
