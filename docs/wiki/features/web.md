# Web

**Source**: `web/src/`  **Updated**: `2026-06-05`  (3 files)

## What it does
React + Vite dashboard for browser-based users. Provides escrow creation, claiming, offer board, and reputation views. Communicates with the indexer REST API.

## Architecture
```
App.tsx (all components)
    │
    ├── api.ts              ──▶ indexer REST API (HTTP)
    ├── styles.css           ──▶ Dark Kaspa green theme
    └── main.tsx             ──▶ React entry point

Components:
    CreateOfferForm         Post a buy/sell listing with expiry
    CreateEscrowForm        Create escrow (standard/mediator/jury) with atomic swap + market price
    EscrowActionForm        Settle, refund, cancel with KasWare signing
    SwapForm                Atomic swap settle via preimage
    DisputeWithEvidenceForm Dispute with evidence + jury mode
    OfferCard               Offer listing with accept/cancel inline
    LookupResult            Reusable fetch-and-display component
    EscrowLookup            Escrow detail with timeline + messages + evidence
    ReputationLookup        Beta score breakdown + vouch scores + wash trading signal
    MyEscrows               List escrows by address
    MyOffersPanel           List offers by address with status filter
    ReceiptLookup           Settlement receipt viewer
    JuryPanel               Register, unregister, vote on cases
    LinkTelegramForm        Verify Telegram handle
    CreateVaultForm         Create time-locked vault
    VaultLookup             Vault details by ID
    VaultListPanel          List vaults by owner address
    VaultStatusPanel        Withdraw from vault (prompts for address + signature)
    WalletStatus            KasWare wallet connection + balance
    SignWithWallet          Schnorr message signing via KasWare
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
