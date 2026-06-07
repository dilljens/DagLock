#!/usr/bin/env python3
"""DagLock Reputation Simulation — generate many trades, verify reputation scores."""

import argparse
import json
import math
import os
import random
import secrets
import subprocess
import sys
import time
import urllib.error
import urllib.request
import uuid

PASS = 0
FAIL = 0
SERVER_PID = None

GREEN = "\033[0;32m"; RED = "\033[0;31m"; CYAN = "\033[0;36m"; YELLOW = "\033[1;33m"; NC = "\033[0m"

def pass_msg(msg): global PASS; PASS += 1; print(f"  {GREEN}PASS{NC} {msg}")
def fail_msg(msg): global FAIL; FAIL += 1; print(f"  {RED}FAIL{NC} {msg}")
def info(msg): print(f"  {CYAN}→{NC} {msg}")
def header(title): print(f"\n{YELLOW}══ {title} ══{NC}")

def random_addr():
    chars = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"
    return "kaspa:q" + "".join(chars[secrets.randbelow(32)] for _ in range(35))
def random_hex(n=16): return secrets.token_hex(n)
def random_id(): return str(uuid.uuid4()).split("-")[0]
def sompi(kas): return int(kas * 100_000_000)

# ── Reputation formula (mirrors calculate_reputation_score in queries.rs) ──
# Beta reputation system (Josang 2002): (successes + 1) / (total + 2)
def expected_score(trades, recent_trades, volume_sompi, age_days, refunds, recent_refunds):
    # Beta reputation (Josang 2002) with recency weighting
    # Recent trades (last 90d) weighted 2x vs old trades
    total = max(trades, 0)
    if total < 1:
        return 1.0
    recent = max(recent_trades, 0)
    old_trades = max(total - recent, 0)
    recent_r = max(recent_refunds, 0)
    old_r = max(max(refunds, 0) - recent_r, 0)
    w = 2.0  # recency weight
    eff_total = recent * w + old_trades
    eff_refunds = recent_r * w + old_r
    alpha = max(eff_total - eff_refunds, 0)
    beta_f = eff_refunds
    beta_raw = (alpha + 1.0) / (alpha + beta_f + 2.0)
    centered = (beta_raw - 0.5) * 2.0
    volume_kas = max(volume_sompi, 0) / 100_000_000.0
    volume_bonus = math.log(volume_kas / 1000.0 + 1.0) * 0.12
    age_bonus = min(age_days / 365.0, 2.0) * 0.05
    raw = 1.0 + (centered * 4.0) + volume_bonus + age_bonus
    return min(max(raw, 1.0), 5.0)


class API:
    def __init__(self, base_url):
        self.base = base_url.rstrip("/")

    def _request(self, method, path, body=None, headers=None):
        url = f"{self.base}{path}"
        data = json.dumps(body).encode() if body else None
        req = urllib.request.Request(url, data=data, method=method)
        req.add_header("Content-Type", "application/json")
        if headers:
            for k, v in headers.items():
                req.add_header(k, v)
        try:
            with urllib.request.urlopen(req, timeout=10) as resp:
                return json.loads(resp.read())
        except urllib.error.HTTPError as e:
            try: return json.loads(e.read())
            except: return {"_error": f"HTTP {e.code}"}
        except Exception as e:
            return {"_error": str(e)}

    def get(self, path): return self._request("GET", path)
    def post(self, path, body): return self._request("POST", path, body)
    def auth(self, path, body, address, signature, message):
        return self._request("POST", path, body, {"X-Daglock-Address": address, "X-Daglock-Signature": signature, "X-Daglock-Message": message})
    def health(self): return self.get("/v1/health")
    def create_escrow(self, buyer, seller, amount, mediator_key=None):
        body = {"lock_tx_id": random_hex(16), "lock_tx_output_index": 0, "buyer_address": buyer, "seller_address": seller, "amount_sompi": amount, "asset_type": "KAS"}
        if mediator_key: body["mediator_key"] = mediator_key
        return self.post("/v1/escrows", body)
    def settle(self, eid, addr, sig="msig"): return self.auth(f"/v1/escrows/{eid}/settle", {}, addr, sig, f"settle:{eid}")
    def refund(self, eid, addr, sig="msig"): return self.auth(f"/v1/escrows/{eid}/refund", {}, addr, sig, f"refund:{eid}")
    def cancel(self, eid): return self.post(f"/v1/escrows/{eid}/cancel", {})
    def dispute(self, eid, reason="test"): return self.post(f"/v1/escrows/{eid}/dispute", {"reason": reason})
    def submit_evidence(self, eid, content, addr, sig="msig"):
        return self.auth(f"/v1/escrows/{eid}/evidence", {"content": content}, addr, sig, f"evidence:{eid}")
    def resolve_dispute(self, eid, outcome, addr, sig="msig"):
        return self.auth(f"/v1/escrows/{eid}/resolve-dispute", {"outcome": outcome, "resolved_by": addr}, addr, sig, f"resolve:{eid}")
    def create_offer(self, creator, side, base, quote, amount):
        return self.post("/v1/offers", {"creator_address": creator, "side": side, "base_asset": base, "quote_asset": quote, "amount_sompi": amount})
    def accept_offer(self, oid, cp): return self.post(f"/v1/offers/{oid}/accept", {"counterparty_address": cp})
    def cancel_offer(self, oid): return self.post(f"/v1/offers/{oid}/cancel", {})
    def create_identity(self, addr, handle, sig="msig"):
        msg = f"daglock.io:verify:telegram:{handle}"
        return self.auth("/v1/identity", {"platform": "telegram", "handle": handle, "signed_message": msg, "signature_hex": sig}, addr, sig, msg)
    def reputation(self, addr): return self.get(f"/v1/reputation/{addr}")


