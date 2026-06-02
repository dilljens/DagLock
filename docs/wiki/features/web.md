# Web

**Source**: `web/src/`  **Updated**: `2026-06-02`  (4 files)

## What it does
React + Vite dashboard for browser-based users. Provides escrow creation, claiming, offer board, and reputation views. Communicates with the indexer REST API.

## Architecture
```
App.tsx (routing)
    │
    ├── api.ts          ──▶ indexer REST API (HTTP)
    └── main.tsx        ──▶ React entry point
```

## Key functions / components
| Name | Kind | File:Line | Purpose |
|------|------|-----------|---------|
| `App` | component | `web/src/App.tsx` | Main app with routing |
| `api.ts` | module | `web/src/api.ts` | REST API client functions |
| `main.tsx` | entry | `web/src/main.tsx` | React DOM render |

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

## Testing strategy
| Aspect | Approach |
|--------|----------|
| Unit tests | Component tests (Vitest) |
| E2E tests | Future (Playwright) |
| Run command | `cd web && npm test` |

## Dependencies
| Depends on | For |
|------------|-----|
| React | UI framework |
| Vite | Build tool |
| Tailwind CSS | Styling |
| `daglock-wasm-sdk` | Transaction assembly |
| KasWare | Wallet signing |

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
