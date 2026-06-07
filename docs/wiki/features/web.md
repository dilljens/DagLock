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
