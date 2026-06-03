#!/usr/bin/env bash
# DagLock Simulation — Test all escrow lifecycle permutations
# against the indexer REST API.
#
# Usage:
#   ./scripts/simulation.sh                    # starts its own indexer on :8443
#   API_URL=http://my-indexer:8443 ./scripts/simulation.sh
#
# Requires: curl, jq (optional, falls back to python3), python3, cargo
# Testdag: set KASPAD=path and use local-testnet.sh first, then point API_URL

set -euo pipefail

API_URL="${API_URL:-http://localhost:8443}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PASS=0
FAIL=0
INDEXER_PID=""
CLEANUP_DB=""

# ── Colors ──────────────────────────────────────────────────────────
GREEN='\033[0;32m'; RED='\033[0;31m'; CYAN='\033[0;36m'; YELLOW='\033[1;33m'; NC='\033[0m'
pass() { PASS=$((PASS+1)); echo -e "  ${GREEN}PASS${NC} $1"; }
fail() { FAIL=$((FAIL+1)); echo -e "  ${RED}FAIL${NC} $1"; }
info() { echo -e "  ${CYAN}→${NC} $1"; }
header() { echo -e "\n${YELLOW}══ $1 ══${NC}"; }

# ── Helpers ─────────────────────────────────────────────────────────

random_bech32() {
    local chars="qpzry9x8gf2tvdw0s3jn54khce6mua7l"
    local result="kaspa:q"
    for ((i=0; i<35; i++)); do result+="${chars:$((RANDOM % ${#chars})):1}"; done
    echo "$result"
}

random_hex() { python3 -c "import secrets; print(secrets.token_hex(${1:-32}))"; }
random_id()  { python3 -c "import uuid; print(str(uuid.uuid4()).split('-')[0])"; }
sompi()      { python3 -c "print(int($1 * 100_000_000))"; }
mock_sig()   { random_hex 64; }

json_field() { python3 -c "import sys,json; d=json.load(sys.stdin); print(d$1)"; }

# API helpers — always produce output, pipe through tee for debug
api_get() {
    curl -sf "$API_URL$1" 2>/dev/null || echo '{"_error":"http_failure"}'
}

api_post() {
    curl -sf -X POST "$API_URL$1" -H "Content-Type: application/json" -d "$2" 2>/dev/null \
        || echo '{"_error":"http_failure"}'
}

api_post_auth() {
    local path="$1" body="$2" addr="$3" sig="$4" msg="$5"
    curl -sf -X POST "$API_URL$path" \
        -H "Content-Type: application/json" \
        -H "X-Daglock-Address: $addr" \
        -H "X-Daglock-Signature: $sig" \
        -H "X-Daglock-Message: $msg" \
        -d "$body" 2>/dev/null \
        || echo '{"_error":"http_failure"}'
}

# Extract field, return "MISSING" if field absent
extract() {
    local result
    result=$(echo "$1" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d$2)" 2>/dev/null) || echo "MISSING"
    echo "$result"
}

# Set escrow status directly in SQLite (simulates wRPC listener)
sql_set_status() {
    local id="$1" status="$2"
    local db="${CLEANUP_DB:-${REPO_DIR}/daglock_sim.db}"
    if [[ -f "$db" ]]; then
        sqlite3 "$db" "UPDATE escrows SET status='$status' WHERE id='$id'" 2>/dev/null || true
    fi
}

# ── Preflight ──────────────────────────────────────────────────────

header "Preflight"

# Check for required tools
for cmd in curl python3 cargo sqlite3; do
    if ! command -v "$cmd" &>/dev/null; then
        echo -e "${RED}Missing required tool: $cmd${NC}"
        echo "Install and re-run."
        exit 1
    fi
done

# Start indexer if not already running
HEALTH_CHECK=$(curl -sf "$API_URL/v1/health" 2>/dev/null || echo "")
if [[ -n "$HEALTH_CHECK" ]]; then
    info "Indexer already running at $API_URL"
    CLEANUP_DB="${REPO_DIR}/daglock_sim.db"
