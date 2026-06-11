# Indexer

**Source**: `indexer/src/`  **Updated**: `2026-06-09`  (30+ files)

## What it does
Rust backend serving the DagLock REST API. Handles escrow lifecycle (create, settle, refund, dispute), offer board, reputation, vaults, jury, encrypted messaging, app registration, webhook dispatch, and WebSocket real-time updates. Uses SQLite or PostgreSQL via SQLx.

---

## Integrator Infrastructure

DagLock exposes a programmatic REST API for partners and integrators. No signup fee — just free public API + on-chain 0.5% covenant fee.

### App Registration / API Keys

Register your project to get API keys for higher rate limits and webhook access.

**All management endpoints require `X-Daglock-Api-Key` header.** The key is SHA-256 hashed on storage — plaintext can't be recovered.

**Cross-app access is forbidden** — a key from App A cannot access App B's resources.

**Registration is unauthenticated** (you need to register before you have a key).

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/apps/register` | Register a new app. Returns `{ app, api_key, warning }` |
| GET | `/v1/apps/:id` | Get app details | `X-Daglock-Api-Key` required |
| GET | `/v1/apps/:id/keys` | List API keys for an app | `X-Daglock-Api-Key` required |
| POST | `/v1/apps/:id/keys` | Generate additional key | `X-Daglock-Api-Key` required |
| DELETE | `/v1/apps/:id/keys/:key_id` | Revoke an API key | `X-Daglock-Api-Key` required |

**Headers:** `X-Daglock-Api-Key <key>` for all management requests.

**Example:**
```bash
# Register
curl -X POST https://api.daglock.io/v1/apps/register \
  -H "Content-Type: application/json" \
  -d '{"name": "MyDex", "owner_address": "kaspa:..."}'

# Use the returned API key for subsequent requests
curl https://api.daglock.io/v1/apps/app_xyz \
  -H "X-Daglock-Api-Key: dl_sk_..."
```

**Important:** The API key is shown once on registration. Save it securely. Key hashes are SHA-256 stored in the database — plaintext keys cannot be recovered.

### Webhooks

Subscribe to lifecycle events and receive HTTP callbacks.

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/apps/:id/webhooks` | Register a webhook for an event type |
| GET | `/v1/apps/:id/webhooks` | List registered webhooks |
| DELETE | `/v1/apps/:id/webhooks/:hook_id` | Remove a webhook |

**Supported events:**
- `escrow.created` — new escrow proposal
- `escrow.settled` — escrow completed successfully
- `escrow.refunded` — escrow refunded to buyer
- `escrow.disputed` — dispute raised
- `escrow.cancelled` — escrow cancelled
- `escrow.expired` — timeout reached
- `offer.created`, `offer.accepted`

**Delivery:** HTTP POST with `X-Daglock-Webhook-Id` and `X-Daglock-Webhook-Timestamp` headers. 3 retries with exponential backoff (1s, 4s, 10s).

**Example webhook payload:**
```json
POST https://partner.com/webhooks/daglock
X-Daglock-Webhook-Id: whd_abc
X-Daglock-Webhook-Timestamp: 1717203600

{
  "event": "escrow.settled",
  "created_at": 1717203600,
  "data": { "id": "esc_xyz" }
}
```

### Telegram Identity Verification

Link a Telegram handle to a Kaspa address for reputation context.

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/identity` | Link Telegram handle to address (requires signed message) |

---

## Price-Locked Offers (v0.2.0)

Market-price offers that auto-update via CoinGecko.

### How it works
- Offers can be `fixed` (creator sets exact price) or `market` (price from CoinGecko)
- Market-priced offers store a `price_offset` (optional ±%), `min_price`, and `max_price`
- The offline reconciliation loop fetches KAS/USD from CoinGecko every 15 minutes
- All market-priced offers are updated in the database
- At ~2,880 calls/month, this easily fits within CoinGecko's free tier (10,000/month)

### Backend
- `migrations/010_price_locked_offers.sql` — adds price columns to offers table
- `listener.rs` — `update_market_prices()` fetches price and updates DB
- `offers.rs` — creation handler fetches initial market price

### Frontend
- CreateOfferForm — price type selector, offset, and bounds
- OfferCard — shows current market price for market-priced offers

## On-Chain Verification (v0.3.0)

UTXO existence checks are performed at settlement/refund time via wRPC.
No block scanning needed — the user initiates every action.

### How it works
- When `--wrpc-url` is provided, the indexer connects to a Kaspa node and creates a `WrpcVerifier`
- When a user calls `POST /v1/escrows/:id/settle`, the verifier checks the UTXO exists on-chain
- When a user calls `POST /v1/escrows/:id/refund`, same verification occurs
- If wRPC is unavailable, falls back to `MockVerifier` (dev mode, always succeeds)
- Expired escrows are reconciled via DAA score polling (requires wRPC connection)

### Lifecycle
```
CREATE -> pending_confirmation
  | (user broadcasts lock tx, then calls settle/refund)
