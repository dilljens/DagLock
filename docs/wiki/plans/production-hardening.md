# Production Hardening — Traffic & Attack Readiness

**Status:** Not started

**Date:** 2026-06-17

**Goal:** Make DagLock ready for real traffic and resistant to common attacks before mainnet launch (June 30).

---

## Threat Model

DagLock's indexer is a financial API. It holds no user funds (escrows are covenant-enforced on-chain), but it can be abused to:

| Attack | Impact | Severity |
|--------|--------|----------|
| Memory exhaustion (big POST body) | Crash the indexer, all users lose access | 🔴 Critical |
| Offer/escrow spam | DB filled with junk, legitimate offers buried | 🔴 High |
| Dispute spam | Moderation burden, legitimate disputes drowned out | 🟡 Medium |
| Sybil jury attack | Malicious jurors control verdicts | 🟡 Medium (gated by score) |
| Message storage abuse | Disk full, indexer crashes | 🟡 Medium |
| IP spoofing rate limit bypass | Attacker bypasses rate limits | 🟡 Medium |

---

## Priority 1: Quick Wins (Fix Today)

### 1.1 — nginx request body size limit (5 min)

**Problem:** No `client_max_body_size` in nginx config. Attacker can send a 1GB POST and exhaust VPS memory.

**Fix:** Add to the nginx `server` block:
```nginx
client_max_body_size 1m;
```

**Files:** `nginx.conf`, VPS `/etc/nginx/sites-available/api.daglock.com`

### 1.2 — Daily creation cap per address (30 min)

**Problem:** No limit on how many offers/escrows a single address can create. Rate limit helps but 30/min is still 43,200/day.

**Fix:** Add a query check in the create handlers:
```rust
// Before inserting, check daily count
let recent = queries::count_recent_escrows(&db, &buyer_address, 86400).await?;
if recent >= 50 {
    return Err(too_many_requests("Max 50 escrows per day per address"));
}
```

**Files:** `indexer/src/api/escrows.rs`, `indexer/src/api/offers.rs`, `indexer/src/db/queries/escrows.rs`, `indexer/src/db/queries/offers.rs`

### 1.3 — Message/evidence size limits (15 min)

**Problem:** No limit on message or evidence content length. Attacker can store unlimited text in the DB.

**Fix:** Add max length checks:
```rust
const MAX_MESSAGE_LENGTH: usize = 4096;
const MAX_EVIDENCE_LENGTH: usize = 10000;

if body.content.len() > MAX_MESSAGE_LENGTH {
    return Err(too_long(MAX_MESSAGE_LENGTH));
}
```

**Files:** `indexer/src/api/messages.rs`, `indexer/src/api/evidence.rs`

---

## Priority 2: Before Mainnet Announcement

### 2.1 — SQLite connection pool tuning (30 min)

**Problem:** Default pool size (typically 1-4) bottlenecks concurrent users.

**Fix:** Configure pool for production load:
```rust
let pool = SqlitePoolOptions::new()
    .max_connections(10)
    .min_connections(2)
    .acquire_timeout(Duration::from_secs(5))
    .connect(&url).await?;
```

**File:** `indexer/src/db/mod.rs`

### 2.2 — API key rate limit tiers (2-3 hours)

**Problem:** API key holders have the same rate limit as unauthenticated users.

**Fix:** Modify the rate limiter to check for `X-Daglock-Api-Key` header and apply higher limits:
```rust
fn check(&self, ip: IpAddr, api_key: Option<&str>) -> Result<(), Response> {
    let max = if api_key.is_some() { 300 } else { 30 };
    // ... existing logic with configurable max ...
}
```

**File:** `indexer/src/ratelimit.rs`

### 2.3 — Offer expiry enforcement (30 min)

**Problem:** Offers can stay on the board forever. The bot auto-cleans after 1 hour, but there's no server-side enforcement.

**Fix:** Add an `expires_at` field to offers and a background cleanup:
```rust
// In listener.rs reconciliation loop
queries::expire_stale_offers(&db, now()).await?;
```

**File:** `indexer/src/db/queries/offers.rs`, `indexer/src/listener.rs`

---

## Priority 3: Post-Launch

### 3.1 — Postgres migration

**Problem:** SQLite doesn't handle concurrent writes well. At scale, Postgres is necessary.

**Fix:** The `--db-type postgres` flag already exists. Needs testing, connection string format docs, and a migration path from SQLite.

### 3.2 — Redis caching

**Problem:** Every API call queries the DB. Offer board, reputation, and stats are read-heavy.

**Fix:** Add optional Redis cache:
```rust
// Cache offer board for 30 seconds
let cache_key = format!("offers:{}", status);
if let Some(cached) = redis.get(&cache_key).await? {
    return Ok(deserialize(cached));
}
let offers = queries::list_offers(&db).await?;
redis.set_ex(cache_key, serialize(&offers), 30).await?;
```

### 3.3 — Permanent IP ban list

**Problem:** Repeat attackers cycle through IPs. Rate limiter resets every 60 seconds.

**Fix:** Add a ban threshold (e.g., 10 rate limit violations in 1 hour = permanent ban for 24h). Store in DB.

### 3.4 — Horizontal scaling

**Problem:** Single VPS, single process. No redundancy.

**Fix:** Load balancer → multiple indexer instances → shared Postgres + Redis. Requires session affinity for WebSocket.

---

## Implementation Status

### Priority 1 (Completed June 17)
- [x] 1.1 — nginx `client_max_body_size 1m` on VPS
- [x] 1.2 — Daily creation cap (50 escrows, 50 offers per address)
- [x] 1.3 — Message/evidence max length enforcement (already existed: 1024 msg, 100KB evidence)

### Priority 2 (Completed June 17)
- [x] 2.1 — SQLite pool tuning (max 10 connections, min 2, 5s acquire timeout)
- [x] 2.2 — API key rate limit tiers (30 req/min default, 300 req/min with X-Daglock-Api-Key)
- [ ] 2.3 — Offer expiry enforcement (pending — trade bot handles cleanup client-side)

### Priority 3 (Post-launch Q3 2026 — deferred)
- [ ] 3.1 — Postgres migration
- [ ] 3.2 — Redis caching
- [ ] 3.3 — Permanent IP ban list
- [ ] 3.4 — Horizontal scaling
