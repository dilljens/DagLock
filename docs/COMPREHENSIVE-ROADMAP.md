# DagLock Comprehensive Roadmap

## Current Status
- ✅ Web dashboard live at daglock.com
- ✅ API backend on Railway
- ✅ Telegram bot (@DagLock_bot)
- ✅ CLI tool
- ✅ Price-locked offers (15-min CoinGecko updates)
- ✅ Basic vault system
- ✅ API docs (OpenAPI + reference)
- ✅ wRPC listener (polling)
- ❌ Wallet integration (critical)
- ❌ Real on-chain verification (critical)
- ❌ KRC-20 trading UI
- ❌ User onboarding
- ❌ Mobile responsive
- ❌ Production security

---

## P0 — Critical (Must Have Before Mainnet)

### 1. Wallet Integration (KasWare Browser Extension)

**Goal:** Users sign transactions with their Kaspa wallet instead of typing fake signatures.

**What it does:**
- Detect KasWare extension (`window.kasware`)
- Connect wallet (request addresses)
- Sign messages for auth
- Sign transactions for settlement/refund

**Files:**
- `web/src/kasware.ts` — Wallet connector module
- `web/src/App.tsx` — Connect button, signed flow

**Effort:** 3-5 days
**Status:** ❌ Not started

### 2. Real Authentication (Replace Mock Verifier)

**Goal:** Verify actual Schnorr signatures instead of accepting any hex string.

**What it does:**
- Real signature verification in `indexer/src/auth.rs`
- Remove mock verifier acceptance of "abcd"
- Require proper signed messages for lifecycle actions

**Files:**
- `indexer/src/auth.rs` — Signature verification
- `indexer/src/verification.rs` — UTXO verification (real)

**Effort:** 2-3 days
**Status:** ⏸️ Real crypto exists, mock verifier is default

### 3. On-Chain UTXO Verification

**Goal:** Verify escrows actually exist on-chain before allowing operations.

**What it does:**
- Connect to Kaspa node via wRPC
- Verify UTXOs exist and match escrow parameters
- Reject operations on non-existent UTXOs

**Files:**
- `indexer/src/verification.rs` — RealVerifier implementation
- `indexer/src/listener.rs` — Block detection

**Effort:** 3-5 days
**Status:** ⏸️ wRPC listener exists (polling), verification is still mock

### 4. Remove Mock Mode

**Goal:** Users must have real testnet KAS to create escrows.

**What it does:**
- Disable mock signature acceptance
- Require real wallet interaction
- Show testnet faucet prominently for new users

**Effort:** 1-2 days
**Status:** ❌ Not started

---

## P1 — Launch Quality

### 5. KRC-20 Token Trading UI

**Goal:** Users can trade KRC-20 tokens (like NACHO, KASPY) through the dashboard.

**What it does:**
- KRC-20 token selector in CreateOfferForm
- KRC-20 balance display
- KRC-20 escrow creation flow
- Token price display (if available)

**Files:**
- `web/src/App.tsx` — New form fields
- `web/src/api.ts` — Token list endpoint

**Dependencies:**
- Needs on-chain KRC-20 data from Kaspa node
- Needs template hash for KRC-20 covenant

**Effort:** 3-5 days
**Status:** ❌ Not started

### 6. User Onboarding Flow

**Goal:** New users know what to do in under 30 seconds.

**What it does:**
- "Get started" button → testnet faucet → set address → create offer
- Tooltip tour of dashboard
- Pre-filled test addresses for sandbox mode
- "What is an escrow?" explanation for newcomers

**Files:**
- `web/src/App.tsx` — Onboarding overlay
- `web/src/styles.css` — Onboarding styles

**Effort:** 2-3 days
**Status:** ❌ Not started

### 7. Mobile Responsive Design

**Goal:** The dashboard works on phones (Kaspa community is mobile-heavy).

**What it does:**
- Responsive CSS for all panels
- Touch-friendly buttons and inputs
- Collapsible sections for small screens
- Bottom navigation instead of side layout

**Files:**
- `web/src/styles.css` — Media queries
- `web/src/App.tsx` — Responsive layout changes

**Effort:** 2-3 days
**Status:** ❌ Not started

### 8. Offer History & Status Tracking

**Goal:** Users can see history of their offers (accepted, cancelled, expired).

**What it does:**
- Offer status timeline
- "My offers" filter (created vs accepted)
- Expired offer reconciliation
- Offer activity notifications

**Files:**
- `indexer/src/db/queries.rs` — Offer history queries
- `web/src/App.tsx` — Offer history display

**Effort:** 2-3 days
**Status:** ⏸️ Basic listing exists, no history

---

## P2 — Post-Launch Improvements

### 9. Production Security Hardening

| Item | Effort |
|------|--------|
| Rate limiting (requests/min per IP) | 1 day |
| SQL injection audit | 1 day |
| Input validation hardening | 1 day |
| CORS strict mode | 0.5 day |
| Error message sanitization | 0.5 day |
| HTTPS enforcement | Already done |

### 10. Telegram Bot Enhancements

| Feature | Effort |
|---------|--------|
| /vault command | 1 day |
| /vaults command (list) | 1 day |
| Inline keyboards for escrow actions | 1 day |
| Notifications when offers match | 2 days |
| Price alerts | 2 days |

### 11. Developer Experience

| Item | Effort |
|------|--------|
| Webhook system (events → HTTP callbacks) | 3 days |
| Rate limit headers | 1 day |
| API key system for automated trading | 3 days |
| SDK examples in README | 1 day |
| Sandbox mode with demo data | 2 days |

### 12. Advanced Features

| Feature | Effort |
|---------|--------|
| Limit orders (price triggers) | 3 days |
| DCA / recurring buys | 4 days |
| Tax export (CSV) | 2 days |
| Multi-language (i18n) | 3 days |
| Dark/light theme toggle | 1 day |

---

## Implementation Timeline

| Phase | Features | Est. Time |
|-------|----------|-----------|
| **Phase 1: Wallet + Auth** | KasWare integration, real auth, remove mock mode | 5-7 days |
| **Phase 2: On-Chain** | Real UTXO verification, KRC-20 support | 5-7 days |
| **Phase 3: UX** | Onboarding, mobile responsive, offer history | 4-6 days |
| **Phase 4: Polish** | Security hardening, Telegram enhancements | 3-5 days |
| **Phase 5: Growth** | Developer tools, advanced features | 5-7 days |

**Total estimated time to mainnet-ready:** 3-4 weeks
