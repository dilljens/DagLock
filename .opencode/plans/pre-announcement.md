# Pre-Announcement Checklist

> **Status:** All automated work done (v2.0.1 SDK bump, deploy, wiki, drafts fixed).
> **Remaining:** Human-only tasks. Target: June 25-28 before mainnet activation.

---

## 1. Deploy the v2.0.1 Binary

Already SCP'd to VPS. One command:

```bash
ssh root@46.224.171.239
systemctl stop daglock-indexer
cp /root/daglock-indexer /usr/local/bin/daglock-indexer
systemctl start daglock-indexer
curl http://127.0.0.1:8443/v1/health   # verify
```

---

## 2. Pre-Announcement Checklist (2-3 hours)

### 2.1 Fresh-user walkthrough (20 min)
Open incognito window, go to daglock.com, pretend you've never seen it.
- Note what confuses you
- Is the connect-wallet flow obvious?
- Does the onboarding modal help or obscure?

### 2.2 Friend test (30 min) — most valuable
Send a friend the link. Sit next to them. **Don't help.**
- Watch where they get stuck
- Listen to what they say out loud
- Fix the top 3 friction points

### 2.3 Bot error messages (15 min)
Open @DagLock_bot. Deliberately trigger errors:
- `/create` with empty amount
- Wrong address format
- Expired offer
- Empty fields
Are the error messages helpful or cryptic?

### 2.4 Health check (2 min)
```bash
curl https://api.daglock.com/v1/health
# Should return <200ms with status: "ok"
```

### 2.5 Faucet link (5 min)
Go through onboarding flow (or just look at the modal). Is the testnet KAS faucet link obvious? Click it — does it still work?

### 2.6 Mobile test (15 min)
Open daglock.com on your phone.
- Can you create an escrow?
- Do buttons overlap?
- Is the sidebar usable?

### 2.7 README review (15 min)
Visit `github.com/dilljens/DagLock`. Does the README reflect:
- v2.0.1 SDK?
- MockVerifier/dev-mode note?
- Correct testnet faucet URL?

### 2.8 Support plan (20 min)
Decide before posting anything:
- Where do bug reports go? (GitHub issues? Telegram DM? Telegram group?)
- Who responds on day 1?
- What's the SLA? ("I'm one dev, expect 24h response")
- Consider creating a `@DagLock` Telegram group or channel for community

### 2.9 Launch time (5 min)
Pick an exact date+time. Considerations:
- Weekday morning UTC (avoid weekends)
- Avoid BTC events, other Kaspa announcements
- Good: Mon-Thu between 14:00-16:00 UTC (catches Europe afternoon + US morning)
- June 29-30 would land right on Toccata activation buzz

---

## 3. Content Creation (1.5-2 hours)

### 3.1 Demo video (1 hour)
30-60 second screen capture. Script is in `docs/announcement-drafts.md`.
- Record with OBS, QuickTime, or phone
- Show: daglock.com → connect KasWare → create escrow → show pending → mention explorer
- Since MockVerifier won't auto-detect, say: *"On-chain confirmation verified via the explorer during dev mode"*
- Upload to Twitter/X natively (not as a link)

### 3.2 Announcement drafts final review (30 min)
Files ready to go:
- `docs/announcement-drafts.md` — Telegram, Twitter/X thread, Discord
- `docs/REDDIT-POST.md` — r/kaspa post

Read them end-to-end. Add your personal voice. Make sure:
- URLs are right (daglock.com, api.daglock.com, github.com/dilljens/DagLock)
- Bot handle is @DagLock_bot (not _test_bot)
- Dev-mode transparency note is included
- Date references say "June 30" not old dates

---

## 4. Announcement Day

Post in this order for maximum reach:

```
1. Reddit (r/kaspa) — text post, detailed
2. Telegram Kaspa main chat — brief, link to Reddit
3. KRC-20 token groups (GHOST, NACHO, KASPY, etc.) — tailored per group
4. Twitter/X thread — with embedded demo video, 1 hour later
5. Discord Kaspa Builders — same content as Telegram
```

After posting:
- Monitor bot DMs for bug reports
- Watch indexer logs: `journalctl -u daglock-indexer --since "5 min ago" -f`
- Reply to comments/questions for the first few hours
