# Plan: Escrow-as-a-Service Widget

> **Goal:** Build `<daglock-pay>` — a vanilla JS web component that any website can embed to accept KAS/KRC-20 escrow payments. Like Stripe but for crypto P2P.
>
> **Status:** Plan created. Ready to execute.

---

## Requirements

- [ ] **R1** `<daglock-pay>` web component renders inline on any HTML page
- [ ] **R2** Buyer enters escrow terms (amount, what-for), connects KasWare, locks funds
- [ ] **R3** Seller receives notification + webhook when funds locked
- [ ] **R4** Buyer clicks "Release" after receiving goods
- [ ] **R5** Seller gets paid, fee goes to DagLock treasury
- [ ] **R6** No redirect — fully embedded in the host page

---

## Pre-resolved Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| **Component format** | Vanilla JS Custom Element (no framework) | Works on any site without React/Vue |
| **Size target** | < 50 KB minified + gzipped | Fast load, no impact on host page |
| **API key** | Merchant registers via daglock.com/apps, gets `<script>` tag | Simple onboarding |
| **Styling** | Shadow DOM with CSS variables for customization | Fits any site theme |
| **Wallet** | KasWare browser extension (same as web dashboard) | Users already have it |
| **Fallback** | Deep link to Kaspium mobile deeplink | Covers mobile users |
| **Hosting** | `https://widget.daglock.com/daglock-pay.js` | CDN-hosted, cache-busted via version |

---

## Track A: Web Component `[ ]`

**Timebox:** 1 week

### Phase A1: Core component `[ ]` [3-5 days]
- [ ] Create `web/src/components/daglock-pay.ts`:
  ```typescript
  class DaglockPay extends HTMLElement {
    // Attributes:
    //   amount — KAS amount (e.g., "100")
    //   asset — "KAS" | "KRC20:TOKEN" (optional, default KAS)
    //   seller — seller's Kaspa address
    //   memo — description (e.g., "Widget design")
    //   api-key — merchant's DagLock API key
    //   on-complete — callback function name
    //   theme — "light" | "dark" (default light)
    
    // Renders:
    //   - Payment button ("Pay 100 KAS")
    //   - Iframe-less inline checkout
    //   - Status updates: "Connecting to KasWare..." → "Locking funds..." → "Waiting for release..." → "Complete!"
    //   - Error states
  }
  ```
- [ ] Register as custom element: `customElements.define('daglock-pay', DaglockPay)`
- [ ] Shadow DOM for style isolation
- [ ] CSS variables for customization: `--daglock-primary`, `--daglock-bg`, `--daglock-text`
- [ ] Build with esbuild (already in web project)
- [ ] Bundle: single `.js` file with all dependencies inlined
- ✅ **Checkpoint:** `<daglock-pay amount="10" seller="kaspa:..." api-key="test_key"></daglock-pay>` renders a working payment button on a blank HTML page
- ⚙ **Fallback:** Redirect to `daglock.com/pay/:id` instead of inline component

### Phase A2: Checkout flow `[ ]` [2-3 days]
- [ ] Connect to KasWare: `window.kasware?.requestAccounts()`
- [ ] Create escrow via API: `POST /v1/escrows` with seller address, amount
- [ ] Show funding instructions: "Send X KAS to this address via KasWare"
- [ ] Poll `lock-status` until confirmed (or use WebSocket)
- [ ] Show awaiting-release state
- [ ] Release button: calls `POST /v1/escrows/:id/settle` with KasWare signature
- [ ] Completion callback: fires `on-complete` with `{ escrow_id, tx_id }`
- ✅ **Checkpoint:** Full payment cycle completes without page navigation
- ⚙ **Fallback:** Deep link to web dashboard for KasWare-unsupported browsers

### Phase A3: Script deployment `[ ]` [1-2 days]
- [ ] Build script: `esbuild web/src/components/daglock-pay.ts --bundle --minify --outfile=dist/daglock-pay.js`
- [ ] Host at `https://widget.daglock.com/daglock-pay.js`
- [ ] Version-based cache busting: `daglock-pay.v1.js`
- [ ] SRI hash for security: `<script src="..." integrity="sha384-...">`
- ✅ **Checkpoint:** `curl https://widget.daglock.com/daglock-pay.js` returns minified bundle
- ⚙ **Fallback:** Serve from daglock.com domain

