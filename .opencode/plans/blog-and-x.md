# Plan: Blog Posts + X Content for DagLock

> **Goal:** Publish original content on `daglock.com/blog` and X/Twitter to build awareness of new features.
>
> **Key constraint:** All messaging must be ORIGINAL — no copying OfficeForge's "physically can't steal" tagline.
>
> **Status:** Plan created. No implementation yet.

---

## Message Pillars (Our Voice, Not OfficeForge's)

| Don't Say | Say Instead |
|-----------|-------------|
| "Physically can't steal" | "The covenant enforces the rules — not us. No admin keys, no backdoors." |
| "Trustless" (overused) | "Self-executing. The SilverScript code defines every possible outcome." |
| "Non-custodial" (jargon) | "We never hold your funds. The Kaspa network holds them." |
| "Killer feature" (hype) | Specific: "The only KRC-20 escrow on Kaspa" / "First AI mediator in crypto escrow" |

**The DagLock voice:** Technical but clear. No hype. Let the code speak.

---

## Track A: Blog Infrastructure `[ ]`

**Description:** Add a `/blog` route to the DagLock website with static markdown-rendered posts. No CMS needed — just markdown files compiled into the app.

**Timebox:** 2-3 days

### Phase A1: Route + Layout `[ ]` [1 day]
- [ ] Add route `/blog` and `/blog/:slug` to `App.tsx`
- [ ] Create `BlogPage.tsx` — lists all posts with title, date, excerpt
- [ ] Create `BlogPost.tsx` — renders a single post with markdown
- [ ] Add "Blog" link to sidebar nav
- [ ] Style: clean reading layout, code highlighting for SilverScript snippets

### Phase A2: Markdown rendering `[ ]` [1 day]
- [ ] Install a lightweight markdown renderer (or write a simple one — just needs headers, code blocks, bold, links)
- [ ] Post data: simple array of `{ slug, title, date, excerpt, content }` objects in a single file
- [ ] Each post is a TS/JS string (or `.md` file loaded at build time)
- ✅ **Checkpoint:** `/blog` shows post list, `/blog/slug` shows rendered post
- ⚙ **Fallback:** Static HTML files served from a `/blog/` directory

---

## Track B: Blog Posts `[ ]`

**Description:** Write 4 original blog posts with our messaging, not OfficeForge's.

**Timebox:** 1-2 hours per post (writing + editing)

### Phase B1: Post 1 — "KRC-20 Token Escrow is Here" `[ ]` [2 hrs]
- **Target audience:** KRC-20 token founders, Kaspa traders
- **Unique angle:** "DagLock is the only platform where you can escrow KRC-20 tokens. No one else has built this."
- **Structure:**
  1. The problem: KRC-20 tokens need trust to trade. Chat-group guarantors charge 3-10%.
  2. The solution: A SilverScript covenant that escrows both KAS and KRC-20 tokens.
  3. How ICC works (simplified): The covenant verifies token ownership via on-chain verification.
  4. Supported features: Milestone payments, subscriptions, atomic swaps, AI mediation.
  5. Call to action: Try it on testnet, deploy your token, create your first escrow.
- **Keywords:** KRC-20 escrow, Kaspa token trading, covenant escrow, SilverScript

### Phase B2: Post 2 — "AI Mediation for Escrow Disputes" `[ ]` [2 hrs]
- **Target audience:** Crypto traders, OTC desks
- **Unique angle:** "First AI mediator in crypto escrow — resolves disputes in minutes, not weeks."
- **Structure:**
  1. The problem: Escrow disputes are slow. Jury systems take days. Arbitration costs more than the trade.
  2. The solution: Non-binding AI mediation. The AI reads encrypted chat evidence, proposes a fair split.
  3. How it works (privacy-focused): E2E encrypted chat, party reveals key during dispute, AI analyzes, human arbiter if needed.
  4. Why it's safe: AI never touches money. Covenant caps where funds can go. Chat key can't move funds.
  5. Call to action: Create an escrow, test the dispute flow on testnet.
- **Keywords:** AI dispute resolution, crypto escrow, decentralized justice, Kaspa

### Phase B3: Post 3 — "What We Built: The Full DagLock Feature Set" `[ ]` [1 hr]
- **Target audience:** New visitors, developers evaluating the platform
- **Unique angle:** Catalog post showing breadth of features. "One platform, 12+ covenant types."
- **Structure:**
  - Bullet-point format with screenshots or diagrams
  - Sections: Escrow types (basic, milestone, subscription, multi-party), Vaults (time-locked, inheritance, check-in), Chat (E2E encrypted, on-chain anchored), AI mediator, API/Widget/SDK
  - Comparison table: DagLock vs chat-group guarantors (fees, security, features)
- **Keywords:** Kaspa DeFi, Kaspa escrow platform, SilverScript covenants

