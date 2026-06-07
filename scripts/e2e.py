#!/usr/bin/env python3
"""DagLock End-to-End Test — exercises all surfaces against a running indexer.

Usage:
  python3 scripts/e2e.py                          # assumes indexer on :8543
  API_URL=http://my-indexer:8543 python3 scripts/e2e.py
  python3 scripts/e2e.py --start-indexer          # starts + stops its own
"""

import argparse, json, os, secrets, subprocess, sys, time, urllib.error, urllib.request, uuid

PASS = FAIL = 0
INDEXER_PID = None
GREEN = "\033[0;32m"; RED = "\033[0;31m"; CYAN = "\033[0;36m"; YELLOW = "\033[1;33m"; NC = "\033[0m"
API_URL = os.environ.get("API_URL", "http://localhost:8543")

def status(m): print(f"\n{YELLOW}== {m} =={NC}")
def passed(m): global PASS; PASS += 1; print(f"  {GREEN}PASS{NC} {m}")
def failed(m): global FAIL; FAIL += 1; print(f"  {RED}FAIL{NC} {m}")
def info(m): print(f"  {CYAN}->{NC} {m}")

def addr():
    c = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"
    return "kaspa:q" + "".join(c[secrets.randbelow(32)] for _ in range(35))

def rid(): return str(uuid.uuid4()).split("-")[0]

def call(path, method="GET", body=None, headers=None):
    url = f"{API_URL}{path}"
    data = json.dumps(body).encode() if body else None
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Content-Type", "application/json")
    if headers:
        for k, v in headers.items(): req.add_header(k, v)
    try:
        with urllib.request.urlopen(req, timeout=15) as r:
            return r.status, json.loads(r.read())
    except urllib.error.HTTPError as e:
        try: return e.code, json.loads(e.read())
        except: return e.code, {"_error": str(e)}
    except Exception as e: return 0, {"_error": str(e)}