---

## Track B: Merchant API `[ ]`

**Timebox:** 3-5 days

### Phase B1: Payment session API `[ ]` [2-3 days]
- [ ] Create `/home/dillon/_code/DagLock/indexer/src/api/pay.rs`:
  - `POST /v1/pay` — create a checkout session
    - Body: `{ amount, asset, seller_address, memo, redirect_url, webhook_url }`
    - Returns: `{ session_id, escrow_id, checkout_url, expires_at }`
  - `GET /v1/pay/:session_id` — session status
    - Returns: `{ status, escrow, buyer_address }`
- [ ] Merchant API key auth (reuse existing `apps.rs` auth)
- ✅ **Checkpoint:** Can create a payment session and poll for status
- ⚙ **Fallback:** Use existing escrow creation API directly

### Phase B2: Webhook delivery `[ ]` [1-2 days]
- [ ] Read `/home/dillon/_code/DagLock/indexer/src/services/webhooks.rs`
- [ ] Ensure webhooks fire for all widget lifecycle events:
  - `payment.created` — escrow created
  - `payment.confirmed` — funds locked (confirmed on-chain)
  - `payment.completed` — funds released to seller
  - `payment.refunded` — refunded
- [ ] Webhook payload: `{ event, session_id, escrow_id, amount, buyer, seller, signature }`
- ✅ **Checkpoint:** Merchant receives POST to webhook_url for each event
- ⚙ **Fallback:** Merchant polls `GET /v1/pay/:session_id`

### Phase B3: Merchant dashboard `[ ]` [1-2 days]
- [ ] Settings page at `/merchant`:
  - API key management (reuse existing)
  - Webhook URL configuration
  - Payment history table
  - Embed code snippet: copy-paste `<script>` tag
- ✅ **Checkpoint:** Merchant can copy-paste embed code and see payment history
- ⚙ **Fallback:** Static docs page with manual instructions

---

## Track C: Documentation + Demo `[ ]`

**Timebox:** 1-2 days

### Phase C1: Integration guide `[ ]` [1 day]
- [ ] New page at `/docs/widget`
- [ ] "Add escrow payments to your site in 3 steps":
  1. Register at daglock.com/apps → get API key
  2. Add script tag: `<script src="https://widget.daglock.com/daglock-pay.js">`
  3. Add element: `<daglock-pay amount="100" seller="kaspa:..." api-key="YOUR_KEY">`
- [ ] Customization guide (CSS variables, callbacks)
- [ ] Webhook reference (event types, payloads, retries)
- ✅ **Checkpoint:** `/docs/widget` has working copy-paste examples
- ⚙ **Fallback:** README.md in widget directory

### Phase C2: Live demo `[ ]` [1 day]
- [ ] Demo page at `/widget-demo` with a working `<daglock-pay>` instance
- [ ] Amount input field, "Generate demo" button
- [ ] Testnet mode with mock wallets
- ✅ **Checkpoint:** Anyone can visit `/widget-demo` and see a working payment flow
- ⚙ **Fallback:** Static screenshot of the widget

---

## Execution Strategy

```
Priority 1:
  Track A — Web Component (1 week)

Priority 2:
  Track B — Merchant API (3-5 days)

Priority 3:
  Track C — Documentation (1-2 days)
```

---

## Files to Create/Modify

| Track | Files |
|-------|-------|
| A | `web/src/components/daglock-pay.ts` (NEW), `web/esbuild.config.js` (update) |
| B | `indexer/src/api/pay.rs` (NEW), `indexer/src/api/mod.rs` (routes), `indexer/src/services/webhooks.rs` (update), `indexer/src/types.rs` (PaymentSession type) |
| C | `web/src/pages/DocsPage.tsx` (update), `web/src/pages/WidgetDemoPage.tsx` (NEW) |
