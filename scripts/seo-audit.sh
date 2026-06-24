#!/usr/bin/env bash
# DagLock SEO Audit — CI-ready script
# Usage: bash scripts/seo-audit.sh [url]
#   Set PAGESPEED_API_KEY for PageSpeed Insights checks
#   Set SEARCH_CONSOLE_KEY for Google Search Console data (JSON key file)
set -uo pipefail

URL="${1:-https://daglock.com}"
PAGESPEED_API_KEY="${PAGESPEED_API_KEY:-}"
SEARCH_CONSOLE_KEY="${SEARCH_CONSOLE_KEY:-}"
PASS=0
FAIL=0

red()   { echo -e "\033[31m$1\033[0m"; }
green() { echo -e "\033[32m$1\033[0m"; }

echo "=========================================="
echo "  DagLock SEO Audit — $(date -u +%Y-%m-%d)"
echo "  Target: $URL"
echo "=========================================="
echo ""

# ── 1. Broken Links (linkinator) ─────────────────────────────────
echo "--- 1. Broken Link Check ---"
if command -v npx &>/dev/null; then
  npx linkinator "$URL" --recurse \
    --skip "mailto:|linkedin.com|facebook.com|twitter.com|x.com|beacon.min.js" \
    2>&1 | tail -5
  
  # Check for non-200 responses
  BROKEN=$(npx linkinator "$URL" --recurse --skip "mailto:|linkedin.com|facebook.com" 2>&1 | grep -v "\[200\]" | grep -c "\[")
  if [ "$BROKEN" -eq 0 ]; then
    green "  ✅ No broken links found"
    PASS=$((PASS + 1))
  else
    red "  ❌ $BROKEN broken links found"
    FAIL=$((FAIL + 1))
  fi
else
  red "  ⚠️  npx not available, skipping"
fi
echo ""

# ── 2. PageSpeed Insights ────────────────────────────────────────
echo "--- 2. PageSpeed Insights ---"
if [ -n "$PAGESPEED_API_KEY" ]; then
  for STRATEGY in mobile desktop; do
    RESP=$(curl -s "https://www.googleapis.com/pagespeedonline/v5/runPagespeed?url=$URL&strategy=$STRATEGY&key=$PAGESPEED_API_KEY")
    SCORE=$(echo "$RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('lighthouseResult',{}).get('categories',{}).get('performance',{}).get('score',0))" 2>/dev/null || echo "0")
    SEO=$(echo "$RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('lighthouseResult',{}).get('categories',{}).get('seo',{}).get('score',0))" 2>/dev/null || echo "0")
    
    PCT=$(echo "$SCORE * 100 / 1" | bc 2>/dev/null || echo "0")
    SEOPCT=$(echo "$SEO * 100 / 1" | bc 2>/dev/null || echo "0")
    
    echo "  $STRATEGY — Performance: ${PCT}/100, SEO: ${SEOPCT}/100"
    
    if [ "$PCT" -ge 50 ] 2>/dev/null; then
      PASS=$((PASS + 1))
    else
      red "  ❌ Performance below 50 on $STRATEGY"
      FAIL=$((FAIL + 1))
    fi
  done
else
  echo "  ⚠️  Set PAGESPEED_API_KEY for performance data"
  echo "     Get a key: https://developers.google.com/speed/docs/insights/v5/get-started"
fi
echo ""

# ── 3. Sitemap Check ─────────────────────────────────────────────
echo "--- 3. Sitemap Check ---"
SITEMAP_URL="${URL%/}/sitemap.xml"
SITEMAP_OK=$(curl -sI "$SITEMAP_URL" | head -1 | grep -c "200")
if [ "$SITEMAP_OK" -eq 1 ]; then
  echo "  ✅ Sitemap reachable: $SITEMAP_URL"
  PASS=$((PASS + 1))
  
  # Count URLs in sitemap
  COUNT=$(curl -s "$SITEMAP_URL" | grep -o '<loc>' | wc -l)
  echo "     URLs listed: $COUNT"
else
  red "  ❌ Sitemap not found at $SITEMAP_URL"
  FAIL=$((FAIL + 1))
fi
echo ""

# ── 4. Robots.txt Check ──────────────────────────────────────────
echo "--- 4. Robots.txt Check ---"
ROBOTS_URL="${URL%/}/robots.txt"
ROBOTS_OK=$(curl -sI "$ROBOTS_URL" | head -1 | grep -c "200")
if [ "$ROBOTS_OK" -eq 1 ]; then
  echo "  ✅ Robots.txt reachable: $ROBOTS_URL"
  PASS=$((PASS + 1))
  
  # Check sitemap reference
  HAS_SITEMAP=$(curl -s "$ROBOTS_URL" | grep -c "Sitemap:" || true)
  if [ "$HAS_SITEMAP" -ge 1 ]; then
    echo "     Sitemap referenced in robots.txt ✅"
  else
    red "  ⚠️  No Sitemap directive in robots.txt"
  fi
else
  red "  ❌ Robots.txt not found"
  FAIL=$((FAIL + 1))
fi
echo ""

# ── 5. OG Tags Check ─────────────────────────────────────────────
echo "--- 5. Open Graph / Meta Tags ---"
HTML=$(curl -s "$URL")
OG_TITLE=$(echo "$HTML" | grep -oP 'og:title[^>]*content="([^"]*)"' | head -1 || echo "")
OG_DESC=$(echo "$HTML" | grep -oP 'og:description[^>]*content="([^"]*)"' | head -1 || echo "")
TW_CARD=$(echo "$HTML" | grep -oP 'twitter:card[^>]*content="([^"]*)"' | head -1 || echo "")
CANONICAL=$(echo "$HTML" | grep -oP 'rel="canonical"[^>]*href="([^"]*)"' | head -1 || echo "")

[ -n "$OG_TITLE" ] && echo "  ✅ og:title found" && PASS=$((PASS + 1)) || { red "  ❌ og:title missing"; FAIL=$((FAIL + 1)); }
[ -n "$OG_DESC" ] && echo "  ✅ og:description found" || { red "  ❌ og:description missing"; }
[ -n "$TW_CARD" ] && echo "  ✅ twitter:card found" || { red "  ❌ twitter:card missing"; }
[ -n "$CANONICAL" ] && echo "  ✅ canonical URL found" || { red "  ❌ canonical URL missing"; }
echo ""

# ── 6. JSON-LD Structured Data ───────────────────────────────────
echo "--- 6. Structured Data (JSON-LD) ---"
HAS_JSONLD=$(echo "$HTML" | grep -c "application/ld+json" || true)
if [ "$HAS_JSONLD" -ge 1 ]; then
  echo "  ✅ JSON-LD found"
  HAS_TYPE=$(echo "$HTML" | grep -c '"@type"' || true)
  HAS_NAME=$(echo "$HTML" | grep -c '"name"' || true)
  [ "$HAS_TYPE" -ge 1 ] && echo "     @type present ✅" || red "     @type missing ❌"
  [ "$HAS_NAME" -ge 1 ] && echo "     name present ✅" || red "     name missing ❌"
  PASS=$((PASS + 1))
else
  red "  ❌ No JSON-LD structured data found"
  FAIL=$((FAIL + 1))
fi
echo ""

# ── Summary ──────────────────────────────────────────────────────
echo "=========================================="
echo "  Results: $PASS passed, $FAIL failed"
echo "=========================================="

if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
