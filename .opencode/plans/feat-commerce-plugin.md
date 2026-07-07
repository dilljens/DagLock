# Plan: Commerce Plugin (P6)

**Goal:** Let merchants accept KAS and KRC-20 payments via DagLock escrow on WooCommerce and Shopify. Buyer pays with escrow protection → merchant ships → buyer confirms → funds release. Turns DagLock into a Stripe-like payment processor for crypto-native e-commerce.

**Effort:** 2 weeks

**Why this matters:** Escrow for goods is the original use case (Escrow.com built a $4.5B business on it). Every WooCommerce/Shopify store that accepts crypto today uses raw address payments — no escrow protection. A plugin adds escrow protection at checkout, creating a differentiated payment method for crypto merchants.

---

## Architecture

```
Customer checks out
  └── Selects "Pay with DagLock" at checkout
       └── DagLock widget/redirect creates escrow
            └── Merchant notified (webhook)
                 └── Merchant ships goods
                      └── Customer confirms receipt
                           └── Escrow settles → merchant gets paid
```

---

## Phase A: WooCommerce plugin `[ ]`
**⏱ Timebox:** 1 week

- [ ] Create `plugins/woocommerce/daglock-escrow.php`:
  - WordPress plugin header with metadata
  - Payment gateway class extending `WC_Payment_Gateway`
- [ ] Admin settings page:
  - DagLock API key (from `daglock.com/merchant`)
  - Merchant Kaspa address (where payments go)
  - Escrow timeout in days (default 7)
  - Dispute mode (standard / jury / mediator)
  - Debug mode toggle
- [ ] Checkout flow:
  - Customer selects "DagLock Escrow (KAS/KRC-20)" at checkout
  - Redirects to DagLock checkout page (or embedded widget)
  - Customer connects wallet, pays into escrow covenant
  - On escrow created → webhook → WooCommerce order marked as "On Hold"
  - On escrow settled → WooCommerce order marked as "Processing"
  - On escrow refunded → WooCommerce order marked as "Cancelled"
- [ ] Order details page:
  - Show escrow ID, status, explorer link
  - Manual settle button (merchant confirms shipment, requests settlement)
  - Manual refund button
- [ ] Currency support: KAS (native), KRC-20 tokens (configurable per store)

**✅ Checkpoint:** WooCommerce admin installs plugin, configures API key, completes a test purchase end-to-end.

---

## Phase B: Shopify app `[ ]`
**⏱ Timebox:** 1 week

- [ ] Create `plugins/shopify/` — Shopify app using the REST Admin API + DagLock API
- [ ] Shopify app registration:
  - Uses Shopify OAuth for merchant authentication
  - Creates DagLock API key automatically for each merchant
- [ ] Checkout extension:
  - Uses Shopify's Checkout UI extension (cart `daglock-escrow` widget)
  - Or redirect checkout: order → external DagLock page → return to Shopify
- [ ] Order webhook handling:
  - `escrow.created` → update Shopify order status
  - `escrow.settled` → fulfill order
  - `escrow.disputed` → flag order for review
- [ ] Shopify admin panel:
  - Escrow transaction list
  - Manual escrow actions (settle/refund)
  - Payment history

**✅ Checkpoint:** Shopify store offers "Pay with DagLock Escrow" at checkout → payment flows through escrow → order fulfills on settlement.

---

## Phase C: Shared widget integration `[ ]`
**⏱ Timebox:** 2 days

- [ ] Both plugins use the same `<daglock-escrow>` web component from the embeddable widget plan
- [ ] Widget parameters for commerce:
  - `mode="checkout"` — shows price, item description, merchant name
  - `merchant-name` — displayed in the widget header
  - `order-id` — linked back to the e-commerce order
  - `return-url` — where to redirect after completion
- [ ] If widget is not yet built, fallback to hosted checkout page:
  - `https://daglock.com/pay/{order_id}` — public payment page (similar to existing invoice flow)
- [ ] Payment page shows: merchant name, item, amount, KasWare/Kaspium connect button

**✅ Checkpoint:** Customer clicks "Pay with DagLock" → widget/checkout page loads → payment completed → returns to store.

---

## Phase D: Documentation + marketplace `[ ]`
**⏱ Timebox:** 2 days

- [ ] WooCommerce plugin published to WordPress plugin directory
- [ ] Shopify app listed on Shopify App Store
- [ ] Docs:
  - `docs/commerce/woocommerce.md` — setup guide
  - `docs/commerce/shopify.md` — setup guide
  - `docs/commerce/self-hosted.md` — generic webhook-based integration
- [ ] Marketing page at `https://daglock.com/commerce`
  - "Accept KAS payments with escrow protection"
  - Feature comparison vs raw crypto payments
  - Fee comparison vs credit cards (2.9% + $0.30) vs DagLock (0.5%)

**✅ Checkpoint:** Follow the docs to set up a WooCommerce store in 10 minutes.

---

## Phase E: Tests `[ ]`
**⏱ Timebox:** 2 days

- [ ] WooCommerce: unit tests for payment gateway class
- [ ] WooCommerce: mock webhook test → order status transitions correctly
- [ ] Shopify: app installation OAuth flow
- [ ] Shopify: checkout redirect → escrow creation → webhook → fulfillment
- [ ] E2E: full purchase lifecycle with mock wallet

**✅ Checkpoint:** All tests pass.

---

## Files Changed / Created

| File | Change |
|------|--------|
| `plugins/woocommerce/daglock-escrow.php` | **New** — full WooCommerce plugin |
| `plugins/woocommerce/readme.txt` | **New** — WordPress plugin readme |
| `plugins/shopify/app.js` | **New** — Shopify app backend |
| `plugins/shopify/extension/` | **New** — checkout UI extension |
| `plugins/shopify/README.md` | **New** — Shopify app docs |
| `docs/commerce/woocommerce.md` | **New** |
| `docs/commerce/shopify.md` | **New** |
| `docs/commerce/self-hosted.md` | **New** |
| `web/src/pages/PayInvoicePage.tsx` | Extend to support merchant checkout flow |
| `indexer/src/api/webhooks.rs` | Already exists — commerce uses same webhook system |

## Fee Model for Commerce

| Party | Fee |
|-------|-----|
| Buyer | 0.5% protocol fee (enforced by covenant) |
| Merchant | 0% — no additional DagLock fee |
| Credit card alternative | 2.9% + $0.30 — DagLock saves merchants ~2.4% |

Selling point: "Accept KAS payments for **0.5% total fee** instead of 2.9% + $0.30 for credit cards. And your customers get escrow protection for free."

## Edge Cases

| Case | Handling |
|------|----------|
| Customer never confirms receipt | Timeout triggers refund to customer. Merchant must dispute. |
| Merchant never ships | Customer disputes escrow after timeout. Jury decides. |
| Partial shipment | Full escrow settles only. No partial release covenant. |
| Plugin deactivated mid-order | Order stays as-is. Funds are in covenant — no one loses money. |
| API key compromised | Merchant revokes key in DagLock dashboard. All active escrows remain secure (covenant-enforced). |
| Customer pays wrong amount | Covenant enforces exact amount. Payment fails if mismatch. |
