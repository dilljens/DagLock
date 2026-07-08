#!/usr/bin/env python3
"""DagLock Trade Bot — populates the testnet offer board with automated activity.

Deploy on VPS alongside the indexer. Runs via systemd timer every 10 minutes.

Two bot identities take turns creating offers, accepting them, and cleaning up.

Tier 1: No on-chain interaction. Pure API calls with mock auth signatures.
"""

import hashlib
import hmac
import json
import os
import random
import secrets
import time
import urllib.error
import urllib.request

API_URL = os.environ.get("API_URL", "http://127.0.0.1:8443")
MIN_OFFERS = 3
MAX_OFFERS = 12
OFFER_TTL = 3600  # 1 hour — cancel offers older than this

# ── Bot identities (generated via scripts/genkeys.py) ──────────────────────
# Bot A — buyer persona
BOT_A_PK = "88cd1916bc7500aa4b4b26145e2fc1a742681d1965e3006fa47a3ec644ed8440"
BOT_A_ADDR = "kaspa:qmkfju948q0fc5ttte5nk"
BOT_A_HANDLE = "@trader_alice"

# Bot B — seller persona
BOT_B_PK = "b4dd418b199b8f2c0ac604c7977f9d0d4c0c763ed8fa8ea281a613387e102896"
BOT_B_ADDR = "kaspa:qjevqxrqetdkc9qj6yswe"
BOT_B_HANDLE = "@trader_bob"

# Map address → private key for signing
PRIVKEYS = {BOT_A_ADDR: BOT_A_PK, BOT_B_ADDR: BOT_B_PK}
BOTS = [BOT_A_ADDR, BOT_B_ADDR]

# ── Asset pairs for offer variety ─────────────────────────────────────────
ASSET_PAIRS = [
    ("KAS", "KAS"),           # KAS escrow service → "KAS Escrow" badge
    ("KAS", "KAS"),           # (weighted 2x for more KAS escrows)
    ("KAS", "KRC20:NACHO"),   # KAS for NACHO → "Atomic Swap" badge
    ("KAS", "KRC20:GHOST"),   # KAS for GHOST → "Atomic Swap" badge
    ("KAS", "KRC20:KASPY"),   # KAS for KASPY → "Atomic Swap" badge
]

# ── Signing (compatible with SchnorrVerifier / mock auth) ─────────────────

def sign_message(privkey_hex, message):
    """Sign a message with the mock-auth-compatible HMAC-SHA256 scheme."""
    msg_hash = hashlib.sha256(message.encode()).digest()
    sig = hmac.new(
        bytes.fromhex(privkey_hex),
        msg_hash,
        hashlib.sha256,
    ).digest() + msg_hash[:32]
    return sig.hex()


def auth_headers(address, message):
    """Build X-Daglock-* auth headers for a given address and message."""
    privkey = PRIVKEYS.get(address)
    if not privkey:
        return {}
    return {
        "User-Agent": "DagLockTradeBot/0.1.0",
        "Content-Type": "application/json",
        "X-Daglock-Address": address,
        "X-Daglock-Signature": sign_message(privkey, message),
        "X-Daglock-Message": message,
    }


# ── HTTP helpers ──────────────────────────────────────────────────────────

def api_post(path, body, headers=None):
    """POST to the indexer API."""
    url = f"{API_URL}{path}"
    data = json.dumps(body).encode()
    req = urllib.request.Request(url, data=data, method="POST", headers=headers or {})
    try:
        with urllib.request.urlopen(req, timeout=10) as r:
            return r.status, json.loads(r.read())
    except urllib.error.HTTPError as e:
        try:
            return e.code, json.loads(e.read())
        except Exception:
            return e.code, {"_error": str(e)}
    except Exception as e:
        return 0, {"_error": str(e)}


def api_get(path):
    """GET from the indexer API."""
    req = urllib.request.Request(
        f"{API_URL}{path}",
        headers={"User-Agent": "DagLockTradeBot/0.1.0"},
    )
    try:
        with urllib.request.urlopen(req, timeout=10) as r:
            return 200, json.loads(r.read())
    except urllib.error.HTTPError as e:
        try:
            return e.code, json.loads(e.read())
        except Exception:
            return e.code, {"_error": str(e)}
    except Exception as e:
        return 0, {"_error": str(e)}


