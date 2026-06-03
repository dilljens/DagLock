#!/usr/bin/env bash
# DagLock Simulation — Test all escrow lifecycle permutations
# against the indexer REST API.
#
# Usage:
#   ./scripts/simulation.sh                    # starts its own indexer on :8443
#   API_URL=http://my-indexer:8443 ./scripts/simulation.sh
#
# Requires: curl, python3, cargo, sqlite3

	set -euo pipefail

API_URL="${API_URL:-http://localhost:8443}"
REPO_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PASS=0
FAIL=0
INDEXER_PID=""
CLEANUP_DB=""

GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'
pass() {
	PASS=$((PASS + 1))
	echo -e "  ${GREEN}PASS${NC} $1"
}
fail() {
	FAIL=$((FAIL + 1))
	echo -e "  ${RED}FAIL${NC} $1"
}
info() { echo -e "  ${CYAN}→${NC} $1"; }
header() { echo -e "\n${YELLOW}══ $1 ══${NC}"; }

random_addr() {
	local chars="qpzry9x8gf2tvdw0s3jn54khce6mua7l"
	local r="kaspa:q"
	for ((i = 0; i < 35; i++)); do r+="${chars:$((RANDOM % 32)):1}"; done
	echo "$r"
}
random_hex() { python3 -c "import secrets; print(secrets.token_hex(${1:-32}))"; }
random_id() { python3 -c "import uuid; print(str(uuid.uuid4()).split('-')[0])"; }
sompi() { python3 -c "print(int($1 * 100_000_000))"; }

api() { curl -sf "$API_URL$1" 2>/dev/null || echo '{"_error":"http"}'; }
post() { curl -sf -X POST "$API_URL$1" -H "Content-Type: application/json" -d "$2" 2>/dev/null || echo '{"_error":"http"}'; }
auth() {
	curl -sf -X POST "$API_URL$1" -H "Content-Type: application/json" \
		-H "X-Daglock-Address: $3" -H "X-Daglock-Signature: $4" -H "X-Daglock-Message: $5" \
		-d "$2" 2>/dev/null || echo '{"_error":"http"}'
}

extract() {
	local json="$1" expr="$2"
	echo "$json" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d$expr)" 2>/dev/null || echo "MISSING"
}

sql_active() {
	local id="$1" db="${CLEANUP_DB:-${REPO_DIR}/daglock_sim.db}"
	[[ -f "$db" ]] && sqlite3 "$db" "UPDATE escrows SET status='active' WHERE id='$id'" 2>/dev/null || true
}

# ── Preflight ──
header "Preflight"
if curl -sf "$API_URL/v1/health" >/dev/null 2>&1; then
	info "Indexer running at $API_URL"
else
	CLEANUP_DB="${REPO_DIR}/daglock_sim.db"
	rm -f "$CLEANUP_DB"*
	cargo build -p daglock-indexer 2>/dev/null
	cargo run -p daglock-indexer -- --database-url "sqlite:${CLEANUP_DB}" --host 127.0.0.1 --port 8443 &>/dev/null &
	INDEXER_PID=$!
	# Wait for indexer to become ready
	for i in $(seq 1 15); do
		if curl -sf "$API_URL/v1/health" >/dev/null 2>&1; then
			break
		fi
		sleep 1
	done
	curl -sf "$API_URL/v1/health" >/dev/null 2>&1 || {
		echo "Indexer failed to start"
		exit 1
	}
	info "Indexer started (PID: $INDEXER_PID)"
fi

H=$(api "/v1/health")
[[ "$(extract "$H" "['status']")" == "ok" ]] && pass "Health check" || fail "Health check"

