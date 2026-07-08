# Findings: Reddit Testnet Tutorial Post

## Requirements (User Answers)

| Question | Answer |
|----------|--------|
| Post purpose | Separate testnet tutorial (before main launch) |
| Target subreddit | r/kaspa only |
| Visuals | Screenshots in post (3-4 annotated) |
| Delivery | Create draft file at `docs/reddit-testnet-post.md`, post manually |

## Pre-resolved Decisions

- **Style**: Friendly, step-by-step, no assumed knowledge. "No wallet, no KAS, no problem."
- **Wallet addresses**: Use the same public test wallets already in `TestnetPage.tsx` and `testnet-quickstart.md` for consistency:
  - Buyer: `kaspa:qtqwyqtmgczzjmj44vjzy`
  - Seller: `kaspa:qjdpca9zm8aafdue2q0zn`
  - Mediator: `kaspa:qyp29592perates764gj8`
- **TX ID example**: `deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef` (exactly 64 hex chars) — matches existing docs
- **Mock signature**: Reddit post can say "Click 'Mock sign' button" — the `SignWithWallet` component handles this
- **Screenshots**: Need 4 new targeted screenshots (local indexer + Playwright)
  - Leave screenshot generation for a separate implementation phase
  - The existing 11 screenshots (`screenshots/*.png`) can serve as fallback

## Post Structure Research

### r/kaspa Community Characteristics
- r/kaspa (~90k members) is generally welcoming of Kaspa ecosystem projects
- The subreddit has active discussion of Kaspa technology, price, and ecosystem
- Technical posts about Kaspa infrastructure (covenants, KRC-20, smart contracts) get good engagement
- Users appreciate clear, honest communication about project maturity
- Best posting times: weekdays 14:00-16:00 UTC (catches European afternoon + US morning)
- Pro tip: Engage with comments in the first few hours to boost visibility

### What NOT to do
- No financial advice, price predictions, or investment suggestions
- Don't over-promise (clearly label "pre-mainnet", "internal audit", "testnet")
- Don't use excessive emojis or clickbait formatting
- Don't make it look like an ad — frame it as a community resource

## Technical Notes

### MockVerifier + Testnet Mode
- Indexer runs with `--mock-auth` → accepts any signature
- Web UI shows testnet banner
- Manual mode bypasses KasWare entirely
- The `SignWithWallet` component provides "Mock sign (dev mode)" button

### Current Test Wallet Sources
1. **.env.testwallets** — Internal test keys for dev automation
2. **TestnetPage.tsx** — Public test wallets shown on the web UI
3. **testnet-quickstart.md** — Same wallets as TestnetPage, documented for users

Use #2/#3 (TestnetPage wallets) for the Reddit post.

### Screenshot Approach
Current screenshots (`scripts/screenshots.cjs`) are generic page captures at 1280x800.
For the tutorial post, we need specific screenshots showing the actual filled-out forms and settled state.
Best approach: Run indexer locally, create test data, then take Playwright screenshots of specific elements.

## Open Questions → Resolved

- Q: Which test wallet addresses to use? → A: The public ones from TestnetPage.tsx (consistent with existing docs)
- Q: Where to put the draft file? → A: `docs/reddit-testnet-post.md` (matches convention from pre-announcement plan which referenced `docs/REDDIT-POST.md`)
- Q: Should screenshot capture be part of this task? → A: Yes but in a separate phase — post content first, screenshots as follow-up
