# Plan: In-App Negotiation + Proportional Swaps (P1)

**Goal:** Turn the offer board from a read-only listing into an interactive marketplace. Users can counter-offer, haggle, and execute partial atomic swaps (e.g., "I'll sell you 50 of my 100 locked KAS").

**Effort:** 3-5 days + covenant audit for proportional swaps

---

## Part A: Counter-Offer System `[ ]`

### Phase A1: Counter-offers API `[ ]`
**⏱ Timebox:** 1 day

- [ ] New DB table `offer_counteroffers`:
  ```sql
  CREATE TABLE offer_counteroffers (
      id TEXT PRIMARY KEY,
      offer_id TEXT NOT NULL REFERENCES offers(id),
      proposer_address TEXT NOT NULL,
      amount_sompi INTEGER,
      price_offset REAL,
      timeout INTEGER,
      dispute_mode TEXT,
      message TEXT,
      status TEXT NOT NULL DEFAULT 'pending',
      created_at INTEGER NOT NULL
  );
  CREATE INDEX idx_counteroffers_offer ON offer_counteroffers(offer_id);
  ```
- [ ] API: `POST /v1/offers/:id/counter` — propose a counter-offer (body: amount, price, message)
- [ ] API: `GET /v1/offers/:id/counters` — list counter-offers on an offer
- [ ] API: `POST /v1/counteroffers/:id/accept` — accept → creates escrow with negotiated terms
- [ ] API: `POST /v1/counteroffers/:id/decline` — reject
- [ ] API: `POST /v1/counteroffers/:id/withdraw` — proposer backs out
- [ ] Validation: can't counter your own offer. Counter must be within ±50% of original amount.
- [ ] Validation: max 10 active counters per offer (anti-spam)

**✅ Checkpoint:** `curl POST /v1/offers/id/counter -d '{"amount_sompi":...}'` → 201 with counter-offer

### Phase A2: Web negotiation UI `[ ]`
**⏱ Timebox:** 1 day

- [ ] Offer cards show "Counter" button + counter count badge ("3 counters")
- [ ] Counter-offer form: editable amount, price, message text
- [ ] Negotiation thread view (expandable): shows offer → counter → counter → accept timeline
- [ ] Accept/Decline buttons on each counter
- [ ] Real-time counter count updates via WebSocket
- [ ] Notification toast: "New counter-offer on your listing!"

**✅ Checkpoint:** Click counter → fill form → submits → appears in thread → accept → escrow created

### Phase A3: Bot negotiation commands `[ ]`
**⏱ Timebox:** 1 day

- [ ] `/counter <offer-id> <amount> [message]` — propose a counter
- [ ] Bot notification: "New counter-offer on your offer!" with inline [View] [Accept] [Decline]
- [ ] `/counters <offer-id>` — list counters with inline actions
- [ ] `/negotiate <offer-id>` — start a back-and-forth via inline keyboard

**✅ Checkpoint:** Bot user receives counter notification, taps Accept → escrow created.

---

## Part B: Proportional Atomic Swap Covenant `[ ]`

**Problem:** Alice has 100 KAS locked in an escrow. Bob wants to trade for only 50 KAS worth of NACHO. Current covenant only does full-release. They'd need to refund the whole 100 KAS first, then create a new 50 KAS swap — wasteful and slow.

**Solution:** A new covenant entrypoint `partialSwap` that splits the UTXO proportionally.

### Phase B1: Covenant design `[ ]`
**⏱ Timebox:** 2 days (design only — audit required before deployment)

**New entrypoint** in `daglock.sil` (or a new `daglock_partial.sil`):

```silverscript
contract DagLockPartial(
    byte[32] buyerKey,
    byte[32] sellerKey,
    byte[32] tradeHash,       // SHA-256 of secret (zeroes = market order)
    int timeout,
    byte[32] treasuryKey
)

// Path A: Partial atomic swap
// sellerKey provides secret preimage → sha256(secret) == tradeHash
// Splits: amountToSeller goes to seller, rest to buyer
swap_partial(byte[] secret, int amountToSeller) {
    require(sha256(secret) == tradeHash);
    
    int fee = amountToSeller / 200;          // 0.5% on settled portion
    int netToSeller = amountToSeller - fee;
    int returnToBuyer = inputValue - amountToSeller;
    
    // Output 0: amount to seller (minus fee)
    // Output 1: change back to buyer
    // Output 2: fee to treasury
}

// Path B: Full atomic swap (existing)
swap(byte[] secret) {
    // existing logic — full release to seller
}

// Path C: Mutual release (existing)
release(sig buyerSig, sig sellerSig) { ... }

// Path D: Timeout refund (existing)
refund(sig buyerSig) { ... }
```

