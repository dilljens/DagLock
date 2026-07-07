# DagLock — Usability Improvement Sprint

**Goal:** Close the gaps between DagLock and production-ready escrow services before mainnet (June 30). Parallel tracks for trust signals, mobile/UX, and covenant design prep.

**Deadline:** June 30, 2026 ~16:15 UTC (Toccata hard fork)

## Requirements
- [ ] R1: Users can estimate fees before creating an escrow (web + bot)
- [ ] R2: Every escrow/tx/vault links to a Kaspa block explorer
- [ ] R3: Users can block/report bad actors (API + web UI)
- [ ] R4: Users can leave trade feedback after settlement
- [ ] R5: Wallet deep links work — KasWare opens for create/claim/refund
- [ ] R6: First-time web visitors get an onboarding explanation
- [ ] R7: Help center / knowledge base exists at `/help`
- [ ] R8: Covenant upgrade specs ready for post-launch audit

## Pre-resolved Decisions
- **No new Rust crates or npm packages** unless absolutely necessary (stdlib preferred)
- **Fee calculator**: pure frontend JS using shared `calculate_fee` logic (no new API endpoint)
- **Explorer links**: use `https://kas.fyi/` as primary (Kaspa block explorer), configured via env var
- **Blocklist**: new DB table `blocked_users`, new API `/v1/blocks`, no covenant changes
- **Trade feedback**: new DB table `trade_feedback`, new API `/v1/escrows/:id/feedback`, no covenant changes
- **Deep links**: KasWare uses `kasware:` URL scheme with action params
- **Onboarding**: single-page modal with 3 slides, stored in localStorage `daglock_onboarded`
- **Help center**: markdown-based FAQ at `/help` route
- **Covenant design docs**: markdown in `docs/design/` — no code changes
- **Error handling**: follow existing patterns — structured API errors (`ApiErrorCode` enum), frontend Toast
- **Testing**: existing test patterns — Rust integration tests, Vitest + RTL for web, Playwright for E2E
- **Email**: optional SMTP integration, opt-in per user

---

## Track A: Trust Signals `[ ]`
**Description:** Build trust with fee transparency, explorer visibility, reputation controls, and trade feedback. Low effort, high trust impact.

### Phase A1: Fee calculator `[ ]`
**⏱ Timebox:** 4h
- [ ] Create `FeeCalculator` component (web) — input amount KAS → shows fee (0.5%) + net to seller + treasury amount
- [ ] Add KAS ↔ sompi toggle
- [ ] Add USD estimate (fetches from existing `/v1/network/price`)
- [ ] Add `/fee <amount>` bot command
- [ ] Include fee calculator in escrow creation flow (web create form)
**✅ Checkpoint:** Open web, enter 1000 KAS → shows 5 KAS fee, 995 KAS net, ~$X USD
**⚙ Fallback:** Deploy fee calculator as raw HTML/CSS widget, skip USD estimate

### Phase A2: Block explorer links `[ ]`
**⏱ Timebox:** 3h
- [ ] Add `explorer_base_url` config to indexer (env `EXPLORER_BASE_URL`, default `https://kas.fyi`)
- [ ] Add `GET /v1/network/explorer` endpoint returning base URL
- [ ] Web: add "View on Explorer" links for: escrow lock TX, escrow settlement TX, vault TX, any address, any TX ID
- [ ] Bot: add explorer links in `/status`, `/receipt`, `/vaults` responses
**✅ Checkpoint:** Web escrow detail page shows clickable explorer link → opens kas.fyi in new tab
**⚙ Fallback:** Hardcode `kas.fyi` in web only, skip API endpoint

### Phase A3: Blocklist / report user `[ ]`
**⏱ Timebox:** 5h
- [ ] Add `blocked_users` DB table: `id, blocker_address, blocked_address, reason, created_at`
- [ ] Add API endpoints: `POST /v1/blocks`, `DELETE /v1/blocks/:id`, `GET /v1/blocks?address=`
- [ ] Add `/v1/reports` endpoint for reporting users
- [ ] Web: block button on reputation lookup page; hide blocked users' offers
- [ ] Bot: `/block <address>`, `/unblock <address>`, `/report <address> <reason>` commands
- [ ] API: filter escrows/offers to hide blocked counterparty's items
**✅ Checkpoint:** `curl -X POST .../v1/blocks -d '{"blocked_address":"kaspa:abc...","reason":"scam"}'` → 201
**⚙ Fallback:** API-only (no bot commands), defer web UI block button styling

### Phase A4: Trade feedback / comments `[ ]`
**⏱ Timebox:** 4h
- [ ] Add `trade_feedback` DB table: `id, escrow_id, reviewer_address, rating (1-5), comment, created_at`
- [ ] Add API: `POST /v1/escrows/:id/feedback`, `GET /v1/escrows/:id/feedback`
- [ ] Only allow feedback after settlement (check escrow status)
- [ ] Web: feedback form on escrow detail after settled — star rating + optional comment
- [ ] Bot: `/feedback <id> <rating> [comment]` command
- [ ] Show feedback count + average rating on reputation page
**✅ Checkpoint:** `curl -X POST .../v1/escrows/<id>/feedback -d '{"rating":5,"comment":"great trader"}'` → 201
**⚙ Fallback:** Skip bot command, web-only feedback form

---

## Track B: Mobile & UX Flow `[ ]`
**Description:** Fix the biggest mobile friction points. Meet Kaspa users where they are.

