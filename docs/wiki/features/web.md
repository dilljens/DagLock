# Web

**Source**: `web/src/`  **Updated**: `2026-06-09`  (14 files)

## What it does
React + Vite dashboard for browser-based users. Provides escrow creation, claiming, offer board, and reputation views. Communicates with the indexer REST API. Uses Vitest + React Testing Library for component tests, Biome for lint.

## Architecture
```
web/src/
  App.tsx                    Main app: layout, routing, tab management (437 lines)
  api.ts                     REST API client + all TypeScript types
  helpers.tsx                Utility functions (money, sompi, time, badge, errMsg)
  ui.tsx                     Reusable UI primitives (Panel, FormField, LookupResult, etc.)
  kasware.ts                 KasWare wallet integration (detect, connect, sign)
  styles.css                 Dark Kaspa green theme
  main.tsx                   React entry point
  vite-env.d.ts              Vite type declarations

  components/
    wallet.tsx               WalletStatus, SignWithWallet
    offers.tsx               CreateOfferForm, OfferCard, MyOffersPanel
    escrows.tsx              CreateEscrowForm, EscrowActionForm, SwapForm,
                             DisputeWithEvidenceForm, EscrowLookup, MyEscrows
    vaults.tsx               CreateVaultForm, VaultLookup, VaultListPanel, VaultStatusPanel
    jury.tsx                 JuryPanel, ResolveDisputePanel, VouchPanel
    identity.tsx             LinkTelegramForm
    lookup.tsx               ReputationLookup, ReceiptLookup
    compile.tsx              CompileCovenantForm

  __tests__/
    setup.ts                 Global mocks (window.kasware, crypto.randomUUID)
    helpers.ts               Shared mockApi factory for test files
    App.test.tsx             Smoke test (App renders)
    CreateOfferForm.test.tsx Offer creation tests
    CreateEscrowForm.test.tsx Escrow creation tests
    EscrowActionForm.test.tsx Settle/cancel/refund/dispute + ConfirmDialog tests
    SwapForm.test.tsx        Atomic swap tests
    DisputeWithEvidenceForm.test.tsx Dispute flow tests
```

## Key functions / components
| Name | Kind | File | Purpose |
|------|------|------|---------|
| `App` | component | `App.tsx` | Main app: layout, tab routing, data loading |
| `api` | module | `api.ts` | REST API client (fetch wrappers + types) |
| `mockApi` | helper | `__tests__/helpers.ts` | Factory for mocked API in tests |
| `errMsg` | helper | `helpers.tsx` | Safely extract error message from unknown |

## Component domains
| File | Components | Domain |
|------|-----------|--------|
| `wallet.tsx` | `WalletStatus`, `SignWithWallet` | KasWare wallet connection + signing |
| `offers.tsx` | `CreateOfferForm`, `OfferCard`, `MyOffersPanel` | Offer lifecycle |
| `escrows.tsx` | `CreateEscrowForm`, `EscrowActionForm`, `SwapForm`, `DisputeWithEvidenceForm`, `EscrowLookup`, `MyEscrows` | Escrow lifecycle |
| `vaults.tsx` | `CreateVaultForm`, `VaultLookup`, `VaultListPanel`, `VaultStatusPanel` | Vault operations |
| `jury.tsx` | `JuryPanel`, `ResolveDisputePanel`, `VouchPanel` | Dispute resolution |
| `identity.tsx` | `LinkTelegramForm` | Telegram verification |
| `lookup.tsx` | `ReputationLookup`, `ReceiptLookup` | Simple lookup panels |
| `compile.tsx` | `CompileCovenantForm` | Dev tool |

## Data flow
1. User navigates to `daglock.io`
2. App fetches escrow/offer data from indexer REST API
3. User creates escrow → unsigned tx assembled (WASM SDK)
4. User signs via KasWare browser extension
5. Signed tx broadcast to network

## Edge cases & gotchas
- Vite dev server on `localhost:5173` — CORS configured on indexer
- KasWare detected via `window.kasware` injection
- No server-side rendering — static SPA
- Destructive actions (cancel, refund, dispute) show `ConfirmDialog` before submitting
- `EscrowActionForm` auth: settle requires `X-Daglock-*` headers, cancel/refund/dispute go through ConfirmDialog flow
- **Jury badge**: `WalletStatus` fires `onConnect(address)` callback. `App` fetches `GET /v1/jury/cases/active/:address` on connect and shows an orange badge count on the Jury button if the user has unread cases

## Testing strategy
| Aspect | Approach |
|--------|----------|
| Framework | Vitest + @testing-library/react + @testing-library/user-event |
| Environment | jsdom (via vitest config) |
| Mocking | `mockApi()` factory in `__tests__/helpers.ts`, `vi.mock("../api")` per test file |
| Global mocks | `window.kasware` mocked in `setup.ts` |
| Coverage | 26 tests across 6 files: App smoke, CreateOffer, CreateEscrow, EscrowAction (settle/cancel/refund/dispute), Swap, Dispute |
| Run | `cd web && npm test` |
| Lint | `cd web && npm run lint` (Biome) |
| Build | `cd web && npm run build` (tsc + vite) |