**Key design decisions:**
| Decision | Choice | Rationale |
|----------|--------|-----------|
| Fee on settled portion only | ✅ `amountToSeller / 200` | Fair — buyer only pays fee on what they trade |
| Change output first | ✅ Output 1 = buyer | Follows Kaspa convention (sender gets change) |
| Minimum settlement | 1000 sompi (0.00001 KAS) | Prevents dust outputs |

**✅ Checkpoint:** `docs/design/partial-swap-covenant.md` written with full spec, ready for audit

### Phase B2: Covenant implementation `[ ]`
**⏱ Timebox:** 3 days (requires audit — do NOT deploy without review)

- [ ] Write `contracts/src/daglock_partial.sil` with the 4 entrypoints
- [ ] Add `contracts/src/lib.rs` compile method + template hash extraction
- [ ] Add execution tests for:
  - `swap_partial` with valid preimage → 2 outputs (seller gets amount, buyer gets change)
  - `swap_partial` with amount = inputValue → behaves like full swap (1 output to seller + fee)
  - `swap_partial` with amount > inputValue → rejected
  - `swap_partial` with wrong preimage → rejected
  - Full `swap` → still works (backwards compat)
  - `release` + `refund` → still work
- [ ] Generate template hash
- [ ] Security review — specifically: fee rounding, overflow checks, output ordering

**✅ Checkpoint:** `cargo test -p daglock-contracts` passes all partial swap tests

### Phase B3: Indexer + API support `[ ]`
**⏱ Timebox:** 1 day

- [ ] Register new template hash in indexer config
- [ ] Add `partial_swap` field to escrows (optional: amount_to_seller, settlement_ratio)
- [ ] Extend `POST /v1/escrows` to accept `covenant_type: "partial"` to use new covenant
- [ ] Add `POST /v1/escrows/:id/partial-swap` endpoint — accepts `{preimage, amount_to_seller}`
- [ ] Update web UI: "Partial Swap" option in create form
- [ ] Update atomic swap wizard: add "amount to swap" slider (percentage of locked funds)

**✅ Checkpoint:** Create partial escrow → settle with partial amount → outputs match expected split

### Phase B4: Integration tests `[ ]`
**⏱ Timebox:** 1 day

- [ ] Integration test: create partial escrow → swap_partial with 50% → verify outputs
- [ ] Integration test: fee calculation on partial amount
- [ ] Integration test: backwards compat — full swap still works on old covenants
- [ ] Web test: slider shows percentage, fee updates live

**✅ Checkpoint:** `cargo test --workspace` passes. `npm test` passes.

---

## Files Changed / Created

### Part A: Counter-offers
| File | Change |
|------|--------|
| `indexer/src/db/migrations/022_create_counteroffers.sql` | **New** |
| `indexer/src/db/queries/counteroffers.rs` | **New** |
| `indexer/src/api/counteroffers.rs` | **New** |
| `indexer/src/api/mod.rs` | Register routes |
| `indexer/src/types.rs` | Add counter-offer types |
| `indexer/src/db/schema.rs` | Add migration |
| `web/src/pages/OffersPage.tsx` | Counter button, form, thread view |
| `web/src/api.ts` | Counter API methods |
| `web/src/styles.css` | Counter UI styles |
| `bot/src/index.js` | `/counter`, `/counters`, `/negotiate` commands |
| `bot/src/lib/api.js` | Counter API methods |
| `indexer/src/websocket.rs` | Counter-offer events |

### Part B: Proportional swaps
| File | Change |
|------|--------|
| `contracts/src/daglock_partial.sil` | **New** — covenant with swap_partial |
| `contracts/src/lib.rs` | Add compile method + template hash |
| `contracts/tests/` | Partial swap execution tests |
| `docs/design/partial-swap-covenant.md` | **New** — design spec for audit |
| `indexer/src/config.rs` | Register new template hash |
| `indexer/src/api/escrows.rs` | Add `partial_swap` action |
| `indexer/src/types.rs` | Add partial swap types |
| `web/src/components/AtomicSwapWizard.tsx` | Add "amount to swap" slider |
| `web/src/api.ts` | Add partialSwap method |

## Edge Cases

| Case | Handling |
|------|----------|
| Counter within 50% of original | Allow. Beyond 50%? Reject with "Your counter is too far from the original offer" |
| Multiple counters on same offer | All visible. First accepted wins. Others auto-declined. |
| Counter accepted while amount changed | Atomic: lock offer row before accept |
| Partial swap amount > locked amount | Reject. Must be ≤ inputValue - dust |
| Partial swap with zero amount | Reject (1000 sompi minimum) |
| Partial swap fee rounding | `amountToSeller / 200` — rounds down, treasury gets slightly less |
| Old covenant still works? | Yes — 100% backwards compatible. Only new escrows use the new template hash |