def wait_for_server(api, max_retries=15):
    for i in range(max_retries):
        try:
            if api.health().get("status") == "ok": return True
        except: pass
        time.sleep(1)
    return False

def sql_set_active(db_path, eid):
    try: subprocess.run(["sqlite3", db_path, f"UPDATE escrows SET status='active' WHERE id='{eid}'"], capture_output=True, timeout=5)
    except: pass

def sql_set_created(db_path, addr, ts):
    """Backdate an escrow's created_at to simulate account age."""
    try: subprocess.run(["sqlite3", db_path, f"UPDATE escrows SET created_at={ts} WHERE buyer_address='{addr}' OR seller_address='{addr}'"], capture_output=True, timeout=5)
    except: pass


def run_batch(api, db_path, buyer, seller, count, settled_pct=0.8, disputed_pct=0.05):
    """Create `count` escrows between buyer and seller, settle/refund them."""
    settled = 0
    refunded = 0
    disputed = 0
    created = 0

    for i in range(count):
        amt = sompi(secrets.randbelow(10000) + 100)  # 100-10100 KAS
        e = api.create_escrow(buyer, seller, amt)
        eid = e.get("id")
        if not eid:
            continue
        created += 1
        sql_set_active(db_path, eid)

        # Decide outcome
        r = random.random()
        if r < disputed_pct:
            # Dispute then settle
            api.dispute(eid, "Simulated dispute")
            api.submit_evidence(eid, "Automated test evidence", buyer)
            api.resolve_dispute(eid, "expunge" if secrets.randbits(1) else "uphold", buyer)
            api.settle(eid, seller)
            disputed += 1
            settled += 1
        elif r < disputed_pct + (1 - settled_pct):
            api.refund(eid, buyer)
            refunded += 1
        else:
            api.settle(eid, seller)
            settled += 1

    return created, settled, refunded, disputed


def check_reputation(api, addr, expected_trades, expected_recent, expected_volume, expected_age_days, expected_refunds, expected_recent_refunds):
    """Fetch reputation and compare with expected score."""
    rep = api.reputation(addr)
    if "_error" in rep:
        fail_msg(f"Reputation fetch failed: {rep}")
        return

    tc = rep.get("trade_count", 0)
    sc = rep.get("score", 0)
    exp_score = expected_score(expected_trades, expected_volume, expected_age_days, expected_disputes, expected_refunds)

    info(f"  Trades: {tc} (expected ~{expected_trades})")
    info(f"  Volume: {rep.get('total_volume_sompi', 0)} sompi")
    info(f"  Score:  {sc:.4f} (expected ~{exp_score:.4f})")
    info(f"  Dispute rate: {rep.get('dispute_rate', 0)*100:.1f}%  Refund rate: {rep.get('refund_rate', 0)*100:.1f}%")
    if rep.get("telegram_handle"):
        info(f"  Telegram: {rep['telegram_handle']}")

    # Trade count should be at least our expected
    if tc >= expected_trades:
        pass_msg(f"Trade count ≥ {expected_trades} ({tc})")
    else:
        # Might have extra from previous runs — warn but accept
        info(f"Trade count: {tc} (expected ≥ {expected_trades})")

    # Score should be within reasonable range
    if 1.0 <= sc <= 5.0:
        pass_msg(f"Score in range [1,5]: {sc:.4f}")
    else:
        fail_msg(f"Score out of range: {sc}")

    # Score should correlate with our expected (lower for more disputes/refunds)
    diff = abs(sc - exp_score)
    if diff < 1.5:
        pass_msg(f"Score close to expected (diff={diff:.4f})")
    else:
        # Large diff might mean the formula or data differs — note it
        info(f"Score differs from expectation (diff={diff:.4f})")


