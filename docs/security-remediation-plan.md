# DagLock Security Remediation Plan

**Priority ordering:** Ship-blocking → Mainnet safety → Defense-in-depth

---

## Phase 0: Quick Wins (Do Before Next Deploy)

These are one-liner config/process changes that buy immediate safety.

| # | Action | File | Risk |
|---|--------|------|------|
| P0.1 | Set `DAGLOCK_MESSAGE_KEY` in production env | deployment config | **H1** — dev key decrypts all past messages |
| P0.2 | Set `BOT_ENCRYPTION_KEY` in production env | deployment config | **H1** — bot stores addresses in plaintext |
| P0.3 | Remove `--no-wrpc` from production startup flags | systemd service / railway.json | **H6** — MockVerifier accepts all UTXOs |
| P0.4 | Set `--kaspa-api-url https://api-tn11.kaspa.org` | systemd service | **H6** — enables real UTXO verification |
| P0.5 | Set `DAGLOCK_MESSAGE_KEY` with a real 64-hex-char key | deployment config | **H1** |

**No code changes needed.** These are operational fixes for the current deployment.

---

## Phase 1: Critical (Ship-Blocking — Fix Before Mainnet)

### C1 — Atomic Swap Needs Signature Auth

**Problem:** `atomic_swap()` in `escrow_service.rs` allows anyone with a preimage to settle an escrow — no caller verification.

**Fix — `indexer/src/services/escrow_service.rs`:**
```rust
pub async fn atomic_swap(
    &self,
    id: &str,
    preimage_hex: &str,
    headers: &axum::http::HeaderMap,     // NEW
) -> Result<(), ServiceError> {
    let current = self.get_settleable_escrow(id).await?;

    // NEW: Verify caller is the counterparty
    let auth = AuthContext::from_headers(headers)
        .map_err(|e| ServiceError::Unauthorized(e.to_string()))?;

    // Must be the seller (counterparty) executing the swap
    let is_seller = current.seller_address.as_deref() == Some(&auth.address);
    if !is_seller {
        return Err(ServiceError::Forbidden(
            "Only the seller can claim an atomic swap".into()
        ));
    }

    let parsed = parse_message(&auth.message)
        .map_err(|e| ServiceError::InvalidInput(e.to_string()))?;
    if parsed.action != "swap" || parsed.escrow_id != id {
        return Err(ServiceError::Forbidden(
            "Message must be 'swap:{id}:ts:nonce'".into()
        ));
    }
    if !self.sig_verifier.verify_signature(&auth.address, &auth.signature, &auth.message)
        .unwrap_or(false)
    {
        return Err(ServiceError::Forbidden("Invalid signature".into()));
    }
    verify_nonce(&self.db, &parsed, &auth.address).await
        .map_err(|e| ServiceError::Forbidden(e.to_string()))?;

    // ... existing preimage verification ...
```

**Files to change:**
- `indexer/src/services/escrow_service.rs` — add auth to `atomic_swap`
- `indexer/src/api/escrows.rs` — pass `headers` to `svc.atomic_swap`
- `indexer/src/api/swap.rs` — check if it also needs auth
- `bot/src/lib/api.js` — add `swapEscrow` auth headers
- `web/src/api.ts` — add auth to `swapEscrow` call
- `indexer/src/api/multi_escrows.rs` — check `swap` handler for multi-escrows

---

### C3 — Add Auth to All State-Changing Endpoints Missing It

**Server-side missing auth** — add `AuthContext` extraction + `verify_*_authorization`:

| Endpoint | File | Handler |
|----------|------|---------|
| `POST /v1/escrows/:id/swap` | `indexer/src/api/escrows.rs` | `atomic_swap` → C1 fix covers this |
| `POST /v1/milestones/:id/release` | `indexer/src/api/milestones.rs` | `release_milestone` |
| `POST /v1/milestones/:id/refund` | `indexer/src/api/milestones.rs` | `refund` |
| `POST /v1/milestones/:id/complete` | `indexer/src/api/milestones.rs` | `complete` |
| `POST /v1/subscriptions/:id/draw` | `indexer/src/api/subscriptions.rs` | `draw` |
| `POST /v1/multi-escrows/:id/swap` | `indexer/src/api/multi_escrows.rs` | `swap` |
| `POST /v1/multi-escrows/:id/refund` | `indexer/src/api/multi_escrows.rs` | `refund` |
| `POST /v1/vaults/:id/sweep` | `indexer/src/api/vaults.rs` | `sweep_vault` |
| `POST /v1/vaults/:id/early-exit` | `indexer/src/api/vaults.rs` | `early_exit` |
| `POST /v1/vaults/:id/heir-withdraw` | `indexer/src/api/vaults.rs` | `heir_withdraw` |
| `POST /v1/escrows/:id/auto-settle` | `indexer/src/api/escrows.rs` | `auto_settle` (already documented as no-auth) |

