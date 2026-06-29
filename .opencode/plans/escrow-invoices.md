# Escrow-Based Invoices

**Goal:** Let freelancers create shareable invoice links that clients can pay via escrow — turning DagLock into a Kaspa-native invoicing platform.

**Design principle:** Invoice is a lightweight metadata wrapper around the existing escrow covenant. Zero covenant changes.

---

## Requirements

- [ ] R1: Freelancer creates an invoice (amount, description, client email/handle, due date) via web or Telegram
- [ ] R2: Invoice gets a shareable link (`daglock.com/pay/INV-xxx`) with rich OG preview for Telegram/Discord
- [ ] R3: Client opens link — sees a professional invoice page with a "Pay with KasWare" button (no wallet required to view)
- [ ] R4: Client pays → invoice-linked escrow created → freelancer notified via Telegram/webhook
- [ ] R5: Standard escrow settlement or refund flow — no covenant changes
- [ ] R6: Settlement receipt links back to the invoice

---

## Pre-resolved Decisions

- **No covenant changes** — invoice is metadata stored in the indexer DB, referenced by escrow
- **Invoice status lifecycle**: `draft → sent → paid → settled / disputed / refunded`
- **Separate table**: `invoices` table, not bloating the existing `escrows` table
- **Auth**: Creating an invoice requires signature (proves freelancer owns the address). Viewing is public (via the `/pay/:id` link).
- **OG tags**: The `/pay/:id` page renders server-side OG meta tags for rich link previews in Telegram, Discord, Twitter

---

## Track A: Indexer — Invoice Data Layer `[ ]`

### Phase A1: Schema + queries `[ ]`
- [ ] Create `018_create_invoices.sql` migration:
  ```sql
  CREATE TABLE IF NOT EXISTS invoices (
      id TEXT PRIMARY KEY,
      freelancer_address TEXT NOT NULL,
      client_address TEXT,
      escrow_id TEXT REFERENCES escrows(id),
      description TEXT NOT NULL,
      amount_sompi INTEGER NOT NULL,
      due_date INTEGER,
      status TEXT NOT NULL DEFAULT 'draft',
      created_at INTEGER NOT NULL,
      paid_at INTEGER,
      settled_at INTEGER
  );
  ```
- [ ] Create `indexer/src/db/queries/invoices.rs` with:
  - `insert_invoice()`
  - `get_invoice()`
  - `list_invoices_by_freelancer()`
  - `update_invoice_status()`
  - `link_invoice_to_escrow()`
- [ ] Register module in `db/queries/mod.rs`
- ✅ Checkpoint: `cargo test` passes new query tests
- ⚙ Fallback: Use JSON field on escrows table if migration issues arise (simpler, less queryable)

### Phase A2: API endpoints `[ ]`
- [ ] `POST /v1/invoices` — create invoice (requires auth headers)
  - Body: `{ description, amount_sompi, due_date?, client_email? }`
  - Returns: `{ id: "INV-xxx", link: "https://daglock.com/pay/INV-xxx" }`
- [ ] `GET /v1/invoices/:id` — public invoice details (no auth)
  - Returns invoice metadata + status + linked escrow status
- [ ] `GET /v1/invoices?address=...` — list invoices for a freelancer (requires auth matching address)
- [ ] Register routes in `api/mod.rs`
- ✅ Checkpoint: `curl /v1/invoices` CRUD works
- ⚙ Fallback: Start with create + get only, defer listing

### Phase A3: Invoice → Escrow bridge `[ ]`
- [ ] Modify escrow creation to accept optional `invoice_id` parameter
- [ ] After escrow created, link invoice: `UPDATE invoices SET status='paid', escrow_id=?, client_address=?`
- [ ] After escrow settled: `UPDATE invoices SET status='settled', settled_at=?`
- [ ] After escrow refunded: `UPDATE invoices SET status='refunded'`
- [ ] After escrow disputed: `UPDATE invoices SET status='disputed'`
- ✅ Checkpoint: Creating an escrow with `invoice_id` transitions invoice status automatically
- ⚙ Fallback: Manual status updates via API (less automated but functional)

---

## Track B: Web UI — Invoice Pages `[ ]`

### Phase B1: Invoice creation form `[ ]`
- [ ] Add "Invoice" tab to existing EscrowsPage or a new `/invoices` route
- [ ] Form: description, amount, due date (optional), client email (optional)
- [ ] On submit: calls `POST /v1/invoices` → displays shareable link + copy button
- [ ] List existing invoices with status badges
- ✅ Checkpoint: User can create an invoice and copy the link
- ⚙ Fallback: Keep it simple — just a form on the EscrowsPage "Create" tab with a toggle for invoice mode

### Phase B2: Public payment page (`/pay/:id`) `[ ]`
- [ ] Create `InvoicesPage.tsx` or add `/pay/:id` route to the existing router
- [ ] Route is public (no wallet required to view)
- [ ] Renders:
  - Invoice header (INVOICE #INV-xxx)
  - Freelancer address (shortened) + optional Telegram handle if on-chain identity exists
  - Description
  - Amount in KAS (with KAS/USD price if available)
  - Due date
  - Status badge
  - "Connect Wallet to Pay" button OR "Paid" / "Settled" state
- [ ] On payment: creates escrow with `invoice_id` linked
- ✅ Checkpoint: `curl https://daglock.com/pay/INV-test` renders invoice page
- ⚙ Fallback: Show invoice details without payment button for now — use existing escrow creation flow

### Phase B3: Rich OG link previews `[ ]`
- [ ] The `/pay/:id` page must render OG meta tags in the server-rendered HTML
- [ ] Tags: `og:title="Invoice from DagLock"`, `og:description="500 KAS — Website redesign"`, `og:image` (invoice graphic)
- [ ] Uses same pattern as existing DocsPage SEO — inject into `<Helmet>`
- ✅ Checkpoint: Pasting invoice link in Telegram shows a rich card preview
- ⚙ Fallback: Generic OG tags without invoice-specific data

---

## Track C: Telegram — Invoice Commands `[ ]`

### Phase C1: `/invoice` command `[ ]`
- [ ] New `/invoice` command: wizard-style (like `/create`)
  - Step 1: "Enter amount in KAS"
  - Step 2: "Describe the work"
  - Step 3: "Due date? (optional, days from now)"
  - Step 4: "Client Telegram username (optional)"
- [ ] On completion: sends invoice link + instructions to share it
- ✅ Checkpoint: `/invoice` in Telegram creates an invoice and returns a link
- ⚙ Fallback: `/invoice` just generates a link to the web invoice creation form

---

## Priority & Timebox

| Phase | Feature | Timebox | Depends On |
|-------|---------|---------|------------|
| A1 | DB schema + queries | 2 hr | Nothing |
| A2 | API endpoints | 2 hr | A1 |
| A3 | Invoice → escrow bridge | 2 hr | A2 |
| B1 | Invoice creation form | 3 hr | A2 |
| B2 | Public payment page | 4 hr | A3 |
| B3 | OG link previews | 1 hr | B2 |
| C1 | Telegram `/invoice` | 2 hr | A2 |
| **Total** | | **~16 hr** | |

---

## Future Iterations (Not in Scope)

- Email notification when invoice is created / paid / settled
- Recurring invoices (subscription escrows — requires covenant changes)
- Invoice template system (save and reuse terms)
- Multi-currency invoices (KAS + KRC-20)
- Discount codes / promo fees