def main():
    global INDEXER_PID
    p = argparse.ArgumentParser()
    p.add_argument("--api-url", default=API_URL)
    p.add_argument("--start-indexer", action="store_true")
    args = p.parse_args()
    globals()["API_URL"] = args.api_url

    if args.start_indexer:
        subprocess.run(["cargo", "build", "-p", "daglock-indexer"], capture_output=True)
        proc = subprocess.Popen(["cargo", "run", "-p", "daglock-indexer", "--",
            "--database-url", "sqlite:e2e_test.db", "--host", "127.0.0.1", "--port", "8543"],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        INDEXER_PID = proc.pid
        for i in range(15):
            try:
                if call("/v1/health")[0] == 200: break
            except: pass
            time.sleep(1)

    status("1. Preflight")
    c, d = call("/v1/health")
    if c == 200 and d.get("status") == "ok":
        passed(f"Health check (uptime {d.get('uptime_seconds', 0)}s)")
    else:
        failed(f"Health: HTTP {c}")
        sys.exit(1)

    status("2. Escrow Lifecycle")
    b1, b2 = addr(), addr()
    c, d = call("/v1/escrows", "POST", {"lock_tx_id": secrets.token_hex(16),
        "lock_tx_output_index": 0, "buyer_address": b1, "seller_address": b2,
        "amount_sompi": 50_000_000_000, "asset_type": "KAS"})
    if c == 201:
        eid = d["id"]
        passed(f"Created escrow: {eid[:16]}... (HTTP {c})")
        subprocess.run(["sqlite3", os.path.join(os.getcwd(), "e2e_test.db"),
            f"UPDATE escrows SET status='active' WHERE id='{eid}'"],
            capture_output=True, timeout=5)
        c, d = call(f"/v1/escrows/{eid}")
        passed(f"Lookup: HTTP {c} (status={d.get('status')})") if c == 200 else failed("Lookup")
        c, _ = call(f"/v1/escrows/{eid}/settle", "POST", {},
            {"X-Daglock-Address": b2, "X-Daglock-Signature": "x", "X-Daglock-Message": f"settle:{eid}"})
        passed(f"Settle: HTTP {c}") if c == 200 else failed(f"Settle: {c}")
        c, d = call(f"/v1/receipts/{eid}")
        passed(f"Receipt: {d.get('receipt_id','?')[:16]}...") if c == 200 else failed("Receipt")
    else:
        failed(f"Create escrow: HTTP {c}")
        sys.exit(1)

    status("3. Offers")
    c, d = call("/v1/offers", "POST", {"creator_address": addr(),
        "side": "sell", "base_asset": "KAS", "quote_asset": "KRC20:NACHO", "amount_sompi": 100_000_000_000})
    passed(f"Offer created: HTTP {c}") if c == 201 else failed(f"Offer: {c}")
    c, d = call("/v1/offers?status=proposed")
    if c == 200 and d.get("total", 0) > 0:
        passed(f"Offer board: {d['total']} offer(s)")
        oid = d["offers"][0]["id"]
        c, _ = call(f"/v1/offers/{oid}/accept", "POST", {"counterparty_address": addr()})
        passed(f"Accept: HTTP {c}") if c in (200,201) else failed("Accept")

    status("4. Reputation")
    c, d = call(f"/v1/reputation/{b1}")
    if c == 200:
        passed(f"Buyer: {d['score']:.2f}/5 ({d['trade_count']} trades)")
    c, d = call("/v1/stats")
    if c == 200 and d.get("total_escrows", 0) >= 1:
        passed(f"Stats: {d['total_escrows']} escrows")

    status("5. Telegram Identity")
    a = addr(); h = f"@e2e_{rid()[:8]}"; m = f"daglock.io:verify:telegram:{h}"
    c, d = call("/v1/identity", "POST", {"platform": "telegram", "handle": h,
        "signed_message": m, "signature_hex": "x"},
        {"X-Daglock-Address": a, "X-Daglock-Signature": "x", "X-Daglock-Message": m})
    passed(f"Identity: HTTP {c} ({d.get('status','?')})") if c == 200 else failed("Identity")
    c, d = call(f"/v1/reputation/{a}")
    if c == 200 and d.get("telegram_handle") == h:
        passed(f"Handle in reputation: {h}")

    status("6. Evidence + Messages")
    b3, s3 = addr(), addr()
    c, d = call("/v1/escrows", "POST", {"lock_tx_id": secrets.token_hex(16),
        "lock_tx_output_index": 0, "buyer_address": b3, "seller_address": s3,
        "amount_sompi": 200_000_000_000, "asset_type": "KAS"})
    e2 = d.get("id", "")
    if e2:
        subprocess.run(["sqlite3", os.path.join(os.getcwd(), "e2e_test.db"),
            f"UPDATE escrows SET status='active' WHERE id='{e2}'"], capture_output=True, timeout=5)
        c, _ = call(f"/v1/escrows/{e2}/dispute", "POST", {"reason": "test"},
            {"X-Daglock-Address": b3, "X-Daglock-Signature": "x", "X-Daglock-Message": f"dispute:{e2}"})
        passed(f"Dispute: HTTP {c}") if c in (200,201) else failed(f"Dispute: {c}")
        c, _ = call(f"/v1/escrows/{e2}/evidence", "POST", {"content": "No delivery"},
            {"X-Daglock-Address": b3, "X-Daglock-Signature": "x", "X-Daglock-Message": f"evidence:{e2}"})
        passed(f"Evidence: HTTP {c}") if c in (200,201) else failed(f"Evidence: {c}")
        c, _ = call(f"/v1/escrows/{e2}/messages", "POST", {"content": "Hello from buyer"},
            {"X-Daglock-Address": b3, "X-Daglock-Signature": "x", "X-Daglock-Message": f"msg:{e2}"})
        passed(f"Message: HTTP {c}") if c in (200,201) else failed(f"Message: {c}")
    else:
        failed("Escrow for evidence")

    status("7. Build Check")
    r = subprocess.run(["cargo", "build", "-p", "daglock-cli"], capture_output=True, timeout=120)
    passed(f"CLI build: exit {r.returncode}") if r.returncode == 0 else failed("CLI build")
    r = subprocess.run(["node", "--check", "src/index.js"], capture_output=True, timeout=10, cwd="bot")
    passed(f"Bot syntax: exit {r.returncode}") if r.returncode == 0 else failed("Bot syntax")

    status("SUMMARY")
    t = PASS + FAIL
    print(f"  {GREEN}Passed: {PASS}{NC}")
    print(f"  {RED}Failed: {FAIL}{NC}")
    print(f"  Total:  {t}")
    if FAIL == 0:
        print(f"\n  {GREEN}ALL E2E CHECKS PASSED{NC}")
    else:
        print(f"\n  {YELLOW}{FAIL} failure(s) — review above{NC}")

    if INDEXER_PID:
        os.kill(INDEXER_PID, 15)
    for f in ["e2e_test.db", "e2e_test.db-wal", "e2e_test.db-shm"]:
        if os.path.exists(f): os.remove(f)
    sys.exit(FAIL)

if __name__ == "__main__":
    main()
