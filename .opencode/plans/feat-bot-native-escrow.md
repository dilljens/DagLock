# Plan: Bot-Native Escrow Create (P0)

**Goal:** Let users create, sign, and settle escrows entirely within Telegram — no web redirects, no KasWare extension, no browser. The bot handles everything via inline keyboards and Kaspium deep links.

**Effort:** 2-3 days

**Why this matters:** 70%+ of the Kaspa community interacts on Telegram via mobile. The current `/create` wizard redirects to the web for signing, which kills conversion on mobile. Users open the link, see a browser, and give up.

---

## Phase A: Bot conversation wizard `[ ]`
**⏱ Timebox:** 1 day

**Current state:** `/create` starts a 4-step wizard (amount → counterparty → timeout → dispute mode) but then generates a web link and tells the user "complete on web."

**Target state:** Full bot-native flow.

- [ ] Extend the existing `convState` wizard to replace the web redirect with inline keyboard steps:
  ```
  Step 1: "How much KAS?" → user types amount
  Step 2: "Counterparty address?" → user types kaspa:...
  Step 3: "Timeout?" → inline keyboard: [1h] [24h] [3d] [7d]
  Step 4: "Dispute mode?" → inline keyboard: [Standard] [Mediator] [Jury]
  Step 5: "Review:" → show summary with [Confirm] [Cancel]
  Step 6: "Connect your wallet to sign — open Kaspium?" → [Open Kaspium] [Copy TX data]
  ```
- [ ] Generate unsigned TX data as a `kaspa:` URI for Kaspium deep link
- [ ] After broadcast, poll `/v1/escrows/:id` until `pending_confirmation` → confirm to user
- [ ] Store escrow ID in user's local state for quick `/status` access

**Key UX details:**
- Each step has a "Back" button
- User can cancel at any step
- Summary screen before final confirmation shows fee breakdown
- After creation, show inline keyboard: [View Status] [Share with Counterparty] [Done]

**✅ Checkpoint:** User creates an escrow from a fresh Telegram chat, never opens a browser, gets a confirmation message with the escrow ID.

---

## Phase B: Kaspium deep link signing `[ ]`
**⏱ Timebox:** 1 day

- [ ] Research Kaspium URL scheme for transaction signing (`kaspium:` or `kaspa:` URIs)
- [ ] Generate `kaspa:` URI that opens Kaspium with pre-filled covenant address + amount
- [ ] Fallback: if no Kaspium detected, show raw TX data as text + "Copy to clipboard" button
- [ ] Add a `/sign <escrow-id>` command for re-triggering the signing flow if the user lost the prompt
- [ ] Verify: the covenant address is computed via the existing compile endpoint

**Kaspa URI format:**
```
kaspa:<address>?amount=<sompi>&memo=<encoded-memo>
```

**✅ Checkpoint:** Tapping "Open Kaspium" makes the mobile wallet open with pre-filled transaction.

---

## Phase C: Settlement in chat `[ ]`
**⏱ Timebox:** 1 day

- [ ] After an escrow is active, show action buttons: [Settle] [Refund] [Dispute]
- [ ] Settle flow: "Sign to release funds to seller" → deep link to Kaspium → confirm
- [ ] Refund flow: "Sign to refund buyer" → deep link → confirm
- [ ] Upon completion, show receipt card with TX link
- [ ] Auto‑notify both parties when status changes (push via Telegram bot)
- [ ] Add `/settle <id>` and `/refund <id>` commands for power users

**✅ Checkpoint:** Full lifecycle in Telegram: create → counterparty claims → settle → receipt.

---

## Phase D: Tests `[ ]`
**⏱ Timebox:** 4h

- [ ] Update `bot/src/__tests__/` to cover new wizard steps
- [ ] Test: conversation state machine navigates all 6 steps
- [ ] Test: deep-link URI generation matches expected format
- [ ] Test: settlement flow works without web redirect
- [ ] Manual test on Telegram: create escrow end-to-end on mobile

**✅ Checkpoint:** `npm test` passes with new tests.

---

## Files Changed

| File | Change |
|------|--------|
| `bot/src/index.js` | Extend convState wizard: 6 steps instead of redirect |
| `bot/src/lib/api.js` | Maybe no changes (endpoints exist) |
| `bot/src/__tests__/` | New tests for wizard state machine |
| `web/src/kaspa-deeplink.ts` | Export URI generation for reuse (import in bot if needed) |

## Edge Cases

| Case | Handling |
|------|----------|
| User closes chat mid-wizard | State is in-memory (`convState` Map) — lost on restart. Show "sorry, start over" |
| Kaspium not installed | Show instructions + copy-to-clipboard fallback |
| Transaction rejected in wallet | Show "Transaction was rejected. Try again?" with inline button |
| Network error during broadcast | Retry 3 times, then show "Timed out. Check explorer:" with link |
| Counterparty never claims | Auto-notify after timeout expires |