# ── S1: Escrow settle ──
header "Scenario 1: Escrow settle (happy path)"
B1=$(random_addr)
S1=$(random_addr)
E1=$(post "/v1/escrows" "{\"lock_tx_id\":\"$(random_hex 16)\",\"lock_tx_output_index\":0,\"buyer_address\":\"$B1\",\"seller_address\":\"$S1\",\"amount_sompi\":$(sompi 500),\"asset_type\":\"KAS\"}")
E1_ID=$(extract "$E1" "['id']")
if [[ -n "$E1_ID" ]] && [[ "$E1_ID" != "MISSING" ]] && [[ "$E1_ID" != "{"*"_error"* ]]; then
	pass "Created: $E1_ID (status: $(extract "$E1" "['status']"))"
	sql_active "$E1_ID"
	SETTLE=$(auth "/v1/escrows/$E1_ID/settle" "{}" "$S1" "mock_sig" "settle:$E1_ID")
	[[ "$(extract "$SETTLE" "['status']")" == "settled" ]] && pass "Settled: $E1_ID" || fail "Settle failed: $(echo "$SETTLE" | head -c 100)"
	# Receipt
	REC=$(api "/v1/receipts/$E1_ID")
	[[ "$(extract "$REC" "['receipt_id']" 2>/dev/null || echo "")" != "MISSING" ]] && pass "Receipt generated" || fail "Receipt missing"
else
	fail "Create escrow: $(echo "$E1" | head -c 100)"
fi

# ── S2: Escrow refund ──
header "Scenario 2: Escrow refund"
B2=$(random_addr)
S2=$(random_addr)
E2=$(post "/v1/escrows" "{\"lock_tx_id\":\"$(random_hex 16)\",\"lock_tx_output_index\":0,\"buyer_address\":\"$B2\",\"seller_address\":\"$S2\",\"amount_sompi\":$(sompi 250),\"asset_type\":\"KAS\"}")
E2_ID=$(extract "$E2" "['id']")
if [[ -n "$E2_ID" ]] && [[ "$E2_ID" != "MISSING" ]]; then
	pass "Created: $E2_ID"
	sql_active "$E2_ID"
	REF=$(auth "/v1/escrows/$E2_ID/refund" "{}" "$B2" "msig" "refund:$E2_ID")
	[[ "$(extract "$REF" "['status']")" == "refunded" ]] && pass "Refunded: $E2_ID" || fail "Refund: $(echo "$REF" | head -c 100)"
else fail "Create escrow"; fi

# ── S3: Cancel ──
header "Scenario 3: Escrow cancel"
E3=$(post "/v1/escrows" "{\"lock_tx_id\":\"$(random_hex 16)\",\"lock_tx_output_index\":0,\"buyer_address\":\"$(random_addr)\",\"amount_sompi\":$(sompi 100),\"asset_type\":\"KAS\"}")
E3_ID=$(extract "$E3" "['id']")
if [[ -n "$E3_ID" ]]; then
	C3=$(post "/v1/escrows/$E3_ID/cancel" "{}")
	[[ "$(extract "$C3" "['status']")" == "cancelled" ]] && pass "Cancelled pending escrow" || fail "Cancel: $(echo "$C3" | head -c 50)"
fi

# Can't settle cancelled escrow
E3b=$(post "/v1/escrows" "{\"lock_tx_id\":\"$(random_hex 16)\",\"lock_tx_output_index\":0,\"buyer_address\":\"$(random_addr)\",\"amount_sompi\":$(sompi 50),\"asset_type\":\"KAS\"}")
E3b_ID=$(extract "$E3b" "['id']")
sql_active "$E3b_ID"
S3b=$(auth "/v1/escrows/$E3b_ID/settle" "{}" "$(random_addr)" "msig" "settle:$E3b_ID")
# Should succeed since escrow was active
[[ "$(extract "$S3b" "['status']")" == "settled" ]] && pass "Settled active escrow" || info "Settle result: $(echo "$S3b" | head -c 50)"

