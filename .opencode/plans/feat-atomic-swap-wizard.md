# Plan: Atomic Swap Wizard UI

> **Goal:** Build a guided step-by-step atomic swap wizard UI. The covenant already supports `swap(secret)`. The backend already generates secrets and hashes (`POST /v1/swap/generate`). What's missing is a polished user-facing wizard.
>
> **Status:** Plan created. Ready to execute.

---

## Requirements

- [ ] **R1** Step-by-step swap flow: Generate → Share → Wait → Reveal → Settle
- [ ] **R2** Counterparty discovery: deep link or QR code to share swap invite
- [ ] **R3** Timer/countdown showing when timeout refund becomes available
- [ ] **R4** Swap history page
- [ ] **R5** Receipt generation after successful swap

---

## Pre-resolved Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| **Pages** | New `/swap` page (exists as stub) + wizard component | Don't clutter escrows page |
| **Wizard steps** | 6 steps (1: Init, 2: Generate, 3: Share, 4: Wait, 5: Claim, 6: Complete) | Matches existing AtomicSwapWizard.tsx |
| **Deep link** | `https://daglock.com/swap/:id` — opens the wizard pre-filled | Copy-paste shareable |
| **Timer** | Client-side countdown from `timeout` field | No server polling needed |
| **Bot** | `/swap` command to initiate + notifications | Existing bot notification pattern |
| **Secret safety** | Big red warning: "SAVE THIS — only chance to claim" | Prevents loss of funds |

---

## Track A: Wizard Component `[ ]`

**Timebox:** 3-5 days

### Phase A1: Refactor existing wizard `[ ]` [2-3 days]
- [ ] Read `/home/dillon/_code/DagLock/web/src/components/AtomicSwapWizard.tsx` — understand current 6-step flow
- [ ] Add missing polish:
  1. **Init**: Amount, counterparty address, timeout selector (1h/6h/24h/48h/7d with sensible defaults)
  2. **Generate**: Shows secret (big warning!) + hash. "I've saved my secret" checkbox to proceed
  3. **Share**: Deep link copy button + QR code. Instructions: "Send this link to your counterparty"
  4. **Wait**: Shows countdown to refund. Polls escrow status. Cancel button
  5. **Claim**: Counterparty enters preimage. "Preimage revealed!" success state. Auto-settle confirm
  6. **Done**: Receipt link. Share result. "Start another swap" button
- [ ] Add countdown timer display on all steps after creation
- [ ] Add secret reveal animation (redacted until explicitly clicked)
- ✅ **Checkpoint:** Full 6-step flow works end-to-end on testnet
- ⚙ **Fallback:** Keep existing basic wizard, add steps incrementally

### Phase A2: Deep linking `[ ]` [1 day]
- [ ] `https://daglock.com/swap/:id` route prefills wizard from escrow data
- [ ] QR code generation for the deep link (use existing QR library or simple `<canvas>`)
- [ ] Deep link also works in Telegram: `t.me/DagLock_bot?start=swap_<id>`
- ✅ **Checkpoint:** Deep link opens wizard with amount + counterparty pre-filled
- ⚙ **Fallback:** Manual copy-paste of escrow ID

### Phase A3: Swap history `[ ]` [1 day]
- [ ] New section on escrows page: "My Swaps" — filters escrows by `trade_hash != null`
- [ ] Shows status: "Waiting for counterparty" / "Complete" / "Refunded" / "Expired"
- [ ] Receipt button for completed swaps
- ✅ **Checkpoint:** "My Swaps" tab shows all atomic swaps with status + receipt
- ⚙ **Fallback:** Filter escrows list manually

---

## Track B: Bot Swap Support `[ ]`

**Timebox:** 2-3 days

### Phase B1: `/swap` command `[ ]` [1-2 days]
- [ ] `/swap create <amount> <counterparty>` — creates escrow with random trade_hash
- [ ] Shows secret: "🔑 SECRET: ... — save this! You'll need it to claim."
- [ ] Shows shareable link
- [ ] `/swap claim <escrow_id> <preimage>` — claims swap
- [ ] Auto-generates secret if not provided
- ✅ **Checkpoint:** Bot can create and claim swaps end-to-end
- ⚙ **Fallback:** Bot links to web wizard

### Phase B2: Swap notifications `[ ]` [1 day]
- [ ] When escrow with `trade_hash` is created: notify counterparty via bot (if known)
- [ ] When swap is claimed: notify both parties
- [ ] When swap is about to timeout: send reminder
- ✅ **Checkpoint:** Notifications fire for all swap lifecycle events
- ⚙ **Fallback:** Generic escrow notifications (already exist)

---

## Track C: Receipts `[ ]`

**Timebox:** 1-2 days

### Phase C1: Swap receipt `[ ]` [1 day]
- [ ] Existing receipt system (`GET /v1/receipts/:id`) already works for all escrows
- [ ] Add swap-specific fields to receipt: preimage hash, preimage (if you're the revealer)
- [ ] Receipt page: "Atomic Swap Complete" with exchange rate, amounts, timestamps
- ✅ **Checkpoint:** Receipt includes swap-specific data
- ⚙ **Fallback:** Use existing generic escrow receipt

### Phase C2: Receipt sharing `[ ]` [1 day]
- [ ] "Share receipt" button: copies link to receipt page
- [ ] Receipt page is publicly viewable (no auth needed) — proof of trade
- ✅ **Checkpoint:** Can share receipt URL publicly
- ⚙ **Fallback:** Receipt visible to parties only

---

## Execution Strategy

```
Priority 1:
  Track A — Wizard Component (3-5 days)

Priority 2:
  Track B — Bot Support (2-3 days)
  Track C — Receipts (1-2 days)
```

---

## Files to Create/Modify

| Track | Files |
|-------|-------|
| A | `web/src/components/AtomicSwapWizard.tsx` (rewrite), `web/src/pages/SwapPage.tsx` |
| B | `bot/src/index.js` (add commands), `bot/src/lib/api.js` |
| C | `web/src/pages/ReceiptPage.tsx` (update), `indexer/src/api/receipts.rs` (update) |
