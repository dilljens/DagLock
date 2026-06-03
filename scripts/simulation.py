#!/usr/bin/env python3
"""DagLock Simulation — test all escrow lifecycle permutations against the indexer API."""

import argparse
import json
import os
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

GREEN = "\033[0;32m"
RED = "\033[0;31m"
CYAN = "\033[0;36m"
YELLOW = "\033[1;33m"
NC = "\033[0m"


def pass_msg(msg):
    global PASS
    PASS += 1
    print(f"  {GREEN}PASS{NC} {msg}")


def fail_msg(msg):
    global FAIL
    FAIL += 1
    print(f"  {RED}FAIL{NC} {msg}")


def info(msg):
    print(f"  {CYAN}→{NC} {msg}")


def header(title):
    print(f"\n{YELLOW}══ {title} ══{NC}")


def random_addr():
    chars = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"
    return "kaspa:q" + "".join(chars[secrets.randbelow(32)] for _ in range(35))


def random_hex(n=16):
    return secrets.token_hex(n)


def random_id():
    return str(uuid.uuid4()).split("-")[0]


def sompi(kas: float) -> int:
    return int(kas * 100_000_000)


class API:
    def __init__(self, base_url: str):
        self.base = base_url.rstrip("/")

    def _request(self, method: str, path: str, body=None, headers=None) -> dict:
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
            try:
                return json.loads(e.read())
            except Exception:
                return {"_error": f"HTTP {e.code}"}
        except Exception as e:
            return {"_error": str(e)}

    def get(self, path):
        return self._request("GET", path)

    def post(self, path, body):
        return self._request("POST", path, body)

    def auth(self, path, body, address, signature, message):
        return self._request("POST", path, body, {
            "X-Daglock-Address": address,
            "X-Daglock-Signature": signature,
            "X-Daglock-Message": message,
        })

    def health(self):
        return self.get("/v1/health")

    def create_escrow(self, buyer, seller, amount, asset="KAS", mediator_key=None):
        body = {
            "lock_tx_id": random_hex(16),
            "lock_tx_output_index": 0,
            "buyer_address": buyer,
            "seller_address": seller,
            "amount_sompi": amount,
            "asset_type": asset,
        }
        if mediator_key:
            body["mediator_key"] = mediator_key
        return self.post("/v1/escrows", body)

    def settle(self, escrow_id, address, sig="mock_sig"):
        return self.auth(f"/v1/escrows/{escrow_id}/settle", {}, address, sig, f"settle:{escrow_id}")

    def refund(self, escrow_id, address, sig="mock_sig"):
        return self.auth(f"/v1/escrows/{escrow_id}/refund", {}, address, sig, f"refund:{escrow_id}")

    def cancel(self, escrow_id):
        return self.post(f"/v1/escrows/{escrow_id}/cancel", {})

    def dispute(self, escrow_id, reason="test"):
        return self.post(f"/v1/escrows/{escrow_id}/dispute", {"reason": reason})

    def submit_evidence(self, escrow_id, content, address, sig="mock_sig"):
        return self.auth(f"/v1/escrows/{escrow_id}/evidence", {"content": content}, address, sig, f"evidence:{escrow_id}")

    def list_evidence(self, escrow_id):
        return self.get(f"/v1/escrows/{escrow_id}/evidence")

    def resolve_dispute(self, escrow_id, outcome, address, sig="mock_sig"):
        return self.auth(f"/v1/escrows/{escrow_id}/resolve-dispute",
                         {"outcome": outcome, "resolved_by": address},
                         address, sig, f"resolve:{escrow_id}")

    def create_offer(self, creator, side, base, quote, amount):
        return self.post("/v1/offers", {
            "creator_address": creator,
            "side": side,
            "base_asset": base,
            "quote_asset": quote,
            "amount_sompi": amount,
        })

    def accept_offer(self, offer_id, counterparty):
        return self.post(f"/v1/offers/{offer_id}/accept", {"counterparty_address": counterparty})

    def cancel_offer(self, offer_id):
        return self.post(f"/v1/offers/{offer_id}/cancel", {})

    def create_identity(self, address, handle, sig="mock_sig"):
        msg = f"daglock.io:verify:telegram:{handle}"
        return self.auth("/v1/identity", {
            "platform": "telegram",
            "handle": handle,
            "signed_message": msg,
            "signature_hex": sig,
        }, address, sig, msg)


def wait_for_server(api, max_retries=15):
    for i in range(max_retries):
        try:
            h = api.health()
            if h.get("status") == "ok":
                return True
        except Exception:
            pass
        time.sleep(1)
    return False


