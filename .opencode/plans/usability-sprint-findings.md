# Findings: DagLock Usability Sprint

## Requirements Discovery
- **User priority:** "All [tracks] in parallel" — trust signals, mobile/UX, and covenant design can proceed simultaneously
- **Deadline:** Before mainnet (June 30) — 4-day sprint
- **Node deferral:** Self-hosted Kaspa node skipped for now (MockVerifier remains)

## Architecture Research

### Fee Calculator
- Existing `shared/src/validation.rs` has `calculate_fee(amount_sompi) -> u64` and `calculate_net_amount() -> u64`
- Existing `shared/src/constants.rs` has `FEE_DENOMINATOR = 200`
- Existing `/v1/network/price` endpoint returns KAS/USD price
- **Decision:** Pure frontend component using hardcoded 0.5% logic (mirrors covenant) — no new API needed

### Block Explorer
- `kas.fyi` is the primary Kaspa block explorer — supports TX hash lookups, address lookups
- URL patterns: `https://kas.fyi/transaction/<txid>`, `https://kas.fyi/address/<address>`
- **Decision:** Configurable via `EXPLORER_BASE_URL` env var, default `https://kas.fyi`

### Deep Links
- KasWare: uses custom protocol `kasware:`. Action format TBD — needs testing with KasWare extension source
- Kaspium: also supports URI scheme for sends
- **Fallback:** if protocol not detected, show QR code or copy-to-clipboard

### Blocklist
- Simple DB table — no covenant changes needed
- API-level filtering on escrow/offer queries (exclude blocked users)
- **Privacy consideration:** blocked_by is never exposed — only blocker sees their blocklist

### Trade Feedback
- Only allowed post-settlement (enforced in API)
- One feedback per escrow per address (upsert)
- Rating 1-5 (integer), optional text comment
- Display average rating + count on reputation endpoint

### Email Notifications
- Use `lettre` crate for Rust SMTP (existing ecosystem, lightweight)
- Optional — off by default, opt-in per user
- Events: escrow.created, escrow.settled, escrow.disputed, escrow.refunded, escrow.expired
- Rate-limited (max 10 emails/address/day)

### Onboarding
- No existing onboarding (audit item U7 was marked low-priority)
- 3-slide modal is the minimal viable version
- Future: could expand to guided tour of each page

### Help Center
- Existing `DocsPage.tsx` at `/docs` — contains markdown docs
- New `HelpPage.tsx` adds FAQ accordion + quick start + glossary
- FAQ questions sourced from common user questions:
  - "What is an escrow?"
  - "How are fees calculated?"
  - "What if the other party doesn't respond?"
  - "How do disputes and the jury system work?"
  - "What currencies are supported?"
  - "How long does an escrow take?"

## Open Questions → Resolved
- Q: Should fee calculator be a new page or embedded? → A: Embedded component on escrow creation form + dedicated section on dashboard
- Q: Should blocklist be on-chain? → A: No — off-chain DB only. On-chain would be expensive and unnecessary for blocking.
- Q: Should trade feedback be on-chain (via reputation covenant)? → A: No — off-chain. The reputation covenant records trade outcomes, not sentiment. Feedback is human context.
- Q: Email SMTP dependency? → A: `lettre` crate + `tokio` for async. Both already in the project's dep tree.
- Q: KasWare deep link format? → A: Research needed — check KasWare extension docs or source. Fallback: use `kaspa:` URI scheme which KasWare/Kaspium both support.
