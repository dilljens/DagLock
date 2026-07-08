# Reddit Testnet Tutorial Post

Goal: Create a Reddit post (r/kaspa) that walks users through testing a DagLock escrow end-to-end using provided testnet wallets — no real KAS, no wallet extension needed.

## Key Decisions

- **Separate testnet tutorial** (not the mainnet launch post). Goes out before the launch.
- **Target**: r/kaspa only
- **Format**: Text + 3-4 annotated screenshots
- **Delivery**: Create draft file at `docs/reddit-testnet-post.md` for manual posting
- **Success criteria**: A complete, beginner-friendly walkthrough that a Kaspa community member can follow in <5 minutes

---

## Proposed Post Content Outline

### Title (3 options, user picks)

Option A (instructional): "Try DagLock trustless escrow on Kaspa testnet — takes 2 minutes, no wallet needed"
Option B (value-first): "You can now test-drive DagLock escrow with a pre-funded test wallet — here's how"
Option C (short): "Test DagLock escrow in 2 minutes — no KAS, no wallet extension required"

### Post Body Structure

**Opening hook** (2-3 sentences):
- DagLock is live on testnet at daglock.com
- Trustless escrow for KAS + KRC-20 tokens via SilverScript covenants
- Full flow works without a wallet or real tokens — here's how

**Step 1: Open the site**
- Go to daglock.com
- Notice the testnet banner
- [Screenshot: homepage with testnet banner highlighted]

**Step 2: Connect with a test wallet**
- Click "Use manual mode" in the sidebar footer
- Paste a test address (provided below)
- [Screenshot: manual mode input with test address filled in]

**Step 3: Create an escrow**
- Go to Escrows → Create tab
- Enter 100 as amount
- For TX ID, paste any 64-char hex: `deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef`
- Click Create
- [Screenshot: create form filled out]

**Step 4: Settle it**
- Click the escrow in "My Escrows"
- Click "Settle"
- Use "Mock sign" button (testnet dev mode)
- Escrow changes to "settled" status
- [Screenshot: settled escrow]

**Step 5: Try the Telegram bot**
- Open @DagLock_bot on Telegram
- `/setaddress` with a test address
- `/create` to start the wizard
- `/offers` to browse live trades

**Test Wallet Addresses** (copy-paste ready):

| Role | Address |
|------|---------|
| Buyer | `kaspa:qtqwyqtmgczzjmj44vjzy` |
| Seller | `kaspa:qjdpca9zm8aafdue2q0zn` |
| Mediator | `kaspa:qyp29592perates764gj8` |

**What else to explore** (quick-fire links):
- **Offer board**: Browse open trade offers
- **Atomic swap wizard**: 6-step guided swap
- **KRC-20 tokens**: Token charts and prices
- **Reputation**: Check an address's trade history
- **Vaults**: Time-locked, multisig, softlock vaults
- **Security page**: Covenant internals explained

**Context / Transparency note**:
- Testnet uses offline verification (MockVerifier)
- Any TX ID and signature are accepted in dev mode
- Mainnet targets June 30 (Toccata hard fork)
- Full audit completed — 28/30 items done
- Open source: github.com/dilljens/DagLock

**Call to action**:
- Try it: daglock.com
- Bot: @DagLock_bot
- Report bugs: GitHub issues or Telegram group
- Mainnet launch coming June 30 — follow for updates

---

## Screenshot Requirements

To make the post effective, we need **4 new targeted screenshots** (the existing ones show empty/static pages). We should update `scripts/screenshots.cjs` to:

1. **Screenshot A**: Homepage with the testnet banner + manual mode button highlighted (dashboard view)
2. **Screenshot B**: Manual mode input with a test address pasted in (wallet.tsx component area)
3. **Screenshot C**: Create escrow form filled with sample data
4. **Screenshot D**: Escrow showing "settled" status after mock signing

These would be taken with a local indexer running in dev mode to ensure the UI has actual data to display.

---

## Plan: Tracks

### Track A: Post Content Creation
**⏱ Timebox**: 45 min
**✅ Checkpoint**: `ls -la docs/reddit-testnet-post.md`

#### Phase A1: Draft the post (30 min)
- [ ] Write full post content following the outline above
- [ ] Include 3 title options
- [ ] Format for Reddit (Markdown + limited HTML)
- [ ] Embed test wallet table
- [ ] ⚙ Fallback: Write a shorter version focused only on the 4-step web flow, cut the bot section

#### Phase A2: Generate screenshots (requires implementation phase)
- [ ] Update `scripts/screenshot-alt.cjs` (or extend existing) to capture the 4 targeted views
- [ ] Takes indexer running locally in `--mock-auth` mode
- [ ] Creates screenshots in `screenshots/reddit/` directory
- [ ] Annotate screenshots (arrows/circles on key elements)
- [ ] ⚙ Fallback: Use existing screenshots (dashboard.png, escrows.png, swap.png) — less targeted but acceptable

### Track B: Research & Validation
**⏱ Timebox**: 20 min
**✅ Checkpoint**: Post content reviewed by user

#### Phase B1: Editorial review (15 min)
- [ ] Check all URLs are correct (daglock.com, api.daglock.com, github.com)
- [ ] Verify test wallet addresses match the TestnetPage.tsx
- [ ] Verify the mock TX ID pattern works
- [ ] Check that the 64-char hex example is exactly 64 chars
- [ ] ⚙ Fallback: Test each link manually with curl

#### Phase B2: Community guidelines check (5 min)
- [ ] Ensure no self-promotion rules violated (r/kaspa is generally welcoming of Kaspa ecosystem projects)
- [ ] Verify post doesn't contain financial advice or guarantees
- [ ] ⚙ Fallback: Remove any potentially promotional language, focus purely on "try this tool"

---

## Posting Strategy Reference (from pre-announcement plan)

From `.opencode/plans/pre-announcement.md`:
1. ~~Reddit (r/kaspa) — text post, detailed~~ ← THIS TASK
2. Telegram Kaspa main chat — brief, link to Reddit
3. KRC-20 token groups (GHOST, NACHO, KASPY, etc.) — tailored per group
4. Twitter/X thread — with embedded demo video, 1 hour later
5. Discord Kaspa Builders — same content as Telegram

---

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Reddit blocks the post as spam | Post from an account with some karma; avoid link-heavy formatting |
| Post gets buried | Post during peak r/kaspa hours (14:00-16:00 UTC weekdays) |
| Test wallets get abused | They're public test wallets already listed on the site — no real funds at risk |
| Users confused by dev mode | Clearly label testnet + MockVerifier upfront with a warning banner |
| Screenshots show empty states | Run indexer locally and create escrows before capturing |