# ── Bot actions ──────────────────────────────────────────────────────────

def create_offer(creator, side, base, quote, amount_kas):
    """Create an offer on the board with auth."""
    body = {
        "creator_address": creator,
        "side": side,
        "base_asset": base,
        "quote_asset": quote,
        "amount_sompi": int(amount_kas * 100_000_000),
        "creator_type": "bot",  # Self-identify as automated trading bot
    }
    nonce = secrets.token_hex(8)
    ts = int(time.time())
    message = f"create_offer:{creator}:{ts}:{nonce}"
    headers = auth_headers(creator, message)
    status, data = api_post("/v1/offers", body, headers)
    if status == 201:
        oid = data.get("id", "?")
        print(f"  ✓ Created offer {oid}: {side} {amount_kas} {base} for {quote} by {creator[:16]}...")
        return oid
    else:
        err = data.get("error", data.get("_error", ""))
        print(f"  ✗ Failed to create offer: HTTP {status} {err}")
        return None


def accept_offer(offer_id, counterparty):
    """Accept an offer as the counterparty with auth."""
    body = {"counterparty_address": counterparty}
    nonce = secrets.token_hex(8)
    ts = int(time.time())
    message = f"accept_offer:{offer_id}:{ts}:{nonce}"
    headers = auth_headers(counterparty, message)
    status, data = api_post(f"/v1/offers/{offer_id}/accept", body, headers)
    if status in (200, 201):
        print(f"  ✓ Accepted offer {offer_id} by {counterparty[:16]}...")
        return True
    else:
        err = data.get("error", data.get("_error", ""))
        print(f"  ✗ Failed to accept offer {offer_id}: HTTP {status} {err}")
        return False


def cancel_offer(offer_id, creator):
    """Cancel an offer with auth."""
    nonce = secrets.token_hex(8)
    ts = int(time.time())
    message = f"cancel_offer:{offer_id}:{ts}:{nonce}"
    headers = auth_headers(creator, message)
    status, data = api_post(f"/v1/offers/{offer_id}/cancel", {}, headers)
    if status in (200, 201, 204):
        print(f"  ✓ Cancelled offer {offer_id}")
        return True
    else:
        err = data.get("error", data.get("_error", ""))
        print(f"  ✗ Failed to cancel offer {offer_id}: HTTP {status} {err}")
        return False


def link_identity(addr, handle):
    """Link a Telegram handle to an address for reputation with auth.

    The identity endpoint requires the exact message format:
    'daglock.io:verify:telegram:<handle>' — no timestamps or nonces.
    """
    msg = f"daglock.io:verify:telegram:{handle}"
    headers = auth_headers(addr, msg)
    body = {
        "platform": "telegram",
        "handle": handle,
        "signed_message": msg,
        "signature_hex": headers.get("X-Daglock-Signature", "x"),
    }
    status, data = api_post("/v1/identity", body, headers)
    if status == 200:
        print(f"  ✓ Linked {handle} → {addr[:16]}...")
        return True
    else:
        err = data.get("error", data.get("_error", ""))
        print(f"  ✗ Failed to link identity: HTTP {status} {err}")
        return False


def random_amount(base_asset):
    """Pick a realistic trade amount based on asset type."""
    if base_asset == "KAS":
        return random.randint(1, 5000)
    else:
        return random.randint(100, 10000)


# ── Rate-limit safety ────────────────────────────────────────────────
# Wait between API calls to avoid hammering the rate limiter (30 req/min).
# Each cycle does at most ~15 calls. At 1 call/500ms that's ~7.5s burst.
BATCH_DELAY = 0.5  # seconds between API calls


def api_sleep():
    """Pause briefly between API calls to respect rate limits."""
    time.sleep(BATCH_DELAY)


# ── Main cycle ─────────────────────────────────────────────────────────

