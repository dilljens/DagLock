# Progress: Reddit Testnet Tutorial Post

## Session 2026-07-07

### Summary
- Created plan files for the Reddit testnet tutorial post
- Researched the project's testnet infrastructure, test wallets, and existing documentation
- Mapped out post structure based on user preferences (separate tutorial, r/kaspa, screenshots, draft file)

### Status
- [x] Research r/kaspa community (known characteristics from ecosystem docs)
- [x] Review existing screenshots and test wallet setup
- [x] Review existing testnet-quickstart.md + TestnetPage.tsx content (avoids duplication)
- [x] Review pre-announcement plan for posting strategy
- [ ] Track A Phase A1: Write post draft (not started — waiting for approval)
- [ ] Track A Phase A2: Generate screenshots (not started)
- [ ] Track B Phase B1: Editorial review (not started)

### Decisions Made
- Use public test wallets from TestnetPage.tsx (consistent across all surfaces)
- Post goes to docs/reddit-testnet-post.md
- 4 new targeted screenshots needed (use Playwright script)
- Post focuses on web flow (create → settle), with bot as optional bonus

### Key Findings
- The existing screenshot tool takes generic 1280x800 screenshots of 11 pages
- Test wallets have their private keys publicly exposed — clearly label "TESTNET ONLY, NO REAL FUNDS"
- The pre-announcement plan already references a `docs/REDDIT-POST.md` file (which doesn't exist yet)
- MockVerifier mode means literally ANY hex string works as a signature — great for demos