**Client-side missing auth headers** — add `X-Daglock-*` headers:

| Bot method | File | Line ~ |
|------------|------|--------|
| `cancelEscrow` | `bot/src/lib/api.js` | 136 |
| `swapEscrow` | `bot/src/lib/api.js` | 142 |
| `releaseMilestone` | `bot/src/lib/api.js` | 326 |
| `refundMilestone` | `bot/src/lib/api.js` | 328 |
| `completeMilestone` | `bot/src/lib/api.js` | 330 |
| `refundMultiEscrow` | `bot/src/lib/api.js` | 360 |
| `swapMultiEscrow` | `bot/src/lib/api.js` | 366 |
| `drawSubscription` | `bot/src/lib/api.js` | 487 |

**Bot command handlers** that need real signing flow:
- `index.js` around line 277: `handleSettle` — needs real Schnorr sig
- `index.js` around line 281: `handleRefund` — needs real Schnorr sig
- `index.js` around line 710+: `handleCancelEscrow` — needs real Schnorr sig

---

### C4 — Require V2 Messages (No Replay) on All Authenticated Endpoints

**Problem:** `parse_message()` accepts V1 format (`action:id`) with no nonce/timestamp, enabling signature replay.

**Fix — `indexer/src/auth.rs`:**

In `parse_message()`, add a `require_v2: bool` parameter. When true (production mode), reject 2-part messages:

```rust
pub(crate) fn parse_message(message: &str, require_v2: bool) -> AuthResult<ParsedMessage> {
    let parts: Vec<&str> = message.split(':').collect();

    if parts.len() == 4 {
        // Version 2: action:id:timestamp:nonce_hex — full replay protection
        // ... existing code ...
    } else if parts.len() == 2 {
        if require_v2 {
            return Err(AuthError::InvalidMessage {
                detail: "Version 1 messages are not accepted on mainnet. \
                         Use format 'action:id:timestamp:nonce_hex'".to_string()
            });
        }
        // Version 1 (legacy) — no replay protection
        // ... existing code ...
    }
}
```

Then update all callers on mainnet paths to pass `true`:
- `verify_settle_authorization` (line 439)
- `verify_refund_authorization` (line 494)
- `verify_cancel_authorization` (line 550)
- `dispute` handler in `escrows.rs` (line 620)
- `chat_pubkey` handler (line ~620 in escrows.rs)

For testnet, pass `false` for backward compatibility.

**Alternatively:** Simpler fix — just remove the V1 fallback entirely. All modern clients already use V2.

**Files to change:**
- `indexer/src/auth.rs` — remove V1 fallback or gate it
- `indexer/src/api/escrows.rs` — check all `parse_message` calls
- `indexer/src/services/escrow_service.rs` — check dispute handler

---

### C5 — WebSocket Auth

**Fix — `indexer/src/api/mod.rs`:**

Add Schnorr signature verification to the WebSocket upgrade handshake. Require an auth header or query parameter:

```rust
async fn websocket_handler(
    ws: axum::extract::WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<AppState>,
    // Require auth via query params:
    Query(auth_params): Query<WsAuthParams>,
) -> axum::response::Response {
    // Verify signature in query params
    let auth = AuthContext::from_query_params(&auth_params)?;
    // ... verify ...
    
    let rx = state.ws_tx.subscribe();
    ws.on_upgrade(move |socket| websocket::handle_socket(socket, state.db, rx))
}
```

**Alternative approach (simpler):** Subscribe only to events the caller is a participant in, by filtering events server-side.

**Files to change:**
- `indexer/src/api/mod.rs` — add auth to `websocket_handler`
- `indexer/src/websocket.rs` — add event filtering by address
- `web/src/hooks/useWebSocket.ts` — add auth header to WebSocket connection
- `bot/src/lib/api.js` — update WebSocket connections

---

## Phase 2: High (Mainnet Safety — Fix for Launch)

### H2 — Fix X-Forwarded-For IP Parsing

**Problem:** Rate limiter trusts `X-Forwarded-For` from the leftmost (attacker-controlled) IP.

