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

---
*Confidence: 0.95 · Last updated: 6/17/2026*