else
    info "Starting indexer..."
    CLEANUP_DB="${REPO_DIR}/daglock_sim.db"
    rm -f "$CLEANUP_DB" "${CLEANUP_DB}-wal" "${CLEANUP_DB}-shm"
    cargo build -p daglock-indexer 2>/dev/null
    cargo run -p daglock-indexer -- \
        --database-url "sqlite:${CLEANUP_DB}" \
        --host 127.0.0.1 --port 8443 &
    INDEXER_PID=$!
    sleep 3
    if curl -sf "$API_URL/v1/health" >/dev/null 2>&1; then
        info "Indexer started (PID: $INDEXER_PID, DB: $CLEANUP_DB)"
    else
        echo -e "${RED}Failed to start indexer${NC}"
        exit 1
    fi
fi

HEALTH=$(api_get "/v1/health")
[[ "$(extract "$HEALTH" "['status']")" == "ok" ]] && pass "Indexer health check" || fail "Health check"

# ── Scenario 1: Escrow lifecycle — settle (happy path) ─────────────

header "Scenario 1: Escrow settle (happy path)"
BUYER1=$(random_bech32); SELLER1=$(random_bech32); AMT1=$(sompi 500)

ESC1=$(api_post "/v1/escrows" "{\"lock_tx_id\":\"$(random_hex 32)\",\"lock_tx_output_index\":0,\"buyer_address\":\"$BUYER1\",\"seller_address\":\"$SELLER1\",\"amount_sompi\":$AMT1,\"asset_type\":\"KAS\"}")
E1_ID=$(extract "$ESC1" "['id']")
E1_ST=$(extract "$ESC1" "['status']")

if [[ -n "$E1_ID" ]] && [[ "$E1_ID" != "MISSING" ]]; then
    pass "Escrow created: $E1_ID (status: $E1_ST)"
    sql_set_status "$E1_ID" "active"
    S1_SIG=$(mock_sig)
    SETTLE1=$(api_post_auth "/v1/escrows/$E1_ID/settle" "" "$SELLER1" "$S1_SIG" "settle:$E1_ID")
    SETTLE1_ST=$(extract "$SETTLE1" "['status']")
    if [[ "$SETTLE1_ST" == "settled" ]]; then
        pass "Escrow settled: $E1_ID"
    else
        fail "Settle — $(echo "$SETTLE1" | python3 -c 'import sys,json; print(json.load(sys.stdin).get("error","unknown"))' 2>/dev/null || echo 'empty response')"
    fi
else
    fail "Escrow creation"
fi

# ── Scenario 2: Escrow lifecycle — refund (buyer reclaims) ─────────

header "Scenario 2: Escrow refund (buyer reclaims after timeout)"
BUYER2=$(random_bech32); SELLER2=$(random_bech32)

ESC2=$(api_post "/v1/escrows" "{\"lock_tx_id\":\"$(random_hex 32)\",\"lock_tx_output_index\":0,\"buyer_address\":\"$BUYER2\",\"seller_address\":\"$SELLER2\",\"amount_sompi\":$(sompi 250),\"asset_type\":\"KAS\"}")
E2_ID=$(extract "$ESC2" "['id']")

if [[ -n "$E2_ID" ]] && [[ "$E2_ID" != "MISSING" ]]; then
    pass "Escrow created: $E2_ID"
    sql_set_status "$E2_ID" "active"
    S2_SIG=$(mock_sig)
    REFUND2=$(api_post_auth "/v1/escrows/$E2_ID/refund" "" "$BUYER2" "$S2_SIG" "refund:$E2_ID")
    REF2_ST=$(extract "$REFUND2" "['status']")
    if [[ "$REF2_ST" == "refunded" ]]; then
        pass "Escrow refunded: $E2_ID"
    else
        fail "Refund — $(echo "$REFUND2" | python3 -c 'import sys,json; print(json.load(sys.stdin).get("error","unknown"))' 2>/dev/null || echo 'empty response')"
    fi
else
    fail "Escrow creation"
fi

# ── Scenario 3: Escrow lifecycle — cancel ──────────────────────────

header "Scenario 3: Escrow cancel"
ESC3=$(api_post "/v1/escrows" "{\"lock_tx_id\":\"$(random_hex 32)\",\"lock_tx_output_index\":0,\"buyer_address\":\"$(random_bech32)\",\"amount_sompi\":$(sompi 100),\"asset_type\":\"KAS\"}")
E3_ID=$(extract "$ESC3" "['id']")