SETTLE / REFUND -> verifier checks UTXO on-chain -> finalized
  (or rejected if UTXO not found)
```
Escrows in `pending_confirmation` are no longer blocked from settling.
The on-chain verification at settlement time replaces the need for
a state machine that tracks pending -> active transitions.

### Configuration
```bash
daglock-indexer \
  --host 0.0.0.0 \
  --port 8543 \
  --database-url sqlite:/data/daglock.db \
  --wrpc-url wss://testnet-nodes.kaspa.com \
  --daglock-kas-template <hex-hash> \
  --daglock-krc20-template <hex-hash>
```

### Template Hashes
Template hashes are generated by compiling the covenants:

```bash
cargo test -p daglock-contracts -- --nocapture print_template_hashes
```

This outputs:
- `daglock_kas_template_hash`
- `daglock_arbiter_template_hash`
- `daglock_krc20_template_hash`
- `daglock_vault_template_hash`

### Files
| File | Purpose |
|------|---------|
| `verification.rs` | `WrpcVerifier`, `MockVerifier`, `EscrowVerifier` trait |
| `main.rs` | Wires `WrpcVerifier` when `--wrpc-url` is set |
| `listener.rs` | DAA polling for expiry, market price updates |
| `db/queries/escrows.rs` | `try_find_escrow_by_lock_tx()`, `update_escrow_status_only()` |

## wRPC Listener (v0.2.0)

Background tasks run when connected to a Kaspa node.

### How it works
- DAA score polled every 10 seconds for expired escrow reconciliation
- Market prices updated from CoinGecko every 15 minutes (offline fallback)
- Does NOT scan blocks for UTXO detection — verification happens at settlement time

## Replay Protection (v0.3.0)

All authenticated actions (settle, refund) use signed messages with replay protection.

### Message Format
```
v1 (legacy):    {action}:{escrow_id}
v2 (current):   {action}:{escrow_id}:{timestamp}:{nonce_hex}
```

- `timestamp` must be within 5 minutes of server clock
- `nonce` is a 20-byte BLAKE2b-160 hex string (40 hex chars)
- Nonces are stored in the `auth_nonces` table — replay attacks are rejected
- Nonce auto-cleanup after 5 minutes

### CLI Helper
```rust
use daglock_indexer::auth::generate_replay_protected_message;
let msg = generate_replay_protected_message("settle", "esc_123");
// Result: "settle:esc_123:1717203600:a1b2c3d4..."
```

## Auth System

- Schnorr signature verification via `SchnorrVerifier` in `auth.rs`
- Auth headers: `X-Daglock-Address`, `X-Daglock-Signature`, `X-Daglock-Message`
- Actions require signed messages: `settle:id`, `refund:id`, `dispute:id`, `cancel:id`
- Default: real Schnorr verification (`--mock-auth` defaults to `false`)
- Panics on startup if `--mock-auth` is combined with `--network mainnet`

### CORS
- Default: `https://daglock.com` (set in `config.rs`)
- Dev: `*` via `--cors-origin *`

## Real Transaction Flow (v0.3.1)

The indexer no longer fabricates lock transaction IDs. Every escrow references a real on-chain UTXO.

### Flow
1. Compile covenant → get P2SH address
2. Fund the P2SH address (send KAS to covenant address)
3. Submit the real tx_id + output_index to the indexer
4. Indexer verifies UTXO exists on-chain at settlement time

### Web Flow
- `window.kasware.getPublicKey()` → buyer's public key
- POST `/v1/compile` with buyer/seller keys → covenant `script_hex` + `covenant_address`
- `window.kasware.sendKaspa(covenant_address, amount)` → real `tx_id`
- POST `/v1/escrows` with real `tx_id`

### CLI Flow
- `kaspawallet keys --show` → get private keys
- Compile covenant → get `covenant_address`
- `kaspawallet send --to <covenant_address>` → real `tx_id`
- POST `/v1/escrows` with real `tx_id`

---

## API Reference