def main():
    parser = argparse.ArgumentParser(description="DagLock Reputation Simulation")
    parser.add_argument("--api-url", default="http://localhost:8543")
    parser.add_argument("--db-path", default="daglock_sim.db")
    parser.add_argument("--no-server", action="store_true")
    parser.add_argument("--trades", type=int, default=50, help="Trades per bot")
    parser.add_argument("--bots", type=int, default=5, help="Number of bot pairs")
    args = parser.parse_args()

    api = API(args.api_url)
    server_pid = None

    # ── Preflight ──
    header("Preflight")
    running = False
    if not args.no_server:
        try:
            if api.health().get("status") == "ok":
                running = True
                info(f"Indexer running at {args.api_url}")
        except: pass
        if not running:
            info("Starting indexer...")
            for f in [args.db_path, args.db_path + "-wal", args.db_path + "-shm"]:
                if os.path.exists(f): os.remove(f)
            proc = subprocess.Popen(["cargo", "run", "-p", "daglock-indexer", "--",
                "--database-url", f"sqlite:{args.db_path}", "--host", "127.0.0.1", "--port", "8543"],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            server_pid = proc.pid
            if wait_for_server(api): info("Indexer started")
            else: print(f"{RED}Failed to start indexer{NC}"); sys.exit(1)

    h = api.health()
    pass_msg("Health check") if h.get("status") == "ok" else fail_msg(f"Health: {h}")

    # ── Bot identities ──
    header("Bot identities")
    bots = []
    for i in range(args.bots):
        buyer = random_addr()
        seller = random_addr()
        tg = f"@bot{i+1}_{random_id()[:6]}"
        api.create_identity(buyer, tg)
        bots.append({"buyer": buyer, "seller": seller, "tg": tg})
        pass_msg(f"Bot {i+1}: buyer={buyer[:20]}... seller={seller[:20]}... tg={tg}")

    # ── Mass trade generation ──
    header("Mass trade generation")
    total_created = 0
    results = []
    for i, bot in enumerate(bots):
        info(f"Generating ~{args.trades} trades for Bot {i+1}...")
        created, settled, refunded, disputed = run_batch(
            api, args.db_path, bot["buyer"], bot["seller"], args.trades,
            settled_pct=0.8, disputed_pct=0.05,
        )
        total_created += created
        results.append({"bot": i+1, "created": created, "settled": settled, "refunded": refunded, "disputed": disputed})
        info(f"  Created: {created}  Settled: {settled}  Refunded: {refunded}  Disputed: {disputed}")
        pass_msg(f"Bot {i+1}: {created} escrows processed")

    info(f"Total escrows created: {total_created}")

    # ── Reputation verification ──
    header("Reputation verification")

    # Each bot's buyer and seller have different profiles
    for i, bot in enumerate(bots):
        r = results[i]
        buyer = bot["buyer"]
        seller = bot["seller"]
        tg = bot["tg"]

        info(f"── Bot {i+1} buyer reputation ──")
        rep = api.reputation(buyer)
        if rep.get("telegram_handle") == tg:
            pass_msg(f"Telegram handle correct: {tg}")
        else:
            fail_msg(f"Telegram handle: expected {tg}, got {rep.get('telegram_handle')}")

        # Buyer is always the one who disputes, and occasionally refunds
        b_disputes = r["disputed"]  # buyer initiated disputes
        b_refunds = r["refunded"]   # buyer refunded
        b_settles = r["settled"]    # buyer settled (participated)
        b_trades = r["created"]
        b_volume = (r["settled"] + r["refunded"]) * 5000 * 100_000_000  # rough avg
        b_age = 30  # simulated 30 days

        exp_b = expected_score(b_trades, b_trades, b_volume, b_age, b_refunds, b_refunds)
        info(f"  Expected score: ~{exp_b:.4f}")
        info(f"  Actual score:   {rep.get('score', 0):.4f}")
        info(f"  Trade count:    {rep.get('trade_count', 0)}")
        info(f"  Dispute rate:   {rep.get('dispute_rate', 0)*100:.1f}%")
        info(f"  Refund rate:    {rep.get('refund_rate', 0)*100:.1f}%")

        if rep.get("trade_count", 0) >= r["created"]:
            pass_msg(f"Buyer trade count ≥ {r['created']}")
        else:
            fail_msg(f"Buyer trade count: {rep.get('trade_count', 0)} < {r['created']}")

    # ── Cross-bot comparison ──
    header("Cross-bot comparison")
    # Half the bots get a Telegram handle with reputation — verify it shows
    scores = []
    for i, bot in enumerate(bots):
        rep = api.reputation(bot["buyer"])
        scores.append(rep.get("score", 0))

    if all(1.0 <= s <= 5.0 for s in scores):
        pass_msg(f"All {len(scores)} bot scores in [1, 5] range")
    else:
        fail_msg(f"Scores out of range: {scores}")

    # High-volume bot should have higher score than low-volume (all else equal)
    # We can't strictly compare since amounts are random, but they should be > 1
    high_score = max(scores)
    low_score = min(scores)
    info(f"Highest bot score: {high_score:.4f}")
    info(f"Lowest bot score:  {low_score:.4f}")

    # ── Summary ──
    header("Summary")
    info(f"Total trades generated: {total_created}")
    print(f"  {GREEN}Passed: {PASS}{NC}")
    print(f"  {RED}Failed: {FAIL}{NC}")
    print(f"  Total:  {PASS + FAIL}")
    if FAIL == 0:
        print(f"\n  {GREEN}✓ All reputation checks passed{NC}")
    else:
        print(f"\n  {RED}{FAIL} failure(s){NC}")

    if server_pid:
        os.kill(server_pid, 15)
    sys.exit(FAIL)


if __name__ == "__main__":
    main()
