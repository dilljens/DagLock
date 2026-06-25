# SEO Overhaul

**Goal:** Make daglock.com discoverable on Google for "Kaspa escrow", "KRC-20 swap", etc.

## Requirements
- [ ] R1: All routes indexable by Google (no hash routing)
- [ ] R2: Rich SERP results (OG tags, structured data)
- [ ] R3: Sitemap + robots.txt for crawlability
- [ ] R4: Per-page unique titles + descriptions

---

## Phase 1: Core SEO `[ ]`
- [ ] Convert `router.tsx` from hash → History API
- [ ] Add `web/public/_redirects` for Cloudflare SPA fallback
- [ ] Install `react-helmet-async` and wrap app
- [ ] Add `<Helmet>` to all 8 page components
- [ ] Update `index.html` with full meta tags
- [ ] Fix hardcoded `href="#/..."` links in OffersPage + SwapPage
- ✅ Checkpoint: `npm run build` passes, routes work without `#`

## Phase 2: Rich SERP Results `[ ]`
- [ ] Create OG image (1200×630)
- [ ] Add `vite-plugin-html` for OG/Twitter tag injection
- [ ] Add JSON-LD structured data (Organization + FinancialService)
- [ ] Generate `sitemap.xml` + `robots.txt`
- ✅ Checkpoint: Open Graph debugger shows correct tags

## Phase 3: Polish `[ ]`
- [ ] Install `vite-plugin-pwa` for manifest + icons
- [ ] Pre-render `/`, `/offers`, `/docs` with static export
- [ ] Submit sitemap to Google Search Console
- [ ] Noindex testnet pages
- ✅ Checkpoint: Lighthouse SEO score 90+
