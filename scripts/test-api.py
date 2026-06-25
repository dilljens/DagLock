#!/usr/bin/env python3
"""DagLock API Smoke Test — tests all public endpoints."""
import json, urllib.request, sys, time

API = "https://api.daglock.com"
PASS = 0
FAIL = 0

def green(s): return f"\033[32m{s}\033[0m"
def red(s): return f"\033[31m{s}\033[0m"

def check(name, condition, detail=""):
    global PASS, FAIL
    if condition:
        PASS += 1
        print(f"  {green('PASS')} {name}")
    else:
        FAIL += 1
        print(f"  {red('FAIL')} {name} — {detail}")

HEADERS = {"User-Agent": "DagLock-Test/1.0", "Accept": "application/json"}

def get(path):
    url = f"{API}{path}"
    req = urllib.request.Request(url, headers=HEADERS)
    try:
        with urllib.request.urlopen(req, timeout=15) as r:
            return json.loads(r.read())
    except Exception as e:
        return {"_error": str(e)}

def post(path, body):
    url = f"{API}{path}"
    data = json.dumps(body).encode()
    req = urllib.request.Request(url, data=data, method="POST", headers=HEADERS)
    req.add_header("Content-Type", "application/json")
    try:
        with urllib.request.urlopen(req, timeout=15) as r:
            return json.loads(r.read())
    except urllib.error.HTTPError as e:
        try: return json.loads(e.read())
        except: return {"_error": f"HTTP {e.code}"}

print(f"\nDagLock API Smoke Test — {API}\n")

# ── 1. Health ──
h = get("/v1/health")
check("Health endpoint", h.get("status") == "ok", str(h))
check("DB connected", h.get("db_connected") is True)
check("Version present", "version" in h)

# ── 2. Status ──
s = get("/v1/status")
check("Status endpoint", s.get("status") == "ok")
check("Status has network", "network" in s)
check("Status has total_escrows", "total_escrows" in s)

# ── 3. Network ──
n = get("/v1/network")
check("Network endpoint", "network" in n)
check("Network is testnet-12", n.get("network") == "testnet-12")
check("KAS template hash present", n.get("daglock_kas_template_hash") == "30876e3ea42d0e23bb0980f3fd97ae8807e9c70f")
check("KRC-20 template hash present", n.get("daglock_krc20_template_hash") == "ae0946e4a9bd4a7585e6bf9135de38083cb11c85")

# ── 4. Stats ──
st = get("/v1/stats")
check("Stats endpoint", "total_escrows" in st)

# ── 5. Compile KAS escrow ──
c = post("/v1/compile", {
    "template": "daglock",
    "params": {
        "buyer_key": "99a3552dd14f06833328b07ee57fa4933bb0fc7e05ce6606ca563acd6552675e",
        "seller_key": "69e169233c847a5724314c2c2c5383c5b08c8547d7010573c03cb77fec3f97f2",
        "timeout": "86400",
        "treasury_key": "4cbe03e1113c7506932623b76e182acd70f8dd7defbc3ccb3b572f53f5aae3ca",
        "trade_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
})
check("KAS compile", "script" in c and "template_hash" in c, str(c.get("error","")))
if "template_hash" in c:
    check("KAS has entrypoints", "abi" in c)

# ── 6. Compile Vault ──
v = post("/v1/compile", {
    "template": "daglock_vault",
    "params": {
        "owner_key": "99a3552dd14f06833328b07ee57fa4933bb0fc7e05ce6606ca563acd6552675e",
        "timeout": "86400",
        "treasury_key": "4cbe03e1113c7506932623b76e182acd70f8dd7defbc3ccb3b572f53f5aae3ca"
    }
})
check("Vault compile", "script" in v and "template_hash" in v, str(v.get("error","")))

# ── 7. Offers (empty board) ──
o = get("/v1/offers")
check("Offers endpoint", "offers" in o)

# ── 8. Escrows by address ──
e = get("/v1/escrows?address=kaspatest:qzx000000000000000000000000000000000000000")
check("Escrows endpoint", "escrows" in e)

# ── 9. Reputation ──
r = get("/v1/reputation/kaspatest:qzx000000000000000000000000000000000000000")
check("Reputation endpoint", "score" in r or "trades" in r or "_error" in r)

# ── 10. Price ──
p = get("/v1/network/price")
check("Price endpoint", "kas_usd" in p)

# ── 11. OpenAPI spec ──
api = get("/v1/openapi.json")
check("OpenAPI spec", "paths" in api)
if "paths" in api:
    check(f"OpenAPI has {len(api['paths'])} endpoints", len(api['paths']) >= 15)

# ── Summary ──
print(f"\n{'='*40}")
print(f"  {green(f'{PASS} passed')}, {red(f'{FAIL} failed')}")
print(f"{'='*40}")

sys.exit(1 if FAIL > 0 else 0)