def main():
    print(f"\n=== DagLock Trade Bot — {time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())} ===")

    # ── 1. Health check ──
    status, data = api_get("/v1/health")
    if status != 200 or data.get("status") != "ok":
        print(f"  ✗ Indexer not healthy (HTTP {status}). Skipping cycle.")
        return

    # ── 2. Fetch current offers ──
    # Always do this first so we have an accurate view of the board.
    api_sleep()
    status, data = api_get("/v1/offers?status=proposed&limit=50")
    if status != 200:
        print(f"  ✗ Failed to fetch offers (HTTP {status}). Skipping.")
        return

    offers = data.get("offers", [])
    active_count = len(offers)
    now = time.time()
    print(f"  Current offers: {active_count}")

    # Track how many of our offers are on the board
    our_offers = [o for o in offers if o.get("creator_address", "") in PRIVKEYS]

    # ── 3. Cancel stale offers (>1 hour old) ──
    stale_ids = []
    for offer in our_offers:
        age = now - offer.get("created_at", now)
        if age > OFFER_TTL:
            stale_ids.append(offer["id"])

    for oid in stale_ids:
        creator = next(
            (o["creator_address"] for o in our_offers if o["id"] == oid),
            BOT_A_ADDR,
        )
        api_sleep()
        if cancel_offer(oid, creator):
            active_count = max(0, active_count - 1)

    # ── 4. Accept a random offer (30% chance, need ≥2 on board) ──
    accepted = False
    if active_count >= 2 and random.random() < 0.3:
        # Pick any valid offer (not ours, not stale)
        accept_candidates = [
            o for o in offers
            if o.get("creator_address", "") not in PRIVKEYS
            and o.get("status") == "proposed"
            and now - o.get("created_at", now) < OFFER_TTL
        ]
        if accept_candidates:
            target = random.choice(accept_candidates)
            acceptor = random.choice(BOTS)
            api_sleep()
            if accept_offer(target["id"], acceptor):
                active_count = max(0, active_count - 1)
                accepted = True

    # ── 5. Create identity links (best-effort, once per run) ──
    # Uses the script's first run to link identities. Silently skips
    # on failure since the identity endpoint requires real Schnorr
    # signatures (HMAC mock signatures only work for offer operations).
    if not os.environ.get("TRADE_BOT_IDENTITY_DONE"):
        api_sleep()
        link_identity(BOT_A_ADDR, BOT_A_HANDLE)
        api_sleep()
        link_identity(BOT_B_ADDR, BOT_B_HANDLE)
        # Don't retry every cycle — mark as done
        os.environ["TRADE_BOT_IDENTITY_DONE"] = "1"

    # ── 6. Refill offers if board is low ──
    if active_count < MIN_OFFERS:
        if active_count == 0:
            print("  Board empty — creating minimum offer (1 KAS)")
            creator = random.choice(BOTS)
            side = random.choice(["buy", "sell"])
            base, quote = random.choice(ASSET_PAIRS)
            api_sleep()
            if create_offer(creator, side, base, quote, 1):
                active_count += 1

        # Create a few more to reach target range
        needed = MAX_OFFERS - active_count
        to_create = min(needed, random.randint(2, 4))
        for _ in range(to_create):
            creator = random.choice(BOTS)
            side = random.choice(["buy", "sell"])
            base, quote = random.choice(ASSET_PAIRS)
            amount = random_amount(base)
            api_sleep()
            if create_offer(creator, side, base, quote, amount):
                active_count += 1

    # ── 7. Cull if board is overfull ──
    if active_count > MAX_OFFERS and not accepted:
        excess = active_count - MAX_OFFERS
        # Cancel the oldest among our offers
        our_sorted = sorted(our_offers, key=lambda o: o.get("created_at", 0))
        for offer in our_sorted[:excess]:
            creator = offer.get("creator_address", "")
            api_sleep()
            if cancel_offer(offer["id"], creator):
                active_count = max(0, active_count - 1)

    print(f"  Done. Active offer count: ~{active_count}")
    print()


if __name__ == "__main__":
    main()
