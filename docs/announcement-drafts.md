# Announcement Drafts

> **Status:** Draft (not posted)
> **When to use:** After Phase 3 (end-to-end verification) passes
> **Target date:** ~June 18-20, 2026

---

## Telegram: Kaspa Main Chat

Post in [t.me/KaspaCurrency](https://t.me/KaspaCurrency). Keep it brief — this is a busy chat. Ask admin permission first.

```
🚀 DagLock — Trustless Escrow on Kaspa (Testnet Live)

We built what the Kaspa ecosystem has been missing: covenant-enforced escrow for KAS and KRC-20 tokens.

How it works:
1. Buyer and seller agree on terms
2. Funds locked in a SilverScript covenant (one UTXO per escrow)
3. Covenant enforces the rules — no admin keys, no backdoors
4. Settlement or refund, all trustless

What's live on testnet:
• Web dashboard: daglock.com
• Telegram bot: @DagLock_bot
• CLI: github.com/.../daglock
• Atomic swaps, vaults, jury dispute resolution

0.5% protocol fee enforced by the covenant itself.
Open source, audited (June 2026).

Try it, break it, tell us what you think. Testnet KAS from the faucet, then go.
```

---

## Telegram: KRC-20 Token Groups

Each token community has different norms. Tailor this. Draft for GHOST/NACHO/KASPY/etc:

```
👋 Hey [token] community

If you've ever done an OTC trade and had to trust the other person — DagLock fixes that.

Trustless escrow for KAS + KRC-20 tokens on Kaspa testnet:
• Bot: @DagLock_bot (/create to start)
• Web: daglock.com
• Your funds are locked in a covenant — nobody can steal them

Perfect for OTC trades between community members.
Testnet is live, mainnet coming June 30 (Toccata hard fork).

Would love feedback from anyone who tries it!
```

---

## Twitter/X

Thread (3-4 posts):

```
1/ We built trustless escrow for Kaspa. On testnet now, mainnet June 30.

DagLock lets you trade KAS and KRC-20 tokens without trusting a counterparty.
Funds locked in a SilverScript covenant — enforced by the protocol, not by us.

→ daglock.com
→ @DagLock_bot

2/ How it works:
• Create escrow → funds locked in covenant UTXO
• Counterparty settles → 0.5% fee to protocol treasury
• If something goes wrong → dispute via mediator or jury
• No admin keys. No backdoors. No custody.

3/ Built for the Kaspa community:
• Web dashboard (KasWare wallet)
• Telegram bot (16 commands + /create wizard)
• CLI for power users
• KRC-20 support from day one
• Atomic swap wizard
• On-chain reputation system

4/ Open source, audited (all 7 critical/high findings fixed).
Testnet is live. Try it, break it, tell us what's wrong.

🔗 daglock.com
🤖 @DagLock_bot
📄 github.com/.../daglock
```

---

## Discord: Kaspa Builders

```
Hey builders 👋

I've been working on DagLock — trustless escrow & atomic swaps on Kaspa L1 using SilverScript covenants.

Testnet is live and I'm looking for feedback before mainnet (June 30, same day as Toccata).

What works:
• KAS + KRC-20 escrow (covenant-enforced, 0.5% fee)
• Telegram bot (@DagLock_bot) with /create wizard
• Web dashboard (daglock.com) with KasWare integration
• Atomic swap wizard (hash preimage)
• Time-locked vaults (standard, softlock, multisig)
• Jury dispute resolution
• On-chain reputation + vouching

Stack: SilverScript contracts, Rust indexer, React web, Node.js bot.
Audited June 6 — all findings fixed.

Would love any feedback on:
• The UX flow (is it intuitive?)
• The covenant design (any edge cases?)
• The announcement messaging

Repo: github.com/.../daglock
Bot: @DagLock_bot
Web: daglock.com
```

---

## Demo Video Script

> Record 30-60 seconds. Screen capture of daglock.com + KasWare popup.

```
[0:00-0:05] Open daglock.com
"Meet DagLock — trustless escrow for Kaspa"

[0:05-0:10] Click "Connect Wallet" → KasWare approves
"Connect your KasWare wallet — one click"

[0:10-0:20] Go to Escrows → Create. Enter 10 KAS, paste seller address. Click Create.
"Create an escrow in seconds. Set the amount, pick your counterparty."

[0:20-0:30] KasWare pops up — shows covenant address. Approve.
"DagLock compiles a covenant, KasWare shows you exactly what you're signing."

[0:30-0:35] Escrow appears with "pending_confirmation" status.
"Once confirmed, the funds are locked in a covenant. Neither of us can steal them."

[0:35-0:45] Click Settle → KasWare signs → Status changes to "settled".
"When the trade completes, settlement needs both signatures."

[0:45-0:50] Settlement receipt page.
"Every trade produces a cryptographic receipt. Verifiable, exportable."

[0:50-0:60] Close with "Try it on testnet. Link in bio."
```

---

## Pre-Announcement Checklist

Use after Phase 3 passes but before posting anything:

- [ ] Try the web flow yourself as a first-time user
- [ ] Send a friend the link and watch them use it (no instructions)
- [ ] Check bot error messages are human-readable
- [ ] Verify the `/v1/health` endpoint is public and responds fast
- [ ] Make sure testnet KAS faucet link is easy to find (onboarding modal)
- [ ] Check that the web UI works on mobile (basic responsive check)
- [ ] Confirm the GitHub repo README is up to date
- [ ] Have a plan for responding to bug reports (Telegram group?)
- [ ] Decide on launch time (avoid weekends, aim for weekday morning)
