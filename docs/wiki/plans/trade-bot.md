# Trade Bot — Tier 1: Offer Board Population

**Status:** Not built

**Date:** 2026-06-17

**Goal:** Make the testnet offer board look active with automated offers from two bot identities.

---

## Architecture

```
systemd timer (every 10 min)
  └── trade-bot.py (Python)
        ├── Bot_A (kaspa:addr_a)
        ├── Bot_B (kaspa:addr_b)
        └── POST /v1/*  →  https://api.daglock.com
```

No on-chain signing needed. No wallets. Just HTTP requests.

---

## What the Bot Does

Each cycle (every 10 minutes), the bot picks from these actions randomly:

| Action | API call | Effect |
|--------|----------|--------|
| Create offer | `POST /v1/offers` | Adds a new offer to the board |
| Accept offer | `POST /v1/offers/:id/accept` | Removes an offer, creates an escrow |
| Cancel old offer | `POST /v1/offers/:id/cancel` | Cleans up offers older than 1 hour |
| Link identity | `POST /v1/identity` | Links a Telegram handle for reputation |
| Create escrow (direct) | `POST /v1/escrows` | Escrow from bot to bot (no offer) |

---

## Bot Identities

```
Bot_A: kaspa:q...a  (buyer persona — always buying)
Bot_B: kaspa:q...b  (seller persona — always selling)
```

Each has a Telegram handle for reputation:
- `@trader_alice`
- `@trader_bob`

---

## Offer Diversity

The bot should create varied offers so the board doesn't look stale:

| Asset pair | Side | Amount range |
|-----------|------|-------------|
| KAS → KRC20:NACHO | buy | 100-5000 KAS |
| KAS → KRC20:NACHO | sell | 100-5000 KAS |
| KAS → KRC20:GHOST | buy | 100-5000 KAS |
| KAS → KRC20:GHOST | sell | 100-5000 KAS |
| KAS → KAS (escrow service) | buy | 500-10000 KAS |

Random price offsets: ±2% from last known price (or use a fixed range).

---

## Open Cycle Count

Target: **5-10 active offers** at any time.

If total offers < 5, create a new one. If total > 15, cancel oldest.

---

## Implementation (Python)

Already has `urllib` built-in. No pip dependencies needed.

```python
#!/usr/bin/env python3
"""Trade bot — populates the offer board with automated activity."""
import json, os, random, secrets, time, urllib.error, urllib.request

API_URL = os.environ.get("API_URL", "http://127.0.0.1:8443")

BOT_A = "kaspa:q..."  # from genkeys.py
BOT_B = "kaspa:q..."

def post(path, body, headers=None):
    url = f"{API_URL}{path}"
    data = json.dumps(body).encode()
    req = urllib.request.Request(url, data=data, method="POST")
    req.add_header("Content-Type", "application/json")
    if headers:
        for k, v in headers.items():
            req.add_header(k, v)
    try:
        with urllib.request.urlopen(req, timeout=10) as r:
            return json.loads(r.read())
    except urllib.error.HTTPError as e:
        return {"_error": f"HTTP {e.code}"}

def get(path):
    try:
        with urllib.request.urlopen(f"{API_URL}{path}", timeout=10) as r:
            return json.loads(r.read())
    except urllib.error.HTTPError as e:
        return {"_error": f"HTTP {e.code}"}

def create_offer(creator, side, base, quote, amount_kas):
    return post("/v1/offers", {
        "creator_address": creator,
        "side": side,
        "base_asset": base,
        "quote_asset": quote,
        "amount_sompi": int(amount_kas * 100_000_000),
    })

def accept_offer(offer_id, counterparty):
    return post(f"/v1/offers/{offer_id}/accept", {
        "counterparty_address": counterparty,
    })

def cancel_offer(offer_id):
    return post(f"/v1/offers/{offer_id}/cancel", {})

def main():
    # 1. Check current offer count
    offers = get("/v1/offers?status=proposed")
    count = len(offers.get("offers", []))

    # 2. Clean up old offers (older than 1 hour)
    for o in offers.get("offers", []):
        age = time.time() - o.get("created_at", 0)
        if age > 3600:
            cancel_offer(o["id"])

    # 3. If too few offers, create some
    if count < 5:
        # Create 2-3 new offers
        assets = ["KAS", "KRC20:NACHO", "KRC20:GHOST"]
        for _ in range(random.randint(2, 3)):
            side = random.choice(["buy", "sell"])
            base = random.choice(assets)
            quote = random.choice([a for a in assets if a != base])
            amount = random.randint(100, 5000)
            creator = random.choice([BOT_A, BOT_B])
            create_offer(creator, side, base, quote, amount)

    # 4. If there are offers, maybe accept one
    if count >= 2 and random.random() < 0.3:
        target = random.choice(offers["offers"])
        acceptor = BOT_A if target["creator_address"] == BOT_B else BOT_B
        accept_offer(target["id"], acceptor)

if __name__ == "__main__":
    main()
```

---

## Systemd Timer

```ini
# /etc/systemd/system/daglock-trade-bot.timer
[Unit]
Description=DagLock Trade Bot — every 10 minutes

[Timer]
OnUnitActiveSec=10min
Unit=daglock-trade-bot.service

[Install]
WantedBy=timers.target
```

```ini
# /etc/systemd/system/daglock-trade-bot.service
[Unit]
Description=DagLock Trade Bot
After=network.target

[Service]
Type=oneshot
ExecStart=/opt/daglock-trade-bot/trade-bot.py
WorkingDirectory=/opt/daglock-trade-bot
Environment=API_URL=http://127.0.0.1:8443
```

---

## Setup Steps

1. Generate two Kaspa addresses for the bot identities
2. (Optional) Copy the script to the VPS
3. Set up systemd timer
4. Verify it runs: `systemctl start daglock-trade-bot.service && journalctl -u daglock-trade-bot -f`

---

## Future: Tier 2

Tier 2 adds real on-chain escrows. See implementation notes in the section below.

### What Tier 2 adds
- Real escrow creation (fund covenant → broadcast tx → register with indexer)
- Settlement with auth signatures
- Refunds
- Atomic swaps
- Reputation scores for the bot addresses

### What it needs
- Private keys for the bot addresses
- Rust binary using `kaspa-txscript` + `kaspa-wrpc-client` for transaction signing
- A few hundred testnet KAS pre-funded to each address

### Architecture
```
scripts/trade-bot-rs/
├── Cargo.toml
├── src/main.rs
├── src/scenarios.rs
└── src/signing.rs
```
