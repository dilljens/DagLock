# KRC-20 Post-Toccata Implementation Plan

> **Status:** Toccata activated. KRC-20 tokens exist on mainnet.
> **Current DagLock state:** Token registry API exists, covenant supports KRC-20 escrow, but compile flow is manual, token dropdowns are hardcoded, and no token-specific escrow UI exists.

---

## Track A: Token Creation Flow (Compile + Deploy)

**Goal:** Complete the CreateTokenPage wizard so users can compile and prepare a KRC-20 covenant deployment from the UI.

**Current state:** Steps 2-3 are disabled. The page registers a token in the DB but tells users to "compile separately."

| Phase | What | Effort |
|-------|------|--------|
| A1 | Wire `POST /v1/compile` (daglock_krc20 template) into CreateTokenPage step 2 | 1 day |
| A2 | Show compiled covenant address + bytecode for manual broadcast | ½ day |
| A3 | Add covenant address to PATCH endpoint so users can link deployed covenant | ½ day |

**Files:** `web/src/pages/CreateTokenPage.tsx`, `indexer/src/api/tokens.rs`

---

## Track B: Dynamic Token Dropdowns

**Goal:** Replace hardcoded NACHO/KASPY/GHOST with dynamic population from token registry.

**Current state:** Tokens are hardcoded in `offers.tsx` and `OffersPage.tsx`.

| Phase | What | Effort |
|-------|------|--------|
| B1 | Fetch registered tokens on offer page load | 1 day |
| B2 | Populate asset dropdown from registry + API | ½ day |
| B3 | Fallback to manual ticker input for unregistered tokens | ½ day |

**Files:** `web/src/components/offers.tsx`, `web/src/pages/OffersPage.tsx`, `web/src/api.ts`

---

## Track C: Token-Specific Escrow & Trading

**Goal:** Make the Buy/Sell buttons on TokenDetailPage navigate to token-specific escrow creation.

**Current state:** Both buttons navigate to generic `/escrows` or `/offers` without token context.

| Phase | What | Effort |
|-------|------|--------|
| C1 | Pass ticker as URL param when navigating from token detail | ½ day |
| C2 | Pre-fill asset type in escrow/offer forms from URL param | 1 day |
| C3 | Show token price chart from real settlement data | ½ day |

**Files:** `web/src/pages/TokenDetailPage.tsx`, `web/src/pages/EscrowsPage.tsx`, `web/src/components/escrows.tsx`

---

## Track D: Polish & Remaining Audit Items

**Goal:** Close remaining audit items and polish rough edges.

| Phase | What | Effort |
|-------|------|--------|
| D1 | Onboarding modal (U7) — ensure it shows for first-time visitors on testnet | ½ day |
| D2 | TradeHash newtype usage in indexer (Q4) | 1 day |
| D3 | Mobile layout fixes for offer board, token pages | 1 day |
| D4 | Better error messages on API 429 (rate limit) | ½ day |

---

## Prioritization

```
Priority 1 (This Sprint):
  Track A — Token creation flow (enables token listing)
  Track B — Dynamic dropdowns (makes tokens visible in offers)
  Track D1 — Onboarding modal polish

Priority 2 (Next Sprint):
  Track C — Token-specific trading (better UX for token buyers)
  Track D3 — Mobile fixes

Priority 3 (Future):
  Track D2 — TradeHash newtype usage
  On-chain covenant deployment wizard
  Token metadata editing via PATCH
  KRC-20 escrow creation wizard
```