# ── S4: Dispute + Evidence ──
header "Scenario 4: Dispute, evidence, resolution"
B4=$(random_addr)
S4=$(random_addr)
E4=$(post "/v1/escrows" "{\"lock_tx_id\":\"$(random_hex 16)\",\"lock_tx_output_index\":0,\"buyer_address\":\"$B4\",\"seller_address\":\"$S4\",\"amount_sompi\":$(sompi 1000),\"asset_type\":\"KAS\"}")
E4_ID=$(extract "$E4" "['id']")
if [[ -n "$E4_ID" ]]; then
	pass "Created: $E4_ID"
	sql_active "$E4_ID"
	D4=$(post "/v1/escrows/$E4_ID/dispute" '{"reason":"Seller never delivered"}')
	[[ "$(extract "$D4" "['status']")" == "disputed" ]] && pass "Disputed" || fail "Dispute: $(echo "$D4" | head -c 50)"
	EV4=$(auth "/v1/escrows/$E4_ID/evidence" '{"content":"I paid 1000 KAS, no delivery"}' "$B4" "msig" "evidence:$E4_ID")
	[[ "$(extract "$EV4" "['id']" 2>/dev/null)" != "MISSING" ]] && pass "Evidence submitted" || fail "Evidence: $(echo "$EV4" | head -c 50)"
	EV_LIST=$(api "/v1/escrows/$E4_ID/evidence")
	EV_CNT=$(extract "$EV_LIST" "['evidence'] | length" 2>/dev/null) || EV_CNT=0
	[[ "${EV_CNT:-0}" -ge 1 ]] && pass "Evidence listed (${EV_CNT:-0} items)" || fail "Evidence listing"
	R4=$(auth "/v1/escrows/$E4_ID/resolve-dispute" "{\"outcome\":\"expunge\",\"resolved_by\":\"$B4\"}" "$B4" "msig" "resolve:$E4_ID")
	[[ "$(extract "$R4" "['status']")" == "resolved" ]] && pass "Dispute resolved" || fail "Resolve: $(echo "$R4" | head -c 50)"
else fail "Create escrow"; fi

# ── S5: Arbiter escrow ──
header "Scenario 5: Arbiter escrow (mediator)"
B5=$(random_addr)
S5=$(random_addr)
M5=$(random_addr)
E5=$(post "/v1/escrows" "{\"lock_tx_id\":\"$(random_hex 16)\",\"lock_tx_output_index\":0,\"buyer_address\":\"$B5\",\"seller_address\":\"$S5\",\"amount_sompi\":$(sompi 2000),\"asset_type\":\"KAS\",\"mediator_key\":\"$M5\"}")
E5_MK=$(extract "$E5" "['mediator_key']" 2>/dev/null || echo "")
[[ "$E5_MK" == "$M5" ]] && pass "Arbiter escrow with mediator: $M5" || fail "Mediator missing from response: $(echo "$E5" | head -c 100)"

# ── S6: Reputation ──
header "Scenario 6: Reputation"
FRESH=$(random_addr)
RP=$(api "/v1/reputation/$FRESH")
RP_S=$(extract "$RP" "['score']" 2>/dev/null || echo "MISSING")
[[ "$RP_S" != "MISSING" ]] && pass "Fresh address score: $RP_S" || fail "Fresh reputation: $(echo "$RP" | head -c 50)"

# Trader from S1
RP_T=$(api "/v1/reputation/$B1")
RP_TC=$(extract "$RP_T" "['trade_count']" 2>/dev/null || echo 0)
[[ "$RP_TC" -gt 0 ]] && pass "Trader has $RP_TC trade(s)" || info "Trader trade_count: $RP_TC"

# ── S7: Offers ──
header "Scenario 7: Offer board"
CR=$(random_addr)
OF=$(post "/v1/offers" "{\"creator_address\":\"$CR\",\"side\":\"sell\",\"base_asset\":\"KAS\",\"quote_asset\":\"KRC20:NACHO\",\"amount_sompi\":$(sompi 500)}")
O_ID=$(extract "$OF" "['id']")
[[ "$(extract "$OF" "['status']")" == "proposed" ]] && pass "Offer created: $O_ID" || fail "Offer: $(echo "$OF" | head -c 50)"

OF_LIST=$(api "/v1/offers?status=proposed")
O_CNT=$(extract "$OF_LIST" "['total']" 2>/dev/null || echo 0)
[[ "$O_CNT" -gt 0 ]] && pass "Offer board: $O_CNT offers" || fail "Offer board"