**Fix — `indexer/src/ratelimit.rs`:**

```rust
let ip = req
    .headers()
    .get("x-forwarded-for")
    .and_then(|v| v.to_str().ok())
    .and_then(|v| {
        // Take the RIGHTMOST IP (the one added by the outermost trusted proxy)
        v.rsplit(',').next().map(|s| s.trim())
    })
    .and_then(|v| v.parse::<IpAddr>().ok())
    .or_else(|| {
        // Fall back to actual connection IP
        req.extensions()
            .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
            .map(|ci| ci.ip())
    })
    .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
```

**Additional fix:** Add `ConnectInfo` extraction in the router:

```rust
// In main.rs or router setup:
app = app.into_make_service_with_connect_info::<SocketAddr>();
```

**Files to change:**
- `indexer/src/ratelimit.rs` — fix IP parsing logic
- `indexer/src/api/mod.rs` — add `ConnectInfo` layer
- `indexer/src/main.rs` — update `axum::serve` to use `into_make_service_with_connect_info`

---

### H3 — Fix Jury Selection Modulo Bias

**Problem:** `rand::random::<usize>() % (i + 1)` introduces modulo bias.

**Fix — all 4 locations (`indexer/src/api/escrows.rs`, `indexer/src/services/escrow_service.rs`, `indexer/src/main.rs`):**

Replace:
```rust
let j = rand::random::<usize>() % (i + 1);
```

With:
```rust
let j = rand::thread_rng().gen_range(0..=i);
```

Or use `rand::seq::SliceRandom`:
```rust
use rand::seq::SliceRandom;
let selected: Vec<String> = candidate_pool
    .choose_multiple(&mut rand::thread_rng(), needed)
    .map(|j| j.address.clone())
    .collect();
```

**Files to change:**
- `indexer/src/api/escrows.rs` lines 710-713
- `indexer/src/services/escrow_service.rs` lines 350-353
- `indexer/src/main.rs` lines 292-294, 296 (jury escalation code)

---

### H4 — CSV Output Sanitization

**Problem:** User-controlled fields (memo, addresses, dispute_reason) are interpolated directly into CSV.

**Fix — `indexer/src/api/escrows.rs`:**

Use the `csv` crate or add proper field escaping:

```rust
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
// Then: csv_escape(e.memo.as_deref().unwrap_or(""))
```

Or switch to the `csv` crate writer for proper RFC 4180 compliance.

**Files to change:**
- `indexer/src/api/escrows.rs` — `export_csv` function
- `indexer/Cargo.toml` — add `csv` dependency (optional)

---

### H5 — Warn on Invalid DAGLOCK_MESSAGE_KEY

**Problem:** Invalid key format silently drops messages.

**Fix — `indexer/src/crypto.rs`:**

In `load_key_optional()`, log warnings instead of silent `None`:

```rust
fn load_key_optional() -> Option<[u8; 32]> {
    let hex_key = match env::var("DAGLOCK_MESSAGE_KEY") {
        Ok(k) => k,
        Err(_) => {
            warn!("DAGLOCK_MESSAGE_KEY not set — encrypted messages cannot be decrypted");
            return None;
        }
    };
    let bytes = match hex::decode(&hex_key) {
        Ok(b) => b,
        Err(e) => {
            warn!("DAGLOCK_MESSAGE_KEY is not valid hex: {e}");
            return None;
        }
    };
    if bytes.len() != 32 {
        warn!(
            "DAGLOCK_MESSAGE_KEY must decode to 32 bytes, got {}",
            bytes.len()
        );
        return None;
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Some(key)
}
```

**Files to change:**
- `indexer/src/crypto.rs` — add logging to `load_key_optional`

---

### H1/H6 — Remove Deterministic Dev Key & Require Real Verifier

**Problem:** Dev key fallback makes encryption pointless. MockVerifier bypasses all on-chain verification.

**Fix — `indexer/src/crypto.rs`:**

Remove the deterministic fallback. Crash if `DAGLOCK_MESSAGE_KEY` is not set:

```rust
fn load_key() -> [u8; 32] {
    match load_key_optional() {
        Some(k) => k,
        None => {
            panic!(
                "DAGLOCK_MESSAGE_KEY environment variable must be set. \
                 Generate one with: openssl rand -hex 32"
            );
        }
    }
}
```

**Fix — `indexer/src/main.rs`:**

Add a startup check that prevents `--no-wrpc` on non-local/dedicated dev networks:

```rust
// In args.validate() or in main():
if args.no_wrpc && args.network == "mainnet" {
    panic!("--no-wrpc is not allowed on mainnet. A real UTXO verifier is required.");
}
if args.no_wrpc && !matches!(args.network.as_str(), "simnet" | "devnet") {
    warn!(
        "--no-wrpc: UTXO verification is DISABLED. Escrows will not be verified on-chain. \
         This should only be used for local development."
    );
}
```

**Files to change:**
- `indexer/src/crypto.rs` — require env var
- `indexer/src/config.rs` — add `no_wrpc` + mainnet guard
- `indexer/src/verification.rs` — add panics/guards to MockVerifier

---

## Phase 3: Medium (Defense-in-Depth)

### M1 — Rate Limiter Memory Bound

**Fix — `indexer/src/ratelimit.rs`:**

Add a maximum map size with LRU-style eviction when exceeded:

```rust
const MAX_WINDOW_ENTRIES: usize = 100_000;

struct RateLimiterInner {
    windows: LruCache<IpAddr, WindowState>,
    tier_cache: HashMap<Vec<u8>, (ApiTier, Instant)>,
}
```

Use the `lru` crate or manual eviction when the map exceeds MAX_WINDOW_ENTRIES.

**Files to change:**
- `indexer/src/ratelimit.rs`
- `indexer/Cargo.toml` — add `lru` dependency (optional)

---

### M2 — Increase Nonce to 32 Bytes

**Fix — `indexer/src/auth.rs`:**

```rust
const NONCE_LENGTH: usize = 32;
const NONCE_HEX_LENGTH: usize = 64;
```

Update `generate_nonce()` to use `blake2b_simd::Params::new().hash_length(32)`.

This changes the wire format — all clients must upgrade. Consider adding a `version` field to the message to negotiate.

**Files to change:**
- `indexer/src/auth.rs`
- `web/src/crypto/chat-crypto.ts` (if messages reference nonces)
- `cli/src/commands/*.rs` (if they generate messages manually)
- `bot/src/lib/api.js` (if bot generates messages)

---

### M3 — Don't Store Secret Server-Side After Swap Generation

**Fix — `indexer/src/api/swap.rs`:**

The swap generation endpoint should:
1. Generate secret + hash
2. Return both to caller
3. **Immediately zeroize** the secret from server memory
4. Store only the hash in the escrow

```rust
pub async fn generate() -> Json<Value> {
    let secret = generate_random_secret();
    let hash = sha256(&secret);
    let response = json!({ "secret": hex::encode(&secret), "hash": hex::encode(&hash) });
    // Zeroize secret from memory
    zeroize::Zeroize::zeroize(&mut secret);
    Json(response)
}
```

**Files to change:**
- `indexer/src/api/swap.rs`
- `indexer/Cargo.toml` — add `zeroize` dependency

---

### M4 — Per-Address Rate Limiting on State-Changing Operations

**Fix — `indexer/src/services/escrow_service.rs`:**

Add per-address rate limit checks in `settle`, `refund`, `dispute`, `cancel`:

```rust
// Max 10 state changes per address per 5 minutes
const MAX_STATE_CHANGES: u32 = 10;
const STATE_WINDOW_SECS: i64 = 300;

let recent = queries::count_recent_state_changes(&self.db, &auth.address, STATE_WINDOW_SECS)
    .await
    .map_err(|_| ServiceError::Internal("Rate limit check failed".into()))?;
if recent >= MAX_STATE_CHANGES {
    return Err(ServiceError::Forbidden(
        format!("Rate limit: max {MAX_STATE_CHANGES} state changes per {STATE_WINDOW_SECS}s")
    ));
}
```

**Files to change:**
- `indexer/src/services/escrow_service.rs`
- `indexer/src/db/queries/*.rs` — add `count_recent_state_changes`

---

## Phase 4: Low (Polish)

### L1 — Mark V1 as Deprecated in Docs and Logs

**Fix — `indexer/src/auth.rs`:**

Log a warning when V1 messages are used:
```rust
if parts.len() == 2 {
    warn!(
        "DEPRECATED: Version 1 message format used (action:id) — no replay protection. \
         Client should upgrade to 'action:id:timestamp:nonce_hex'"
    );
}
```

### L2 — Treasury Pubkey Required on Mainnet

**Fix — `indexer/src/config.rs`:**

```rust
if self.network == "mainnet" && self.treasury_pubkey.is_none() {
    panic!("--treasury-pubkey is required on mainnet");
}
```