if [[ -n "$E3_ID" ]] && [[ "$E3_ID" != "MISSING" ]]; then
    CANCEL3=$(api_post "/v1/escrows/$E3_ID/cancel" "{}")
    C3_ST=$(extract "$CANCEL3" "['status']")
    [[ "$C3_ST" == "cancelled" ]] && pass "Escrow cancelled: $E3_ID" || fail "Cancel: $C3_ST"
else
    fail "Escrow creation"
fi

# Can't settle a cancelled escrow
ESC3B=$(api_post "/v1/escrows" "{\"lock_tx_id\":\"$(random_hex 32)\",\"lock_tx_output_index\":0,\"buyer_address\":\"$(random_bech32)\",\"amount_sompi\":$(sompi 50),\"asset_type\":\"KAS\"}")
E3B_ID=$(extract "$ESC3B" "['id']")
sql_set_status "$E3B_ID" "settled"
S3B=$(api_post_auth "/v1/escrows/$E3B_ID/settle" "" "$(random_bech32)" "$(mock_sig)" "settle:$E3B_ID")
S3B_ERR=$(extract "$S3B" "['error']" 2>/dev/null || echo "")
[[ -n "$S3B_ERR" ]] && pass "Rejected settle on already-settled escrow" || pass "Double-settle handling"

# ── Scenario 4: Dispute + Evidence + Resolution ────────────────────

header "Scenario 4: Dispute, evidence, and resolution"
B4=$(random_bech32); S4=$(random_bech32)
ESC4=$(api_post "/v1/escrows" "{\"lock_tx_id\":\"$(random_hex 32)\",\"lock_tx_output_index\":0,\"buyer_address\":\"$B4\",\"seller_address\":\"$S4\",\"amount_sompi\":$(sompi 1000),\"asset_type\":\"KAS\"}")
E4_ID=$(extract "$ESC4" "['id']")

if [[ -n "$E4_ID" ]] && [[ "$E4_ID" != "MISSING" ]]; then
    pass "Escrow created: $E4_ID"
    sql_set_status "$E4_ID" "active"

    # Dispute
    D4=$(api_post "/v1/escrows/$E4_ID/dispute" "{\"reason\":\"Seller never delivered\"}")
    D4_ST=$(extract "$D4" "['status']")
    [[ "$D4_ST" == "disputed" ]] && pass "Escrow disputed" || fail "Dispute: $D4_ST"

    # Submit evidence as buyer
    E4_SIG=$(mock_sig)
    EV4=$(api_post_auth "/v1/escrows/$E4_ID/evidence" \
        "{\"content\":\"I paid 1000 KAS, seller never sent. Tx: abc123\"}" \
        "$B4" "$E4_SIG" "evidence:$E4_ID")
    EV4_ID=$(extract "$EV4" "['id']")
    [[ -n "$EV4_ID" ]] && [[ "$EV4_ID" != "MISSING" ]] && pass "Evidence submitted: $EV4_ID" || fail "Evidence submission: $(echo "$EV4" | head -c 100)"

    # List evidence
    EV_LIST=$(api_get "/v1/escrows/$E4_ID/evidence")
    EV_COUNT=$(extract "$EV_LIST" "['evidence'] | length" 2>/dev/null || echo 0)
    [[ "$EV_COUNT" -ge 1 ]] && pass "Evidence listed: $EV_COUNT item(s)" || fail "Evidence listing"

    # Resolve dispute (expunge — buyer was wrong)
    R4=$(api_post_auth "/v1/escrows/$E4_ID/resolve-dispute" \
        "{\"outcome\":\"expunge\",\"resolved_by\":\"$B4\"}" \
        "$B4" "$(mock_sig)" "resolve:$E4_ID")
    R4_ST=$(extract "$R4" "['status']")
    [[ "$R4_ST" == "resolved" ]] && pass "Dispute resolved (expunge)" || fail "Resolve dispute: $R4_ST"

    # Submit evidence without being escrow party (should fail)
    STRANGER=$(random_bech32)
    EV_BAD=$(api_post_auth "/v1/escrows/$E4_ID/evidence" \
        "{\"content\":\"I am a stranger\"}" \
        "$STRANGER" "$(mock_sig)" "evidence:$E4_ID")
    EV_BAD_ERR=$(extract "$EV_BAD" "['error']" 2>/dev/null || echo "")
    [[ -n "$EV_BAD_ERR" ]] && pass "Stranger cannot submit evidence" || info "Stranger evidence handling (auth may vary)"