### Escrow Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/v1/escrows?address=<addr>` | No | List escrows for an address |
| GET | `/v1/escrows/:id` | No | Get escrow details |
| POST | `/v1/escrows` | No | Create escrow proposal |
| GET | `/v1/escrows/:id/lock-status` | No | Check if UTXO is confirmed on-chain |
| POST | `/v1/escrows/:id/settle` | Yes | Settle escrow (buyer or seller) |
| POST | `/v1/escrows/:id/refund` | Yes | Refund escrow (buyer only) |
| POST | `/v1/escrows/:id/dispute` | No | Raise a dispute |
| POST | `/v1/escrows/:id/cancel` | Yes | Cancel escrow |
| POST | `/v1/escrows/:id/swap` | No | Atomic swap settle via hash preimage |
| GET | `/v1/stats` | No | Platform statistics |

### Offer Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/v1/offers` | No | List open offers |
| POST | `/v1/offers` | No | Create an offer |
| POST | `/v1/offers/:id/accept` | No | Accept an offer |
| POST | `/v1/offers/:id/cancel` | No | Cancel an offer |

### Vault Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/v1/vaults` | No | List vaults |
| POST | `/v1/vaults` | No | Create a vault |
| GET | `/v1/vaults/:id` | No | Get vault details |
| POST | `/v1/vaults/:id/withdraw` | Yes | Withdraw from vault |
| POST | `/v1/vaults/:id/transfer` | Yes | Transfer vault ownership |

### Jury Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/v1/jury/register` | Yes | Register as a juror |
| POST | `/v1/jury/unregister` | Yes | Unregister as a juror |
| GET | `/v1/jury/cases` | Yes | List assigned cases |
| GET | `/v1/jury/cases/active/:address` | No | Active case count for badge |
| GET | `/v1/jury/cases/:id` | No | Get case details |
| POST | `/v1/jury/cases/:id/vote` | Yes | Cast a vote |
| GET | `/v1/jury/candidates` | No | List registered jurors |

### Other Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/v1/health` | No | Health check — pings DB, returns `db_connected` status |
| GET | `/v1/status` | No | Platform status + uptime |
| GET | `/v1/network` | No | Network info (DAA, difficulty) |
| GET | `/v1/network/price` | No | KAS/USD price |
| GET | `/v1/fees/estimate` | No | Fee estimate |
| POST | `/v1/compile` | No | Compile SilverScript covenant (dev only) |
| POST | `/v1/swap/generate` | No | Generate atomic swap secret/hash |
| GET | `/v1/openapi.json` | No | OpenAPI spec |
| GET | `/v1/reputation/:address` | No | Reputation score |
| GET | `/v1/receipts/:id` | No | Settlement receipt |
| POST | `/v1/vouches` | Yes | Vouch for an address |
| GET | `/v1/vouches` | No | List vouches |
| POST | `/v1/vouches/:id` | Yes | Revoke a vouch |
| POST | `/v1/identity` | Yes | Link Telegram handle |
| GET | `/v1/ws` | No | WebSocket real-time feed |

---

## Rate Limiting (v0.3.1)

All API endpoints have per-IP rate limiting via a custom Axum middleware.

### Configuration
- **30 requests per minute** per IP (configurable via `RateLimiter::new(max_requests, window_secs)`)
- Window resets fully after 60 seconds
- Different IPs have independent counters
- Uses `X-Forwarded-For` header for IP detection (works behind reverse proxy)

### When Exceeded
```json
HTTP 429 Too Many Requests
{
  "error": "rate_limited",
  "message": "Rate limit exceeded. Max 30 requests per 60 seconds."
}
```

### Key Files
| File | Purpose |
|------|---------|
| `indexer/src/ratelimit.rs` | `RateLimiter` struct + Axum middleware |
| `indexer/src/api/mod.rs` | Wired via `.route_layer()` on the main router |

---

## Security

### Auth System
- Schnorr signature verification via `SchnorrVerifier` in `auth.rs`
- Auth headers: `X-Daglock-Address`, `X-Daglock-Signature`, `X-Daglock-Message`
- Actions require signed messages: `settle:id`, `refund:id`, `dispute:id`, `cancel:id`

### CORS
- Default: `https://daglock.com` (set in `config.rs`)
- Dev: `*` via `--cors-origin *`

### Best Practices
- No `.unwrap()` in production code — all panics removed (2026-06-09 audit)
- All SQL queries use bind parameters (no string interpolation)
- Kaspa address validation on create
- Replay protection mandatory on all authenticated actions
- Rate limiting: 30 req/min per IP, returns HTTP 429
- API keys: `X-Daglock-Api-Key` required for app management, SHA-256 hashed at rest
- Address validation on all endpoints accepting Kaspa addresses
- `DAGLOCK_MESSAGE_KEY` checked at startup — panics on mainnet if unset
- MockVerifier for dev, WrpcVerifier for production
- Panics on startup if `--mock-auth` is combined with `--network mainnet`

---

## Audit Findings (2026-06-06)