### L4 — Move anchor_wallet_key to Env Var Instead of CLI Arg

**Fix — `indexer/src/config.rs`:**

```rust
#[arg(long, env = "DAGLOCK_ANCHOR_WALLET_KEY")]
pub anchor_wallet_key: Option<String>,
```

---

## Implementation Order Across Files

```
Phase 1 — Critical (blocking)
├── indexer/src/services/escrow_service.rs    (C1: atomic_swap auth)
├── indexer/src/api/escrows.rs                 (C1, C3: pass headers, C4 V2 messages)
├── indexer/src/api/milestones.rs              (C3: add auth to handlers)
├── indexer/src/api/subscriptions.rs           (C3: add auth to handlers)
├── indexer/src/api/multi_escrows.rs           (C3: add auth to handlers)
├── indexer/src/api/vaults.rs                  (C3: add auth to handlers)
├── indexer/src/auth.rs                        (C4: require V2 on mainnet)
├── indexer/src/api/mod.rs + websocket.rs      (C5: WS auth)
├── web/src/api.ts                             (C1, C3: add auth headers)
├── bot/src/lib/api.js                         (C1, C3: add auth headers)
├── bot/src/index.js                           (C3: real signing flow)

Phase 2 — High (mainnet safety)
├── indexer/src/ratelimit.rs                   (H2: X-Forwarded-For fix)
├── indexer/src/api/mod.rs + main.rs           (H2: ConnectInfo layer)
├── indexer/src/api/escrows.rs                 (H3: jury selection, H4: CSV)
├── indexer/src/services/escrow_service.rs     (H3: jury selection)
├── indexer/src/main.rs                        (H3: jury selection)
├── indexer/src/crypto.rs                      (H1: require env/h5: warn)
├── indexer/src/config.rs + verification.rs    (H6: no-wrpc guards + MockVerifier guard)

Phase 3 — Medium (defense in depth)
├── indexer/src/ratelimit.rs                   (M1: memory bound)
├── indexer/src/auth.rs                        (M2: 32-byte nonce)
├── indexer/src/api/swap.rs                    (M3: zeroize secret)
├── indexer/src/services/escrow_service.rs     (M4: per-address rate limit)
├── indexer/src/db/queries/                   (M4: new query)

Phase 4 — Low (polish)
├── indexer/src/auth.rs                        (L1: deprecation warnings)
├── indexer/src/config.rs                      (L2: treasury key required)
└── indexer/src/config.rs                      (L4: env var for wallet key)
```

---

## Per-Issue Checklist

Use this to track completion per file.

### `indexer/src/auth.rs`
- [ ] C4: Gate V1 format behind network type
- [ ] L1: Warn on V1 usage
- [ ] M2: Bump NONCE_LENGTH to 32

### `indexer/src/api/escrows.rs`
- [ ] C1: Pass `headers` to `atomic_swap`
- [ ] C3: Verify `auto_settle` auth for non-timeout paths
- [ ] C4: Require V2 in dispute handler
- [ ] H3: Fix modulo bias in jury selection
- [ ] H4: CSV field escaping

### `indexer/src/services/escrow_service.rs`
- [ ] C1: Add auth verification to `atomic_swap`
- [ ] H3: Fix modulo bias (2 locations)
- [ ] M4: Per-address rate limiting

### `indexer/src/ratelimit.rs`
- [ ] H2: Parse X-Forwarded-For from rightmost IP
- [ ] M1: Add LRU eviction

### `indexer/src/crypto.rs`
- [ ] H1/H5: Remove dev key fallback, add warnings
- [ ] Require env var

### `indexer/src/api/mod.rs`
- [ ] C5: WS auth
- [ ] H2: ConnectInfo layer

### `bot/src/lib/api.js`
- [ ] C1: Add auth to `swapEscrow`
- [ ] C3: Add auth headers to all state-changing methods

### `web/src/api.ts`
- [ ] C1: Add auth to `swapEscrow`
- [ ] C3: Add auth headers where missing

---

## Verification

Before closing each phase, run:

```bash
# 1. All tests pass
cargo test --workspace 2>&1 | tail -20

# 2. Web builds
cd web && npm run build 2>&1 | tail -10

# 3. Bot tests pass
cd bot && npm test 2>&1 | tail -10

# 4. No critical warnings
cargo clippy --workspace -- -D warnings 2>&1 | tail -10

# 5. Dependency audit
cargo audit 2>&1 | tail -10
```
