# Plan: Embeddable Escrow Widget (P2)

**Goal:** Turn DagLock into escrow infrastructure. Offer a `<daglock-escrow>` web component that any website, marketplace, or Telegram group can embed. Buyer clicks "Pay with DagLock" → widget creates escrow → seller ships → buyer confirms → funds release.

**Effort:** 2-3 weeks

**Why this matters:** Escrow as a destination product requires users to come to daglock.com. Escrow as infrastructure goes where the users already are — marketplaces, Discord servers, e-commerce stores, OTC desks. This is how Stripe grew: not by being a payment page, but by being an embeddable checkout.

---

## Architecture

```
Any Website
  └── <daglock-escrow amount="100" asset="KAS" seller="kaspa:...">
        │
        ├── Web Component (vanilla JS, no framework needed)
        ├── Creates escrow via DagLock REST API
        ├── Buyer connects KasWare/Kaspium
        ├── Funds locked in covenant
        ├── Seller notified via webhook
        └── Both parties can settle/refund/dispute via widget
```

---

## Phase A: Web Component `[ ]`
**⏱ Timebox:** 1 week

- [ ] Create `web/src/components/daglock-escrow.js` — vanilla JS custom element
  ```javascript
  class DaglockEscrow extends HTMLElement {
      connectedCallback() {
          const amount = this.getAttribute('amount');
          const asset = this.getAttribute('asset') || 'KAS';
          const seller = this.getAttribute('seller');
          const apiUrl = this.getAttribute('api-url') || 'https://api.daglock.com';
          // Render inline checkout UI
      }
  }
  customElements.define('daglock-escrow', DaglockEscrow);
  ```
- [ ] Attributes:
  - `amount` (required) — KAS amount
  - `asset` (optional, default "KAS") — asset type
  - `seller` (required) — seller's Kaspa address
  - `api-url` (optional) — custom API endpoint
  - `theme` (optional) — "light" | "dark" | "auto"
  - `lang` (optional) — localization
- [ ] Slots/events:
  - `oncreate(escrowId)` — called when escrow is created
  - `onsettle(receipt)` — called when escrow settles
  - `onerror(message)` — called on errors
- [ ] States: `idle` → `connecting-wallet` → `creating-escrow` → `funds-locked` → `waiting-for-seller` → `completed` / `refunded`
- [ ] Built-in KasWare detection + connect button
- [ ] Mobile responsive (single column, large tap targets)
- [ ] i18n support for English + Chinese (major Kaspa community languages)

**✅ Checkpoint:** `npm run build` produces `daglock-escrow.js` that can be dropped into any HTML page.

---

## Phase B: Widget CDN + docs `[ ]`
**⏱ Timebox:** 2 days

- [ ] Host bundled widget on `https://cdn.daglock.com/widget/daglock-escrow.js`
- [ ] Cloudflare Pages routing: `cdn.daglock.com/*` → serves from R2 or Pages
- [ ] Create `docs/widget.md` with:
  - Quick start: copy-paste snippet
  - Full API reference
  - Theme customization
  - Event handling
  - Security notes
- [ ] Demo page at `https://daglock.com/widget/demo` — interactive sandbox
- [ ] Example integrations:
  - Plain HTML page (paste this code)
  - React example (`useEffect` + ref)
  - Vue example

**Quick start snippet:**
```html
<script src="https://cdn.daglock.com/widget/daglock-escrow.js"></script>
<daglock-escrow amount="100" asset="KAS" seller="kaspa:q..."></daglock-escrow>
```

**✅ Checkpoint:** Copy-paste snippet works on a blank HTML page.

---

## Phase C: Webhook-enforced lifecycle `[ ]`
**⏱ Timebox:** 3 days

- [ ] Widget polls escrow status every 15 seconds (already have `GET /v1/escrows/:id`)
- [ ] Widget uses WebSocket for real-time updates when available
- [ ] Webhook callbacks:
  - `escrow.created` → seller gets notification
  - `escrow.settled` → widget shows success, calls `onsettle`
  - `escrow.disputed` → widget shows dispute UI
  - `escrow.refunded` → widget shows refund status
- [ ] Seller webhook: POST to a URL the merchant provides (configured via dashboard)
- [ ] Widget shows persistent status even after page refresh (reads escrow ID from URL param or localStorage)

**✅ Checkpoint:** Widget survives page refresh — rehydrates from escrow ID in URL.

---

## Phase D: Merchant dashboard `[ ]`
**⏱ Timebox:** 1 week

- [ ] Merchant registration at `https://daglock.com/merchant`
  - Create API key for webhook signing
  - Set webhook URL for settlement notifications
  - Configure branding (logo, colors, merchant name)
- [ ] Merchant dashboard:
  - List of all escrows created via your widget
  - Pending, settled, disputed counts
  - Total volume in KAS
  - Settlement history with receipts
- [ ] Webhook signing (HMAC-SHA256) so merchants can verify callbacks are genuine
- [ ] Rate limits: 100 escrows/day for free tier, unlimited for API key holders

**✅ Checkpoint:** Merchant sees their widget-created escrows in a dashboard, receives signed webhooks.

---

## Phase E: Tests + security audit `[ ]`
**⏱ Timebox:** 2 days

- [ ] Unit tests for widget state machine
- [ ] E2E: load widget → connect mock wallet → create escrow → settle
- [ ] Security review:
  - Widget does not have access to user's private keys (KasWare handles signing)
  - Widget does not store any credentials
  - XSS prevention: all attributes sanitized
  - CSP compliance: widget works on sites with strict Content Security Policy
- [ ] Test on: Chrome, Firefox, Safari, Edge, mobile Chrome, mobile Safari

**✅ Checkpoint:** All tests pass, security review passes.

---

## Files Changed / Created

| File | Change |
|------|--------|
| `web/src/components/daglock-escrow.js` | **New** — vanilla JS custom element |
| `web/package.json` | Add build step for widget bundle |
| `vite.config.ts` | Add multi-page build (app + widget) |
| `docs/widget.md` | **New** — full widget documentation |
| `indexer/src/api/apps.rs` | Extend for merchant config |
| `indexer/src/api/webhooks.rs` | Already exists — widget uses it |
| `web/src/pages/MerchantDashboard.tsx` | **New** — merchant dashboard page |
| `web/src/App.tsx` | Add `/merchant` route |
| `web/src/router.tsx` | Add `/merchant` route type |
| `web/src/layout/Sidebar.tsx` | Add merchant link (?) |

## Edge Cases

| Case | Handling |
|------|----------|
| User refreshes page mid-flow | Widget reads escrow ID from URL/ls, rehydrates |
| Seller is offline | Widget shows "Awaiting seller confirmation" with last-seen timestamp |
| KasWare not installed | Show install prompt + manual mode fallback |
| Merchant disables widget | API returns 403, widget shows "This merchant is no longer accepting escrows" |
| Network timeout during creation | Retry 3x with exponential backoff, then show error with retry button |
| Browser doesn't support Web Components | Show polyfill notice + link to checkout page |