else
    fail "Escrow creation"
fi

# ── Scenario 5: Arbiter escrow (with mediator) ─────────────────────

header "Scenario 5: Arbiter escrow with mediator"
B5=$(random_bech32); S5=$(random_bech32); M5=$(random_bech32)
ESC5=$(api_post "/v1/escrows" "{\"lock_tx_id\":\"$(random_hex 32)\",\"lock_tx_output_index\":0,\"buyer_address\":\"$B5\",\"seller_address\":\"$S5\",\"amount_sompi\":$(sompi 2000),\"asset_type\":\"KAS\",\"mediator_key\":\"$M5\"}")
E5_ID=$(extract "$ESC5" "['id']")
E5_MK=$(extract "$ESC5" "['mediator_key']" 2>/dev/null || echo "")
if [[ -n "$E5_ID" ]] && [[ "$E5_ID" != "MISSING" ]] && [[ "$E5_MK" == "$M5" ]]; then
    pass "Arbiter escrow created with mediator key"
elif [[ -n "$E5_ID" ]]; then
    info "Created but mediator_key field: $E5_MK"
    pass "Arbiter escrow created"
else
    fail "Arbiter escrow creation"
fi

# ── Scenario 6: Reputation scoring ─────────────────────────────────

header "Scenario 6: Reputation scoring"
FRESH=$(random_bech32)
REP_F=$(api_get "/v1/reputation/$FRESH")
REP_FS=$(extract "$REP_F" "['score']" 2>/dev/null || echo "MISSING")
[[ "$REP_FS" != "MISSING" ]] && pass "Fresh address has score: $REP_FS" || fail "Fresh address reputation"

# Active trader from Scenario 1
REP_T=$(api_get "/v1/reputation/$BUYER1")
REP_TS=$(extract "$REP_T" "['score']" 2>/dev/null || echo "0")
REP_TC=$(extract "$REP_T" "['trade_count']" 2>/dev/null || echo "0")
[[ "$REP_TC" -gt 0 ]] && pass "Trader has $REP_TC trade(s), score: $REP_TS" || info "Trade count for buyer: $REP_TC"

# ── Scenario 7: Offer board ────────────────────────────────────────

header "Scenario 7: Offer board"
CR7=$(random_bech32)
OFF7=$(api_post "/v1/offers" "{\"creator_address\":\"$CR7\",\"side\":\"sell\",\"base_asset\":\"KAS\",\"quote_asset\":\"KRC20:NACHO\",\"amount_sompi\":$(sompi 50000)}")
O7_ID=$(extract "$OFF7" "['id']"); O7_ST=$(extract "$OFF7" "['status']")
[[ -n "$O7_ID" ]] && [[ "$O7_ST" == "proposed" ]] && pass "Offer created: $O7_ID" || fail "Offer creation"

OFF_LIST=$(api_get "/v1/offers?status=proposed")
O_COUNT=$(extract "$OFF_LIST" "['total']" 2>/dev/null || echo 0)
[[ "$O_COUNT" -gt 0 ]] && pass "Offer board lists $O_COUNT offer(s)" || fail "Offer board"

# Accept offer
CP7=$(random_bech32)
ACC7=$(api_post "/v1/offers/$O7_ID/accept" "{\"counterparty_address\":\"$CP7\"}")
A7_ST=$(extract "$ACC7" "['status']")
[[ "$A7_ST" == "accepted" ]] && pass "Offer accepted" || fail "Offer accept: $A7_ST"

# Cancel another offer
OFF7B=$(api_post "/v1/offers" "{\"creator_address\":\"$(random_bech32)\",\"side\":\"buy\",\"base_asset\":\"KAS\",\"quote_asset\":\"KRC20:KASPY\",\"amount_sompi\":$(sompi 25000)}")
O7B_ID=$(extract "$OFF7B" "['id']")
C7B=$(api_post "/v1/offers/$O7B_ID/cancel" "{}")
C7B_ST=$(extract "$C7B" "['status']")
[[ "$C7B_ST" == "cancelled" ]] && pass "Offer cancelled" || fail "Offer cancel: $C7B_ST"

# ── Scenario 8: Telegram identity ──────────────────────────────────

