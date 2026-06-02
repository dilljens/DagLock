# Indexer

**Source**: `indexer/src/`  **Updated**: `2026-06-02`  (10 files)

## What it does
Rust backend that tracks DagLock escrows on-chain via wRPC, exposes a REST API for escrow CRUD, counterparty discovery (offer board), on-chain reputation, and settlement receipts. Uses SQLite (alpha) with SQLx.

## Architecture
```
Kaspa Node (wRPC) ──▶ listener.rs ──▶ db/queries.rs ──▶ SQLite
                                                │
REST API (Axum) ◀──── api/mod.rs ◀──────────────┘
    ├── escrows.rs    (CRUD + lifecycle transitions)
    ├── offers.rs     (counterparty discovery)
    ├── reputation.rs (on-chain stats)
    ├── receipts.rs   (settlement proof)
    └── network.rs    (chain info + fee estimate)
```

## Key functions / components
| Name | Kind | File:Line | Purpose |
|------|------|-----------|---------|
| `main()` | function | `indexer/src/main.rs` | Boot: parse args → init DB → spawn listener → start Axum |
| `build_router()` | function | `indexer/src/api/mod.rs` | Register all REST routes with Axum |
| `AppState` | struct | `indexer/src/api/mod.rs` | Shared state: DB pool, templates, network config |
| `migrate()` | function | `indexer/src/db/schema.rs` | Run SQL migrations + idempotent ALTER TABLE |
| `spawn()` | function | `indexer/src/listener.rs` | Background reconciliation loop (30s interval) |
| `EscrowStatus` | enum | `indexer/src/types.rs` | Lifecycle: PendingConfirmation → Active → Settled/Refunded/Expired |
| `OfferStatus` | enum | `indexer/src/types.rs` | Offer lifecycle: Proposed → Accepted → Locked → Settled |
| `Reputation` | struct | `indexer/src/types.rs` | On-chain stats: trade count, volume, age, dispute rate |
| `Receipt` | struct | `indexer/src/types.rs` | Settlement proof with verification flags |

## Data flow
1. wRPC listener receives `BlockAdded` notifications
2. Template matcher scans tx outputs for DagLock template hashes (KAS + KRC-20)
3. Detected UTXOs inserted into `escrows` table as `PendingConfirmation`
4. Reconciliation loop marks expired escrows (`expiration_daa_score < current_daa_score`)
5. REST API serves escrow state to CLI/web/bot
6. User triggers lifecycle transitions via API (settle, refund, dispute, cancel)
7. Receipt generated on settlement with on-chain verification data

## Edge cases & gotchas
- Alpha uses SQLite — swap to PostgreSQL for > 50 concurrent users
- wRPC listener is optional (started only if `--wrpc-url` provided) — can run standalone with manual POST
- Reconciliation runs every 30s — not real-time expiration
- Duplicate UTXOs (same params, different tx) are separate escrow instances
- `ensure_escrow_lifecycle_columns()` — idempotent ALTER TABLE for schema evolution

## Testing strategy
| Aspect | Approach |
|--------|----------|
| Unit tests | Type serialization, status conversions |
| Integration tests | REST API endpoints against SQLite |
| Run command | `cargo test -p daglock-indexer` |

## Dependencies
| Depends on | For |
|------------|-----|
| `axum` | HTTP framework |
| `sqlx` (SQLite) | Database |
| `tokio` | Async runtime |
| `kaspa-wrpc-client` | Node communication |
| `blake2b_simd` | Template hash matching |

## Consumed by
| Consumer | How |
|----------|-----|
| `cli` | HTTP REST calls |
| `web` | HTTP REST calls |
| `bot` | HTTP REST calls |

## Related domains
| Domain | Doc | Relationship |
|--------|-----|--------------|
| contracts | `features/contracts.md` | Provides template hashes for UTXO detection |
| cli | `features/cli.md` | REST API consumer |
