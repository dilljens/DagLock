# UI/UX Improvement Plan

**Status:** Active
**Started:** 2026-06-17
**Target:** Production-ready web UI for mainnet launch

---

## Phase 0: Dependencies & Setup

- [x] Install new npm packages (motion, @tanstack/react-query, react-hook-form, zod, @hookform/resolvers, @radix-ui/*)

## Phase 1: Visual Polish (High impact, low effort)

- [x] 1.1 — Skeleton loading states for all data types (CSS + components + all pages)
- [x] 1.2 — Page transition animations with Framer Motion (AnimatePresence in App.tsx)
- [ ] 1.3 — Transaction confirmation celebration animation
- [x] 1.4 — Consistent number formatting (moneyCompact, formatKas, locale-aware)
- [x] 1.5 — Design token system (CSS variables: colors, spacing, typography, shadows)
- [x] 1.3 — Enhanced toast animations (motion AnimatePresence + icon ring)

## Phase 2: Accessibility & Interaction

- [x] 2.1 — Radix Dialog (replace custom ConfirmDialog)
- [x] 2.2 — Radix Tabs (CSS + provider ready; needs page migration)
- [x] 2.3 — Radix Tooltip (Provider in App root)
- [x] 2.4 — Error boundaries for each page
- [x] 2.5 — Focus management and keyboard navigation (built into Radix)

## Phase 3: Data & Performance UX

- [x] 3.1 — TanStack Query (health/stats/offers useQuery + QueryClientProvider)
- [x] 3.2 — React Hook Form + Zod (shared schemas, VaultForm migration; remaining forms use manual validation)
- [x] 3.3 — WebSocket real-time updates (useWebSocket hook, auto-reconnect, query invalidation)
- [ ] 3.4 — Loading/error/success state components

## Phase 4: Design System & Consistency

- [ ] 4.1 — CSS modules refactor (split styles.css)
- [x] 4.2 — Better empty states (EmptyState component, dashboard/offers/vaults updated)
- [ ] 4.3 — Onboarding wizard for first-time users
- [ ] 4.4 — Mobile responsiveness refinements