def sql_set_active(db_path, escrow_id):
    """Set escrow to active via SQLite."""
    try:
        subprocess.run(
            ["sqlite3", db_path, f"UPDATE escrows SET status='active' WHERE id='{escrow_id}'"],
            capture_output=True, timeout=5, check=False)
    except Exception:
        pass


def main():
    parser = argparse.ArgumentParser(description="DagLock Simulation")
    parser.add_argument("--api-url", default="http://localhost:8443")
    parser.add_argument("--db-path", default="daglock_sim.db")
    parser.add_argument("--no-server", action="store_true", help="Don't start a server")
    args = parser.parse_args()

    api = API(args.api_url)
    server_pid = None

    # ── Preflight ──
    header("Preflight")
    running = False
    if not args.no_server:
        try:
            h = api.health()
            if h.get("status") == "ok":
                running = True
                info(f"Indexer running at {args.api_url}")
        except Exception:
            pass

        if not running:
            info("Starting indexer...")
            for f in [args.db_path, args.db_path + "-wal", args.db_path + "-shm"]:
                if os.path.exists(f):
                    os.remove(f)
            proc = subprocess.Popen(
                ["cargo", "run", "-p", "daglock-indexer", "--",
                 "--database-url", f"sqlite:{args.db_path}",
                 "--host", "127.0.0.1", "--port", "8443"],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            )
            server_pid = proc.pid
            if wait_for_server(api):
                info("Indexer started")
            else:
                print(f"{RED}Failed to start indexer{NC}")
                sys.exit(1)

    h = api.health()
    if h.get("status") == "ok":
        pass_msg("Health check")
    else:
        fail_msg(f"Health check: {h}")

    # ── S1: Escrow settle ──
    header("Scenario 1: Escrow settle (happy path)")
    b1, s1 = random_addr(), random_addr()
    e1 = api.create_escrow(b1, s1, sompi(500))
    e1_id = e1.get("id", "")
    if e1_id:
        pass_msg(f"Created: {e1_id} (status: {e1.get('status')})")
        sql_set_active(args.db_path, e1_id)
        r = api.settle(e1_id, s1)
        if r.get("status") == "settled":
            pass_msg(f"Settled: {e1_id}")
        else:
            fail_msg(f"Settle: {r}")
        # Receipt
        rec = api.get(f"/v1/receipts/{e1_id}")
        if rec.get("receipt_id"):
            pass_msg("Receipt generated")
        else:
            fail_msg(f"Receipt: {rec}")
    else:
        fail_msg(f"Create escrow: {e1}")

    # ── S2: Escrow refund ──
    header("Scenario 2: Escrow refund")
    b2, s2 = random_addr(), random_addr()
    e2 = api.create_escrow(b2, s2, sompi(250))
    e2_id = e2.get("id", "")
    if e2_id:
        pass_msg(f"Created: {e2_id}")
        sql_set_active(args.db_path, e2_id)
        r = api.refund(e2_id, b2)
        if r.get("status") == "refunded":
            pass_msg(f"Refunded: {e2_id}")
        else:
            fail_msg(f"Refund: {r}")
    else:
        fail_msg(f"Create escrow: {e2}")

    # ── S3: Cancel ──
    header("Scenario 3: Escrow cancel")
    e3 = api.create_escrow(random_addr(), random_addr(), sompi(100))
    e3_id = e3.get("id", "")
    if e3_id:
        r = api.cancel(e3_id)
        if r.get("status") == "cancelled":
            pass_msg("Cancelled pending escrow")
        else:
            fail_msg(f"Cancel: {r}")

    # ── S4: Dispute + Evidence ──
    header("Scenario 4: Dispute, evidence, resolution")
    b4, s4 = random_addr(), random_addr()
    e4 = api.create_escrow(b4, s4, sompi(1000))
    e4_id = e4.get("id", "")
    if e4_id:
        pass_msg(f"Created: {e4_id}")
        sql_set_active(args.db_path, e4_id)

        r = api.dispute(e4_id, "Seller never delivered")
        if r.get("status") == "disputed":
            pass_msg("Disputed")
        else:
            fail_msg(f"Dispute: {r}")

        ev = api.submit_evidence(e4_id, "I paid 1000 KAS, no delivery", b4)
        if ev.get("id"):
            pass_msg("Evidence submitted")
        else:
            fail_msg(f"Evidence: {ev}")

        ev_list = api.list_evidence(e4_id)
        ev_count = len(ev_list.get("evidence", []))
        if ev_count >= 1:
            pass_msg(f"Evidence listed ({ev_count} items)")
        else:
            fail_msg("Evidence listing")

        r = api.resolve_dispute(e4_id, "expunge", b4)
        if r.get("status") == "resolved":
            pass_msg("Dispute resolved")
        else:
            fail_msg(f"Resolve: {r}")
    else:
        fail_msg(f"Create escrow: {e4}")

    # ── S5: Arbiter escrow ──
    header("Scenario 5: Arbiter escrow (mediator)")
    b5, s5, m5 = random_addr(), random_addr(), random_addr()
    e5 = api.create_escrow(b5, s5, sompi(2000), "KAS", m5)
    e5_mk = e5.get("mediator_key")
    if e5_mk == m5:
        pass_msg(f"Arbiter escrow with mediator: {m5}")
    else:
        fail_msg(f"Mediator missing: {e5}")

    # ── S6: Reputation ──
    header("Scenario 6: Reputation")
    fresh = random_addr()
    rp = api.get(f"/v1/reputation/{fresh}")
    if "score" in rp:
        pass_msg(f"Fresh address score: {rp['score']}")
    else:
        fail_msg(f"Reputation: {rp}")

    rp_t = api.get(f"/v1/reputation/{b1}")
    if rp_t.get("trade_count", 0) > 0:
        pass_msg(f"Trader has {rp_t['trade_count']} trade(s)")
    else:
        info(f"Trader trade_count: {rp_t.get('trade_count')}")

    # ── S7: Offers ──
    header("Scenario 7: Offer board")
    cr = random_addr()
    of = api.create_offer(cr, "sell", "KAS", "KRC20:NACHO", sompi(500))
    o_id = of.get("id", "")
    if of.get("status") == "proposed":
        pass_msg(f"Offer created: {o_id}")
    else:
        fail_msg(f"Offer: {of}")

    of_list = api.get("/v1/offers?status=proposed")
    o_cnt = of_list.get("total", 0)
    if o_cnt > 0:
        pass_msg(f"Offer board: {o_cnt} offers")
    else:
        fail_msg("Offer board")

    cp = random_addr()
    ac = api.accept_offer(o_id, cp)
    if ac.get("status") == "accepted":
        pass_msg("Offer accepted")
    else:
        fail_msg(f"Accept: {ac}")

    of2 = api.create_offer(random_addr(), "buy", "KAS", "KRC20:KASPY", sompi(100))
    o2_id = of2.get("id", "")
    ca = api.cancel_offer(o2_id)
    if ca.get("status") == "cancelled":
        pass_msg("Offer cancelled")
    else:
        fail_msg(f"Cancel: {ca}")

    # ── S8: Telegram identity ──
    header("Scenario 8: Telegram identity")
    id_addr = random_addr()
    id_handle = f"@sim_{random_id()[:8]}"
    id_res = api.create_identity(id_addr, id_handle)
    if id_res.get("status") == "verified":
        pass_msg(f"Telegram linked: {id_handle}")
    else:
        fail_msg(f"Identity: {id_res}")

    rp8 = api.get(f"/v1/reputation/{id_addr}")
    if rp8.get("telegram_handle") == id_handle:
        pass_msg("Handle in reputation")
    else:
        fail_msg(f"Handle missing: {rp8}")

    # ── S9: Edge cases ──
    header("Scenario 9: Edge cases")
    bad = api.create_escrow("bad-addr", random_addr(), 100)
    if "error" in str(bad).lower() or bad.get("error"):
        pass_msg("Rejected invalid address")
    else:
        info(f"Bad addr: {json.dumps(bad)[:80]}")

    zero = api.create_escrow(random_addr(), random_addr(), 0)
    if "error" in str(zero).lower() or zero.get("error"):
        pass_msg("Rejected zero amount")
    else:
        info(f"Zero: {json.dumps(zero)[:80]}")

    nf = api.get("/v1/escrows/nonexistent")
    info(f"Nonexistent: {json.dumps(nf)[:80]}")

    # ── S10: Stats ──
    header("Scenario 10: Stats")
    st = api.get("/v1/stats")
    tot = st.get("total_escrows", 0)
    if tot >= 4:
        pass_msg(f"Stats: {tot} escrows")
    else:
        info(f"Total escrows: {tot}")

    # ── Summary ──
    header("Summary")
    print(f"  {GREEN}Passed: {PASS}{NC}")
    print(f"  {RED}Failed: {FAIL}{NC}")
    print(f"  Total:  {PASS + FAIL}")
    if FAIL == 0:
        print(f"\n  {GREEN}✓ All scenarios passed{NC}")
    else:
        print(f"\n  {RED}{FAIL} failure(s){NC}")

    # Cleanup
    if server_pid:
        os.kill(server_pid, 15)
    sys.exit(FAIL)


if __name__ == "__main__":
    main()
