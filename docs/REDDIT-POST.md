# Reddit Post — DagLock Testnet Launch

**Subreddit:** r/kaspa
**Draft date:** June 5, 2026
**Tone:** Direct, transparent, zero hype

---

## Title Option 1 (Straightforward)

> **DagLock — trustless escrow for KAS and KRC-20. Testnet is live on TN12. No signups, no KYC, no middleman.**

## Title Option 2 (Problem-Focused)

> **Trading KRC-20 P2P? DagLock lets you escrow KAS and tokens on Kaspa L1 without trusting a middleman. Testnet is live.**

## Title Option 3 (Short)

> **DagLock testnet is live — trustless escrow on Kaspa TN12**

---

## Body

**What is DagLock?**

Trustless escrow and atomic swaps on Kaspa L1 via SilverScript covenants.

You lock KAS or KRC-20 tokens into a UTXO governed by a covenant. Release is only possible when both parties sign, or a hash preimage is revealed, or a timeout expires. No admin keys, no upgrade mechanism, no custodial risk. The covenant is the only thing that controls the funds.

**Does it work right now?**

Yes — on Testnet 12. Toccata activates on mainnet around June 30. That's when DagLock's covenants become deployable on mainnet. Until then, everything is testnet with fake KAS.

**Try it in 30 seconds:**

1. Go to https://test.daglock.com
2. Create an offer (sell KAS for KRC-20, or whatever)
3. Check reputation of an address
4. Browse open offers

No wallet needed to browse. If you want to sign transactions, DM me for the test wallet private key or use the mock signature (any hex string works on testnet).

**Or via CLI:**

```bash
cargo install --git https://github.com/dilljens/DagLock daglock-cli
daglock-cli --api-url https://test-api.daglock.com offer list
daglock-cli --api-url https://test-api.daglock.com reputation kaspa:qdyzkrhd74v6cetrv4fhv
```

**What I need from you:**

- Try to break it. Create offers, dispute them, message escrows.
- Tell me if something is confusing or doesn't work.
- What feature would make you actually use this on mainnet?

**What's in the pipeline:**

- Telegram bot (@DagLock_test_bot already up for testing)
- Volume-based fee tiers (0.5% standard, down to 0.15% for whales)
- Jury-based dispute resolution
- Atomic swap wizard (abstracts the hash preimage stuff)

**Links:**

- Web UI: https://test.daglock.com
- API: https://test-api.daglock.com
- Repo: https://github.com/dilljens/DagLock
- Telegram bot: https://t.me/DagLock_test_bot

---

## Optional: Sticky Comment

> A few things to be clear about:
>
> - **This is testnet.** No real KAS. You can't lose money.
> - **I'm one dev.** If something breaks, give me a day to fix it.
> - **The fee is 0.5%, hardcoded in the covenant.** I can't change it after deployment, I can't steal funds, I can't upgrade it. That's the whole point.
> - **Mainnet launches when Toccata activates.** Not before.
>
> Questions? Drop them here or open an issue on GitHub.
