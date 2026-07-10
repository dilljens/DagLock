#!/bin/bash
# Health check for DagLock services on VPS.
# Run: ssh ubuntu@40.160.241.74 "bash -s" < scripts/health-check.sh
# Or use systemd timer to run periodically.

set -e

API_URL="${1:-https://api.daglock.com}"
BOT_NAME="@DagLock_bot"
WEB_URL="${2:-https://daglock.com}"

fail=0

echo "=== DagLock Health Check $(date -u '+%Y-%m-%dT%H:%M:%SZ') ==="

# 1. Indexer API health — try internal first, then external
echo -n "Indexer API: "
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 --max-time 10 "http://127.0.0.1:8443/v1/offers?limit=1" 2>/dev/null)
if [ "$HTTP_CODE" != "200" ]; then
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 --max-time 10 "${API_URL}/v1/offers?limit=1" 2>/dev/null || echo "000")
fi
if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ (HTTP $HTTP_CODE)"
elif [ "$HTTP_CODE" = "403" ] || [ "$HTTP_CODE" = "429" ]; then
    echo "⚠️  (HTTP $HTTP_CODE — rate limited, service likely up)"
else
    echo "❌ (HTTP $HTTP_CODE)"
    fail=$((fail + 1))
fi

# 2. Price oracle (checks that background task is running)
echo -n "Price oracle: "
PRICE_DATA=$(curl -s --connect-timeout 3 --max-time 5 "${API_URL}/v1/network/price" 2>/dev/null)
KAS_USD=$(echo "$PRICE_DATA" | python3 -c "import sys,json; print(json.load(sys.stdin).get('kas_usd','N/A'))" 2>/dev/null || echo "N/A")
if [ "$KAS_USD" != "N/A" ] && [ -n "$KAS_USD" ]; then
    echo "✅ KAS/USD=\$$KAS_USD"
else
    echo "⚠️  No price data"
fi

# 3. Offer board is serving data
echo -n "Offer board: "
OFFER_COUNT=$(curl -s --connect-timeout 5 --max-time 10 "${API_URL}/v1/offers?limit=1" 2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('total',0))" 2>/dev/null || echo "error")
if [ "$OFFER_COUNT" != "error" ]; then
    echo "✅ $OFFER_COUNT offers"
else
    echo "❌ Failed to fetch offers"
    fail=$((fail + 1))
fi

# 4. Web UI is serving
echo -n "Web UI: "
WEB_CODE=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 --max-time 10 "${WEB_URL}" 2>/dev/null || echo "000")
if [ "$WEB_CODE" = "200" ]; then
    echo "✅ (HTTP $WEB_CODE)"
else
    echo "⚠️  (HTTP $WEB_CODE)"
fi

# 5. Indexer process (requires SSH — skip if not on VPS)
if [ -n "$SSH_CLIENT" ] || [ -d /opt/daglock-indexer ]; then
    echo -n "Indexer process: "
    if systemctl is-active --quiet daglock-indexer 2>/dev/null; then
        echo "✅"
    else
        echo "❌ daglock-indexer not running"
        fail=$((fail + 1))
    fi

    echo -n "Bot process: "
    if systemctl is-active --quiet daglock-bot 2>/dev/null; then
        echo "✅"
    else
        echo "❌ daglock-bot not running"
        fail=$((fail + 1))
    fi

    echo -n "Nginx: "
    if nginx -t 2>&1 | grep -q "syntax is ok"; then
        echo "✅"
    else
        echo "❌"
        fail=$((fail + 1))
    fi

    echo -n "Disk: "
    DISK_PCT=$(df / | tail -1 | awk '{print $5}' | tr -d '%')
    if [ "$DISK_PCT" -lt 80 ]; then
        echo "✅ ${DISK_PCT}% used"
    else
        echo "⚠️  ${DISK_PCT}% used"
    fi

    echo -n "Memory: "
    MEM_INFO=$(free -m | grep Mem)
    MEM_USED=$(echo "$MEM_INFO" | awk '{print $3}')
    MEM_TOTAL=$(echo "$MEM_INFO" | awk '{print $2}')
    MEM_PCT=$((MEM_USED * 100 / MEM_TOTAL))
    if [ "$MEM_PCT" -lt 80 ]; then
        echo "✅ ${MEM_USED}MB/${MEM_TOTAL}MB (${MEM_PCT}%)"
    else
        echo "⚠️  ${MEM_USED}MB/${MEM_TOTAL}MB (${MEM_PCT}%)"
    fi
fi

echo ""
echo "=== Result ==="
if [ "$fail" -eq 0 ]; then
    echo "✅ All checks passed"
    exit 0
else
    echo "❌ $fail check(s) failed"
    exit 1
fi