### Phase B1: Wallet deep links `[ ]`
**⏱ Timebox:** 6h
- [ ] Research KasWare URL scheme (`kasware:` protocol, action format)
- [ ] Add deep link generation to web: "Create Escrow" → `kasware:send?to=...&amount=...` or equivalent
- [ ] Add deep link for claim + refund: same pattern
- [ ] Bot: deep link URLs in `/create` response, `/claim` response
- [ ] Test on Kaspium mobile workflow (link opens mobile wallet)
**✅ Checkpoint:** Create escrow on mobile → tap link → KasWare/Kaspium opens with pre-filled TX
**⚙ Fallback:** Copy-to-clipboard fallback if wallet protocol not supported (manual paste)

### Phase B2: Web onboarding wizard `[ ]`
**⏱ Timebox:** 5h
- [ ] Create `OnboardingModal` component — 3 slides:
  1. "What is DagLock?" — trustless escrow explained in 2 sentences
  2. "How it works" — 4-step flow (propose → lock → confirm → settle)
  3. "Get started" — connect KasWare wallet or manual address
- [ ] Store dismissed state in `localStorage` key `daglock_onboarded`
- [ ] Show on first visit (no `daglock_onboarded` in localStorage)
- [ ] Add "Skip tour" button + "Show again" link in sidebar footer
**✅ Checkpoint:** Clear localStorage → reload → onboarding shows → dismiss → reload → does not show
**⚙ Fallback:** Single static banner at top of Dashboard instead of modal

### Phase B3: Help center / knowledge base `[ ]`
**⏱ Timebox:** 4h
- [ ] Create `HelpPage.tsx` with sections:
  - FAQ (accordion): "What is escrow?", "How do fees work?", "What if the other party doesn't respond?", "How do disputes work?", "What is the jury system?"
  - Quick start guide: numbered steps for first escrow
  - Glossary of terms (from `docs/wiki/_glossary.md`)
- [ ] Add `/help` route to `App.tsx` router
- [ ] Add help link in sidebar
- [ ] Bot: `/help` command — add link to web help page
**✅ Checkpoint:** Navigate to `/help` → see FAQ with working accordion, glossary, quick start
**⚙ Fallback:** Static markdown-rendered page instead of interactive accordion

### Phase B4: Email notifications `[ ]`
**⏱ Timebox:** 6h
- [ ] Add optional SMTP config to indexer (env `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASS`, `NOTIFICATION_FROM`)
- [ ] Add `user_notifications` table: `address, email, email_verified`
- [ ] Add `notification_preferences` table: `address, event TEXT, channel TEXT`
- [ ] Add email verification flow (send code, verify endpoint)
- [ ] Add notification dispatch on escrow status changes
- [ ] Web: notification preferences on settings page
- [ ] Bot: `/notifications` command to set email
**✅ Checkpoint:** Create escrow → email sent to buyer + seller with status + link
**⚙ Fallback:** Defer entirely — ship without email, add post-launch (lowest priority phase)

---

## Track C: Covenant Upgrades (Design & Spec) `[ ]`
**Description:** Research and document covenant changes. No code changes — only spec documents for post-launch audit. This track is non-blocking for mainnet.

### Phase C1: Partial refund / return mechanism `[ ]`
**⏱ Timebox:** 4h
- [ ] Research approaches:
  - A: New `partialRelease(buyerSig, sellerSig, buyerAmount, sellerAmount)` entrypoint
  - B: Two-step — refund to intermediary UTXO then re-settle with split
  - C: Off-chain — full settle + off-chain payment (trust-based)
- [ ] Analyze trade-offs (covenant complexity vs. UX vs. trust)
- [ ] Write `docs/design/partial-refund.md`
- [ ] Include: covenant changes, API changes, UI, migration path
**✅ Checkpoint:** `docs/design/partial-refund.md` exists with all options analyzed
**⚙ Fallback:** Document unknowns and move on

### Phase C2: Time extension mechanism `[ ]`
**⏱ Timebox:** 3h
- [ ] Research approaches:
  - A: `extendTimeout(prevTimeoutSig, newTimeout)` — pre-signed extension
  - B: Multi-phase timeout (first timeout → extension window → final)
  - C: `agreeExtension(buyerSig, sellerSig, newTimeout)` mutual entrypoint
- [ ] Write `docs/design/time-extension.md`
- [ ] Include: covenant entrypoint, API changes, bot flow, frontend UI
**✅ Checkpoint:** `docs/design/time-extension.md` exists
**⚙ Fallback:** Document as "deferred — mutual re-create is current workaround"

### Phase C3: Broker / agent role `[ ]`
**⏱ Timebox:** 3h
- [ ] Design three-party escrow flow: Buyer → Escrow → Broker → Seller
- [ ] Analyze fee splitting (0.5% protocol + optional broker fee)
- [ ] Write `docs/design/broker-role.md`
- [ ] Include: covenant changes (new `brokerKey` field), API changes, offer board updates
**✅ Checkpoint:** `docs/design/broker-role.md` exists
**⚙ Fallback:** Defer — OTC desks can use manual fee splitting

---

## Out of Scope
- Self-hosted Kaspa node (deferred per user)
- Atomic swap wizard UI (already in PENDING.md)
- Escrow-as-a-service widget (post-launch)
- KRC-20 token explorer (post-launch)
- Cross-chain HTLC (post-launch)
- Volume-based fee rebates (no volume yet)