### Phase B4: Post 4 — "How Silverscript Covenants Enable Trustless Trading" `[ ]` [2-3 hrs]
- **Target audience:** Developers, technical Kaspa community
- **Unique angle:** Technical deep-dive into how covenants work, aimed at developers who want to build on Kaspa.
- **Structure:**
  1. What is a covenant? (UTXO-based smart contract)
  2. The DagLock escrow covenant, line by line (explain `checkSig`, `tx.time`, `sha256`)
  3. ICC pattern for KRC-20 tokens (how covenant-ID ownership works)
  4. Security properties (no admin keys, fixed fee, capped outcomes, dust protection)
  5. GitHub link, compile API, invite to contribute
- **Keywords:** SilverScript, Kaspa covenants, UTXO smart contracts, KRC-20 ICC

---

## Track C: X/Twitter Content `[ ]`

**Description:** Daily posting schedule to build a following @DagLock. No budget — organic only.

**Timebox:** 15 min/day

### Phase C1: Account Setup `[ ]` [1 hr]
- [ ] Create @DagLock handle if not already done
- [ ] Bio: "Trustless escrow & atomic swaps on Kaspa. KRC-20 support. AI mediation. Open source."
- [ ] Profile pic: DagLock logo
- [ ] Header: Dashboard screenshot or covenant diagram
- [ ] Link: daglock.com

### Phase C2: Launch Week Thread (Day 1) `[ ]` [1 hr]
A 10-tweet thread introducing DagLock:
```
1/ We built the first KRC-20 token escrow on Kaspa.
Not a promise. A SilverScript covenant.
Here's how it works 🧵

2/ DagLock is a covenant-based escrow protocol.
When you create an escrow, your funds are locked in a Kaspa UTXO.
The covenant defines every possible outcome.
There is no "send funds to admin" path. By design.

3/ KRC-20 tokens are supported from day one.
No other escrow platform on Kaspa can say this.
Your tokens stay in your custody until the covenant releases them.

4/ Disputes? We built an AI mediator.
It reads the encrypted chat, analyzes both sides' claims, and proposes a fair split in minutes.
If both parties accept, it's resolved. If not, a human jury votes.

5/ 12+ covenant types for different use cases:
• Standard escrow (KAS + KRC-20)
• Milestone payments (up to 5 stages)
• Recurring subscriptions
• Multi-party (up to 4 parties)
• Time-locked vaults with inheritance

6/ Every message is E2E encrypted.
Message hashes are anchored on Kaspa.
During a dispute, you can reveal the chat key to the jury — read-only, can't move funds.

7/ Want to add escrow to your site?
Drop in a <daglock-pay> tag. No redirect. No custody.
Or use our API, CLI, or Telegram bot (35+ commands).

8/ Everything is open source. Audited. No admin keys.
Try it: daglock.com
GitHub: github.com/dilljens/DagLock

9/ Questions? Feedback? We're here.
Follow @DagLock for updates.
RT/favorite if you found this useful 🙏
```

### Phase C3: Daily Posts (Weeks 1-4) `[ ]` [15 min/day]
| Day | Content |
|-----|---------|
| 1 | Launch thread (above) |
| 2 | Screenshot of the swap wizard + "Atomic swaps in 6 steps" |
| 3 | Short clip of the "Try to break it" security page |
| 4 | Feature highlight: AI mediator ("Settle disputes in minutes, not days") |
| 5 | Feature highlight: E2E encrypted chat + on-chain anchoring |
| 6 | Retweet/reply to Kaspa ecosystem accounts (Rock the Kaspa, KaspaCurrency) |
| 7 | Weekly roundup: "This week on DagLock" — user stats, new features |
| 8 | Feature highlight: Milestone payments for freelancers |
| 9 | Screenshot of the analytics dashboard (`/stats`) |
| 10 | Feature highlight: Telegram bot commands |
| 11 | Thread: "How SilverScript covenants work" (simplified) |
| 12 | Retweet/reply to KRC-20 token projects |
| 13 | Feature highlight: Time-locked vaults with inheritance |
| 14 | Weekly roundup |
| 15-28 | Repeat pattern with different angles |

### Phase C4: Engage With Kaspa Community `[ ]` [10 min/day]
- Reply to @KaspaCurrency tweets (add value, don't shill)
- Reply to @RockTheKaspa tweets
- Reply to questions about KRC-20 tokens with relevant DagLock features
- **Never** post price speculation, moon memes, or "wen token"

---

## Execution Strategy

```
Priority 1 (Foundation):
  Track A — Blog infrastructure (2-3 days)
  Track B1-B2 — Posts 1 + 2 (4 hrs writing)

Priority 2 (Social):
  Track C — X account setup + launch thread (Day 1)
  Track C3 — Daily posting (15 min/day ongoing)

Priority 3 (Content library):
  Track B3-B4 — Posts 3 + 4 (3-4 hrs writing)
```

---

## Anti-scope

- No Medium republishing (unless explicitly requested)
- No paid promotions
- No price/market commentary
- No copying competitor language
- No newsletter (not yet — too early)

---

## Files to Create/Modify

| Track | Files |
|-------|-------|
| A | `web/src/pages/BlogPage.tsx` (NEW), `web/src/pages/BlogPost.tsx` (NEW), `web/src/App.tsx` (routes), `web/src/layout/Sidebar.tsx` (nav link) |
| B | `web/src/content/blog-posts.ts` (NEW — post data) |
| C | X/Twitter account only — no code changes |
