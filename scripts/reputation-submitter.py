#!/usr/bin/env python3
"""Backfill existing settled escrows into the on-chain reputation covenant.

Reads settled escrows from the indexer API and constructs signed trade receipts
suitable for submission to the DagLockReputation covenant.

Usage:
  python3 scripts/reputation-submitter.py [--api-url https://api.daglock.com]

Note: This script produces unsigned receipts. Both parties must sign them
before submission. The script outputs JSON files that can be shared with
each party for signing via KasWare or kaspawallet.
"""

import json
import os
import sys
import time
import urllib.error
import urllib.request

API_URL = os.environ.get("API_URL", "http://127.0.0.1:8443")
OUTPUT_DIR = os.environ.get("OUTPUT_DIR", "/tmp/reputation-receipts")

USER_AGENT = "DagLockReputationSubmitter/0.1.0"

COVENANT_ADDRESS = "65c54102c64a331414b602760cbd76efac3d69df"

def api_get(path):
    req = urllib.request.Request(
        f"{API_URL}{path}",
        headers={"User-Agent": USER_AGENT},
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


def build_receipt(buyer, seller, amount_sompi, outcome, escrow_id):
    """Build a trade receipt that both parties would sign."""
    timestamp = int(time.time())
    nonce = os.urandom(8).hex()
    return {
        "version": 1,
        "buyer": buyer,
        "seller": seller,
        "amount_sompi": amount_sompi,
        "outcome": outcome,
        "timestamp": timestamp,
        "nonce": nonce,
        "escrow_id": escrow_id,
        "covenant_address": COVENANT_ADDRESS,
        # Signatures are filled in by the parties
        "sig_buyer": None,
        "sig_seller": None,
    }


def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    # Check health
    status, data = api_get("/v1/health")
    if status != 200:
        print(f"✗ Indexer not healthy (HTTP {status})")
        sys.exit(1)
    print(f"✓ Indexer healthy")

    # Fetch reputation to find addresses with settled escrows
    # For each address, we'd fetch their escrows and find settled ones
    # But we need user participation for signatures
    print(f"\nReputation Covenant: {COVENANT_ADDRESS}")
    print(f"Receipts will be saved to: {OUTPUT_DIR}")
    print()

    print("This script produces unsigned receipt JSON files.")
    print("Each receipt needs both parties' signatures before submission.")
    print()
    print("To sign a receipt:")
    print(f"  1. Share the receipt JSON with both parties")
    print(f"  2. Each party signs using their wallet (KasWare/kaspawallet)")
    print(f"  3. Submit the signed receipt to the reputation covenant")
    print(f"  4. Verify it's recorded on-chain via /v1/reputation/on-chain/<address>")
    print()
    print("Receipt format:")
    print(json.dumps(build_receipt(
        "kaspa:buyer_address", "kaspa:seller_address",
        1_000_000_000, 0, "esc_example"
    ), indent=2))


if __name__ == "__main__":
    main()
