# CLI

**Source**: `cli/src/`  **Updated**: `2026-06-02`  (7 files)

## What it does
Command-line power-user tool for DagLock escrow operations. Connects to the indexer REST API for queries and assembles unsigned transactions for signing with kaspawallet or KasWare.

## Architecture
```
main.rs (clap dispatch)
    │
    ├── commands/create.rs    ──▶ POST /v1/escrows
    ├── commands/claim.rs     ──▶ POST /v1/escrows/:id/settle, /refund, /dispute, /cancel
    ├── commands/offer.rs     ──▶ POST /v1/offers, /accept, /cancel
    ├── commands/status.rs    ──▶ GET /v1/escrows/:id
    ├── commands/reputation.rs ──▶ GET /v1/reputation/:address
    ├── commands/receipt.rs   ──▶ GET /v1/receipts/:id
    │
    └── tx.rs                 ──▶ Transaction assembly (unsigned)
```

## Key functions / components
| Name | Kind | File:Line | Purpose |
|------|------|-----------|---------|
| `Commands` | enum | `cli/src/main.rs` | All CLI subcommands (Create, Claim, Refund, Dispute, Cancel, Offer, Status, Reputation, Receipt, Config) |
| `OfferCommands` | enum | `cli/src/main.rs` | Offer subcommands (List, Create, Accept, Cancel) |
| `commands::create::run()` | function | `cli/src/commands/create.rs` | Create escrow proposal via API |
| `commands::claim::run()` | function | `cli/src/commands/claim.rs` | Claim/release escrow |
| `commands::offer::list()` | function | `cli/src/commands/offer.rs` | Browse open offers |

## Data flow
1. User invokes `daglock-cli <command>` with args
2. `clap` parses args → dispatches to `commands::<module>::run()`
3. Command function calls indexer REST API (HTTP)
4. Response displayed to user (human-readable)
5. For signing: unsigned tx returned, user signs via kaspawallet/KasWare

## Edge cases & gotchas
- CLI never signs transactions — it assembles unsigned tx for external wallet
- Amounts: user provides KAS (decimal), internal uses sompi (integer) — conversion needed
- Config stored locally (`daglock config --api-url ...`)

## Testing strategy
| Aspect | Approach |
|--------|----------|
| Unit tests | Arg parsing, config handling |
| Integration tests | End-to-end against running indexer |
| Run command | `cargo test -p daglock-cli` |

## Dependencies
| Depends on | For |
|------------|-----|
| `clap` (derive) | CLI argument parsing |
| `reqwest` | HTTP client for REST API |
| `tokio` | Async runtime |
| `anyhow` | Error handling |
| `tracing` | Logging |

## Consumed by
| Consumer | How |
|----------|-----|
| Power users | Direct terminal usage |
| Scripts/automation | Non-interactive invocation |

## Related domains
| Domain | Doc | Relationship |
|--------|-----|--------------|
| indexer | `features/indexer.md` | REST API consumer |
| contracts | `features/contracts.md` | Transaction assembly |
