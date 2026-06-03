# Indexer

**Source**: `indexer/src/`  **Updated**: `2026-06-02`  (12 files)

## What it does
Rust backend that tracks DagLock escrows on-chain via wRPC, exposes a REST API for escrow CRUD, counterparty discovery (offer board), on-chain reputation, and settlement receipts. Uses SQLite (alpha) with SQLx.

## Architecture
```
Kaspa Node (wRPC) ──▶ listener.rs ──▶ db/queries.rs ──▶ SQLite
                                                │
REST API (Axum) ◀──── api/mod.rs ◀──────────────┘
    ├── escrows.rs    (CRUD + lifecycle + auth)
    ├── offers.rs     (counterparty discovery)
    ├── reputation.rs (on-chain stats)
    ├── receipts.rs   (settlement proof)
    └── network.rs    (chain info + fee estimate)

auth.rs          (signature verification)
verification.rs  (UTXO verification)
```

## Key functions / components
| Name | Kind | File:Line | Purpose |
|------|------|-----------|---------|
| `main()` | function | `indexer/src/main.rs` | Boot: parse args → init DB → spawn listener → start Axum |
| `build_router()` | function | `indexer/src/api/mod.rs` | Register all REST routes with Axum + CORS |
| `AppState` | struct | `indexer/src/api/mod.rs` | Shared state: DB pool, templates, verifier |
| `settle_escrow_atomic()` | function | `indexer/src/db/queries.rs` | Atomic settle (status + timestamp) |
| `refund_escrow_atomic()` | function | `indexer/src/db/queries.rs` | Atomic refund (status + timestamp) |
| `validate_kaspa_address()` | function | `indexer/src/api/escrows.rs` | Validate Kaspa address format |
| `SignatureVerifier` | trait | `indexer/src/auth.rs` | Swappable signature verification |
| `EscrowVerifier` | trait | `indexer/src/verification.rs` | Swappable UTXO verification |
| `AuthContext` | struct | `indexer/src/auth.rs` | Extract X-Daglock-* headers |

## Data flow
1. wRPC listener receives `BlockAdded` notifications (stub)
2. Template matcher scans tx outputs for DagLock template hashes
3. Detected UTXOs inserted into `escrows` table as `PendingConfirmation`
4. Reconciliation loop marks expired escrows (30s interval)
5. REST API serves escrow state to CLI/web/bot
6. Settle/refund endpoints require auth + UTXO verification
7. Atomic queries prevent race conditions
8. Receipt generated on settlement with verification data

## Security features
- **Atomic queries:** Settle/refund use `WHERE status = 'active'` — no race conditions
- **Address validation:** Buyer/seller addresses validated on create
- **Authentication:** Lifecycle endpoints require X-Daglock-* headers
- **CORS:** Configured for browser access
- **Fee verification:** Receipts verify 0.5% fee calculation

## Edge cases & gotchas
- Alpha uses SQLite — swap to PostgreSQL for > 50 concurrent users
- wRPC listener is a stub — runs reconciliation only
- Mock verifiers in place — no real crypto yet
- Reconciliation runs every 30s — not real-time expiration
- Duplicate UTXOs (same params, different tx) are separate escrow instances

## Testing strategy
| Aspect | Approach |
|--------|----------|
| Unit tests | Type serialization, status conversions, reputation formula |
| Integration tests | DB migrations, escrow CRUD, lifecycle transitions |
| Edge case tests | Atomic settle/refund, address validation, fee calculation |
| Run command | `cargo test -p daglock-indexer` |

## Dependencies
| Depends on | For |
|------------|-----|
| `axum` | HTTP framework |
| `tower-http` | CORS middleware |
| `sqlx` (SQLite) | Database |
| `tokio` | Async runtime |
| `kaspa-wrpc-client` | Node communication |
| `blake2b_simd` | Template hash matching |

## Consumed by
| Consumer | How |
|----------|-----|
| `cli` | HTTP REST calls |
| `web` | HTTP REST calls (via Vite proxy) |
| `bot` | HTTP REST calls |

## Related domains
| Domain | Doc | Relationship |
|--------|-----|--------------|
| contracts | `features/contracts.md` | Provides template hashes for UTXO detection |
| cli | `features/cli.md` | REST API consumer |
| web | `features/web.md` | REST API consumer |
| bot | `features/bot.md` | REST API consumer |