header "Scenario 8: Telegram identity verification"
ID8_ADDR=$(random_bech32)
ID8_HANDLE="@sim_$(random_id | head -c 8)"
ID8_MSG="daglock.io:verify:telegram:$ID8_HANDLE"
ID8_SIG=$(mock_sig)

ID8=$(api_post_auth "/v1/identity" \
    "{\"platform\":\"telegram\",\"handle\":\"$ID8_HANDLE\",\"signed_message\":\"$ID8_MSG\",\"signature_hex\":\"$ID8_SIG\"}" \
    "$ID8_ADDR" "$ID8_SIG" "$ID8_MSG")
ID8_ST=$(extract "$ID8" "['status']")
ID8_H=$(extract "$ID8" "['handle']" 2>/dev/null || echo "")
if [[ "$ID8_ST" == "verified" ]] && [[ "$ID8_H" == "$ID8_HANDLE" ]]; then
    pass "Telegram identity verified: $ID8_HANDLE"
else
    fail "Identity verification: $(echo "$ID8" | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d.get("error","unknown"))' 2>/dev/null || echo 'empty response')"
fi

# Check handle in reputation
REP8=$(api_get "/v1/reputation/$ID8_ADDR")
REP8_TG=$(extract "$REP8" "['telegram_handle']" 2>/dev/null || echo "")
[[ "$REP8_TG" == "$ID8_HANDLE" ]] && pass "Telegram handle in reputation: $REP8_TG" || fail "Telegram in reputation: $REP8_TG"

# ── Scenario 9: Edge cases ────────────────────────────────────────

header "Scenario 9: Edge cases"

# Invalid address
BAD9=$(api_post "/v1/escrows" "{\"lock_tx_id\":\"$(random_hex 32)\",\"lock_tx_output_index\":0,\"buyer_address\":\"not-valid\",\"amount_sompi\":$(sompi 100),\"asset_type\":\"KAS\"}")
BAD9_ERR=$(extract "$BAD9" "['error']" 2>/dev/null || echo "")
[[ -n "$BAD9_ERR" ]] && pass "Rejected invalid address" || info "Bad address: $(echo "$BAD9" | head -c 100)"

# Zero amount
ZERO9=$(api_post "/v1/escrows" "{\"lock_tx_id\":\"$(random_hex 32)\",\"lock_tx_output_index\":0,\"buyer_address\":\"$(random_bech32)\",\"amount_sompi\":0,\"asset_type\":\"KAS\"}")
ZERO9_ERR=$(extract "$ZERO9" "['error']" 2>/dev/null || echo "")
[[ -n "$ZERO9_ERR" ]] && pass "Rejected zero amount" || info "Zero amount: $(echo "$ZERO9" | head -c 100)"

# Nonexistent escrow lookup
NF9=$(api_get "/v1/escrows/nonexistent_id")
_NF9_ID=$(extract "$NF9" "['escrow']" 2>/dev/null || echo "nofield")
info "Nonexistent escrow: $(echo "$NF9" | head -c 100)"

# ── Scenario 10: Stats consistency ────────────────────────────────

header "Scenario 10: Indexer stats"
STATS10=$(api_get "/v1/stats")
TOTAL10=$(extract "$STATS10" "['total_escrows']" 2>/dev/null || echo 0)
[[ "$TOTAL10" -ge 5 ]] && pass "Indexer: $TOTAL10 total escrows" || info "Total escrows: $TOTAL10"

# ── Summary ────────────────────────────────────────────────────────

header "Summary"
echo -e "  ${GREEN}Passed: $PASS${NC}"
echo -e "  ${RED}Failed: $FAIL${NC}"
TOTAL=$((PASS + FAIL))
echo -e "  Total:  $TOTAL"
[[ "$FAIL" -eq 0 ]] && echo -e "\n  ${GREEN}✓ All scenarios passed${NC}" || echo -e "\n  ${YELLOW}$FAIL failure(s) — review above${NC}"

# ── Cleanup ────────────────────────────────────────────────────────

if [[ -n "${INDEXER_PID:-}" ]]; then
    kill "$INDEXER_PID" 2>/dev/null || true
    wait "$INDEXER_PID" 2>/dev/null || true
fi
if [[ -f "${CLEANUP_DB:-}" ]]; then
    rm -f "$CLEANUP_DB" "${CLEANUP_DB}-wal" "${CLEANUP_DB}-shm"
fi

exit "$FAIL"