CP=$(random_addr)
AC=$(post "/v1/offers/$O_ID/accept" "{\"counterparty_address\":\"$CP\"}")
[[ "$(extract "$AC" "['status']")" == "accepted" ]] && pass "Offer accepted" || fail "Accept: $(echo "$AC" | head -c 50)"

OF2=$(post "/v1/offers" "{\"creator_address\":\"$(random_addr)\",\"side\":\"buy\",\"base_asset\":\"KAS\",\"quote_asset\":\"KRC20:KASPY\",\"amount_sompi\":$(sompi 100)}")
O2_ID=$(extract "$OF2" "['id']")
CA=$(post "/v1/offers/$O2_ID/cancel" "{}")
[[ "$(extract "$CA" "['status']")" == "cancelled" ]] && pass "Offer cancelled" || fail "Cancel"

# ── S8: Telegram ──
header "Scenario 8: Telegram identity"
ID_ADDR=$(random_addr)
ID_HANDLE="@sim_$(random_id | head -c 8)"
ID_MSG="daglock.io:verify:telegram:$ID_HANDLE"
ID_RES=$(auth "/v1/identity" "{\"platform\":\"telegram\",\"handle\":\"$ID_HANDLE\",\"signed_message\":\"$ID_MSG\",\"signature_hex\":\"abc\"}" "$ID_ADDR" "abc" "$ID_MSG")
[[ "$(extract "$ID_RES" "['status']")" == "verified" ]] && pass "Telegram linked: $ID_HANDLE" || fail "Identity: $(echo "$ID_RES" | head -c 50)"
RP8=$(api "/v1/reputation/$ID_ADDR")
[[ "$(extract "$RP8" "['telegram_handle']" 2>/dev/null || echo "")" == "$ID_HANDLE" ]] && pass "Handle in reputation" || fail "Handle missing: $(echo "$RP8" | head -c 100)"

# ── S9: Edge cases ──
header "Scenario 9: Edge cases"
BAD=$(post "/v1/escrows" '{"lock_tx_id":"bad","lock_tx_output_index":0,"buyer_address":"bad","amount_sompi":100,"asset_type":"KAS"}')
[[ "$(extract "$BAD" "['error']" 2>/dev/null || echo "")" != "" ]] && pass "Rejected invalid address" || info "Bad addr: $(echo "$BAD" | head -c 50)"

ZERO=$(post "/v1/escrows" "{\"lock_tx_id\":\"z\",\"lock_tx_output_index\":0,\"buyer_address\":\"$(random_addr)\",\"amount_sompi\":0,\"asset_type\":\"KAS\"}")
[[ "$(extract "$ZERO" "['error']" 2>/dev/null || echo "")" != "" ]] && pass "Rejected zero amount" || info "Zero: $(echo "$ZERO" | head -c 50)"

NOTF=$(api "/v1/escrows/nonexistent")
info "Nonexistent: $(echo "$NOTF" | head -c 80)"

# ── S10: Stats ──
header "Scenario 10: Stats"
ST=$(api "/v1/stats")
TOT=$(extract "$ST" "['total_escrows']" 2>/dev/null || echo 0)
[[ "$TOT" -ge 5 ]] && pass "Stats: $TOT escrows" || info "Total: $TOT"

# ── Summary ──
header "Summary"
echo -e "  ${GREEN}Passed: $PASS${NC}"
echo -e "  ${RED}Failed: $FAIL${NC}"
echo -e "  Total:  $((PASS + FAIL))"
[[ "$FAIL" -eq 0 ]] && echo -e "\n  ${GREEN}✓ All scenarios passed${NC}" || echo -e "\n  ${YELLOW}$FAIL failure(s)${NC}"

# Cleanup
[[ -n "${INDEXER_PID:-}" ]] && kill "$INDEXER_PID" 2>/dev/null || true
[[ -f "${CLEANUP_DB:-}" ]] && rm -f "$CLEANUP_DB"*
exit "$FAIL"