## Dependencies
| Depends on | For |
|------------|-----|
| React 18 | UI framework |
| Vite 7 | Build tool + dev server |
| Vitest 3 | Test runner |
| @testing-library/react | Component testing |
| @testing-library/user-event | Realistic user interactions |
| @biomejs/biome | Lint + format |
| `daglock-wasm-sdk` | Transaction assembly (future) |
| KasWare | Wallet signing (browser extension) |

## Removed dependencies
| Package | Reason removed |
|---------|---------------|
| `react-router-dom` | Zero imports — app uses hash anchors + tab state instead |
| `tailwindcss` | Zero usage — all styling is custom CSS in `styles.css` |
| `autoprefixer` | Paired with tailwindcss — no longer needed |

## Consumed by
| Consumer | How |
|----------|-----|
| Desktop users | Browser access |
| OTC desks | Premium trading UI |

## Related domains
| Domain | Doc | Relationship |
|--------|-----|--------------|
| wasm-sdk | `features/wasm-sdk.md` | Transaction assembly |
| indexer | `features/indexer.md` | REST API backend |

---

## Audit Findings (2026-06-06)

### High-Priority Usability Issues (Block Real Usage)

| ID | Finding | Location | Fix Required |
|----|---------|----------|--------------|
| **U2** | **Web CreateEscrowForm generates fake `lock_tx_id`** — Uses `crypto.randomUUID()` instead of actual transaction ID from wallet. Indexer accepts it but UTXO won't exist on-chain. | `components/escrows.tsx:72` | Flow: form → WASM compile/assemble → KasWare sign → broadcast → KasWare returns `tx_id` → submit to indexer |
| **U7** | **No web onboarding for first-time users** — "Getting Started" panel exists but no interactive walkthrough. Users don't know they need KasWare + testnet KAS. | `App.tsx:104-126` | First-visit modal: "Need KasWare + testnet KAS + connect wallet." Buttons: Dismiss, Open Faucet. |
| **Q7** | **Web API no request timeout** — `fetch()` calls have no `AbortController`. Stalled requests hang UI indefinitely. | `api.ts` | Wrap fetch with `AbortController` (30s timeout). Add `timeout` option to `loadJson`, `postJson`. |

### Code Quality Issues

| ID | Finding | Impact |
|----|---------|--------|
| **Q1** | `.expect()` on UUID in `escrows.tsx:72` — violates Rule #1 | Replace with proper error handling |
| **Q2/Q3** | Magic number `200` implied in fee calculations — no shared constant | Use shared `FEE_DENOMINATOR` (via WASM SDK or API types) |
| **Q4** | `trade_hash` handling — optional string in `CreateEscrowRequest`, no validation | Use `TradeHash` type from API (validated 64 hex chars) |

### Fix Plan (Phase 2 — Usability + Phase 4 — Polish)

1. **Task 10 (U2):** Web real `lock_tx_id` flow — WASM compile → KasWare sign → broadcast → submit `tx_id`
   - Update `CreateEscrowForm` to use WASM SDK for tx assembly
   - Use `SignWithWallet` component for KasWare signing
   - After broadcast, extract `tx_id` from KasWare response
   - Submit `lock_tx_id` + `lock_tx_output_index` to indexer

2. **Task 15 (U7):** Web onboarding modal — first-visit detection via localStorage

3. **Task 29 (Q7):** Web API request timeout — `AbortController` 30s default

4. **Task 25 (Q1):** Remove `.expect()`/`.unwrap()` in production code

5. **Task 26 (Q2/Q3):** Use shared fee constant (via API response or WASM SDK)

6. **Task 27 (Q4):** `TradeHash` type in `api.ts` with validation

### Dependencies

- **WASM SDK** must export `compile_escrow` + `assemble_unsigned_tx` for browser use
- **KasWare** must support `signTransaction` returning `tx_id` (verify API)
- **Indexer** must have real UTXO verification (S1) and template hash verification (A8)
- **Shared crate** (Phase 0) for fee constant and validation helpers

### KasWare Integration Notes

Current `kasware.ts` has `requestSignTransaction` but the flow in `CreateEscrowForm` doesn't use it properly. The fix requires:

```typescript
// 1. Compile covenant via WASM SDK
const { script, template_hash } = await wasm.compile_daglock(params);

// 2. Assemble unsigned tx (WASM SDK)
const unsigned_tx = await wasm.assemble_create_escrow(script, amount, output_index);

// 3. Sign via KasWare
const signed_tx = await kasware.signTransaction(unsigned_tx);

// 4. Broadcast via KasWare (returns tx_id)
const tx_id = await kasware.broadcastTransaction(signed_tx);

// 5. Submit to indexer
await api.createEscrow({ lock_tx_id: tx_id, lock_tx_output_index: 0, ... });
```

### Verification

- [ ] `cd web && npm test && npm run lint && npm run build` passes
- [ ] Manual: Create escrow via web → KasWare signs → broadcasts → indexer shows `pending_confirmation` → settle → receipt
- [ ] Manual: First visit shows onboarding modal with faucet link
- [ ] Manual: Network stall → API request times out after 30s with friendly error

