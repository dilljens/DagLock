# Plan #2: Atomic Swap Wizard UI

**Goal:** Replace the raw hex-entry swap UI with a step-by-step guided wizard that walks users through the atomic swap flow — generate secret, share hash, wait for counterparty, reveal preimage, settle.

**Effort:** 4-5 days

**Current state:** `POST /v1/swap/generate` returns `{secret, hash}`. `POST /v1/escrows/:id/swap` accepts `{preimage}`. Web has `SwapForm` with raw text inputs for escrow ID + preimage hex. Users must manually manage the entire flow with hex strings.

---

## Phase 2A: Swap wizard state machine `[ ]`
**⏱ Timebox:** 1 day

- [ ] Create `web/src/components/AtomicSwapWizard.tsx` with a state machine:
  ```
  type SwapStep = 
    | "init"        — user enters amount, counterparty, selects asset
    | "secrets"     — "Generate Secret" button → shows secret once + copy button
    | "create"      — escrow created with trade_hash, shows share link
    | "waiting"     — polling escrow status every 10s, countdown timer
    | "claim"       — counterparty enters preimage → validates sha256
    | "done"        — receipt link, success animation
  ```
- [ ] Step "init": Amount input, asset selector (KAS/KRC-20), counterparty address
- [ ] Step "secrets": Generate button → calls `api.generateSwap()` → displays secret in warning banner with copy button ("Save this! You won't see it again!")
  - Secret shows in a styled box with red border + "⚠️ Copy this now" warning
  - Trade hash auto-populates for the create step
- [ ] Step "create": Uses existing `CreateEscrowRequest` with `trade_hash` pre-filled
  - After creation, shows shareable link: `https://daglock.com/swap/{escrow_id}`
  - Telegram deep link: `https://t.me/DagLock_bot?start=swap_{escrow_id}`
  - QR code for mobile (simple inline SVG, no library needed)
- [ ] Step "waiting": Polls `api.escrow(id)` every 10 seconds
  - Shows: "Waiting for counterparty to claim..." with animated dots
  - Countdown timer: "Time remaining before refund: 23h 59m"
  - Cancel button if still in `proposed` state
- [ ] Step "claim" (counterparty view): Preimage hex input or paste from clipboard
  - Auto-detects if URL ends in `?preimage=xxx` for clickable links
  - "Claim" button → calls `api.swapEscrow(id, preimage)`
- [ ] Step "done": Checkmark animation + receipt link + "Start another swap" button

**✅ Checkpoint:** Full wizard flow works on testnet — create swap with secret, share link, counterparty claims with preimage

---

## Phase 2B: Add wizard to existing pages `[ ]`
**⏱ Timebox:** 1 day

- [ ] Add "Atomic Swap" tab to `EscrowsPage.tsx` alongside "Create", "Lookup", etc.
  ```tsx
  <button className={`tab-btn ${tab === "swap" ? "tab-btn--active" : ""}`}
          onClick={() => setTab("swap")}>
    Atomic Swap
  </button>
  ```
- [ ] Wire the wizard: `{tab === "swap" && <AtomicSwapWizard />}`
- [ ] Create `SwapPage.tsx` — dedicated page at `/swap` route (already exists as a simple form)
  - Replace existing `SwapForm` with the new `AtomicSwapWizard` component
  - Add route for `/swap/:id` to deep-link directly into a specific swap's claim step
- [ ] Update `Dashboard.tsx` — add "Atomic Swap" quick action card
- [ ] Add navigation: sidebar "Swap" → swap wizard (already exists)

**✅ Checkpoint:** Navigate to `/swap` → see step-by-step wizard, not raw hex inputs

---

## Phase 2C: Counterparty deep links `[ ]`
**⏱ Timebox:** 1 day

- [ ] Bot: `/swap <escrow-id>` command should detect if escrow has a `trade_hash` and show different UI
  - If trade_hash present: "This is an atomic swap! Claim by clicking: https://daglock.com/swap/{id}"
  - If no trade_hash: normal settlement flow
- [ ] Bot: `/start swap_<id>` deep link handler → redirects to web swap page
- [ ] Web: `/swap/:id` route loads escrow data and initializes wizard at "claim" step if user is counterparty
- [ ] Web: if user navigates to `/swap/:id` and they are the initiator, show "waiting" step

**✅ Checkpoint:** Telegram link `t.me/DagLock_bot?start=swap_<id>` → claims escrow in wizard

---

## Phase 2D: Tooltips, edge cases, polish `[ ]`
**⏱ Timebox:** 1 day

- [ ] Error states for each step:
  - Invalid counterparty address → red field + message
  - Secret lost → "We can't recover your secret. You'll need to cancel and start over."
  - Wrong preimage → "Invalid preimage — sha256 doesn't match escrow's trade hash"
  - Escrow already claimed → "This swap has already been settled"
  - Timeout expired → "Time's up! The escrow can now be refunded."
- [ ] Copy-to-clipboard for secret + share link with toast notification
- [ ] Loading spinners for API calls at each step
- [ ] Mobile responsive — wizard should work on phone screens (single column, large tap targets)
- [ ] Add `FeeCalculator` to "init" step so users see the 0.5% fee before creating

**✅ Checkpoint:** All error states render correctly, mobile layout works, fee shows before creation

---

## Phase 2E: Tests `[ ]`
**⏱ Timebox:** 1 day

- [ ] `web/src/__tests__/AtomicSwapWizard.test.tsx`:
  - Renders all 6 steps
  - Generate secret calls API and displays result
  - Create with trade_hash escrow created
  - Wrong preimage rejected
  - Correct preimage settles
  - Cancel works during "waiting" step
- [ ] Playwright E2E test (optional, if suite exists):
  - Full flow: create swap → share → claim → receipt

**✅ Checkpoint:** `npm test -- --run` passes with new tests
