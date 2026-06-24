# SEO Keyword & Content Plan

## Target Keywords

### Primary (High Intent — Transactional)

| Keyword | Search Volume | Intent | Target Page |
|---------|--------------|--------|-------------|
| "Kaspa escrow" | Medium | Transactional | `/` + `/kaspa-escrow` |
| "trustless escrow Kaspa" | Low-Medium | Informational | `/` + `/how-it-works` |
| "KRC-20 swap" | Medium | Transactional | `/swap` + `/krc20-swap` |
| "Kaspa atomic swap" | Low-Medium | Transactional | `/swap` |
| "Kaspa OTC trading" | Low | Transactional | `/offers` |

### Secondary (Informational — Top of Funnel)

| Keyword | Why | Target Page |
|---------|-----|-------------|
| "what is KRC-20" | Captures token community | Blog post |
| "Kaspa covenant" | Developer audience | `/docs` |
| "SilverScript language" | Developer audience | Blog post |
| "Toccata hard fork" | News/educational | Blog post |
| "crypto escrow without admin keys" | Differentiator | `/security` |

### Long-Tail (Specific Queries)

| Keyword | Target |
|---------|--------|
| "how to safely swap KRC-20 tokens" | Blog post |
| "Kaspa token escrow service" | `/kaspa-escrow` |
| "best way to trade KAS for KRC-20" | Blog post |
| "trustless atomic swap Kaspa tutorial" | Blog post |
| "DagLock vs traditional escrow" | Blog post |
| "whale-to-whale KAS swap platform" | `/offers` |
| "audited Kaspa smart contract escrow" | `/security` |

---

## Content Assets to Create

### Static Landing Pages (pre-rendered HTML)

1. **`/kaspa-escrow`** — Landing page for "Kaspa escrow" keyword
   - Content: What is Kaspa escrow, why trustless, how DagLock works, 0.5% fee, audit badge
   - Call to action: Create escrow, Connect wallet
   - Schema: `Service` + `FinancialProduct`

2. **`/how-it-works`** — Step-by-step escrow process
   - Content: 3-step guide (Lock → Trade → Settle), diagrams, fee explanation
   - Schema: `HowTo`

3. **`/security`** — Audit results + security model
   - Content: Covenant architecture, no admin keys, audit findings, bug bounty
   - Schema: `TechArticle`

4. **`/faq`** — Frequently asked questions
   - Content: 10-15 common questions about fees, safety, supported assets, dispute resolution
   - Schema: `FAQPage` (eligible for rich results)

### Blog Posts (at `/blog/`)

| # | Title | Target Keyword | Est. Words |
|---|-------|---------------|-----------|
| 1 | "What is KRC-20? A Complete Guide to Kaspa's Token Standard" | "what is KRC-20" | 2000 |
| 2 | "How to Safely Swap KRC-20 Tokens: Trustless Escrow Guide" | "swap KRC-20 tokens safely" | 2500 |
| 3 | "Kaspa Escrow: A Complete Guide to Trustless Trading" | "Kaspa escrow" | 2000 |
| 4 | "What is the Toccata Hard Fork? Kaspa Covenants Explained" | "Toccata hard fork" | 1500 |
| 5 | "Atomic Swaps on Kaspa: How They Work" | "Kaspa atomic swap" | 2000 |
| 6 | "DagLock Security Audit: What We Found and Fixed" | "DagLock audit" | 1500 |

---

## Implementation Plan

### Phase 1: Technical Foundation (This Week)
- [x] `_headers` file for caching + robots
- [x] `_redirects` file for SPA fallback
- [ ] Skeleton components → verify proper height/width for CLS

### Phase 2: Pre-rendered Pages (Next Sprint)
- [ ] Create `/how-it-works` as static HTML in `public/`
- [ ] Create `/security` as static HTML in `public/`
- [ ] Create `/faq` as static HTML in `public/`
- [ ] Add `FAQPage` schema to `/faq`
- [ ] Add `HowTo` schema to `/how-it-works`

### Phase 3: Blog (Ongoing)
- [ ] Set up `/blog/` directory with Cloudflare Pages Function
- [ ] Publish 3 pillar posts before mainnet launch
- [ ] Internal link between blog posts and product pages

### Phase 4: Backlinks (Pre-Mainnet)
- [ ] Submit to kaspa.org ecosystem page
- [ ] Publish audit results (drives developer backlinks)
- [ ] Guest post on Kaspa community channels
