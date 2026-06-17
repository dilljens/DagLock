# CLI

**Source**: `cli/src/`  **Updated**: `2026-06-16`  (14 files — 4 src + 10 commands)

## What it does
Command-line power-user tool for DagLock escrow operations. Connects to the indexer REST API for queries and assembles unsigned transactions for signing with kaspawallet or KasWare.

## Architecture
```
main.rs (clap dispatch)
    
     commands/create.rs     POST /v1/escrows
     commands/claim.rs      POST /v1/escrows/:id/settle, /refund, /dispute, /cancel
     commands/offer.rs      POST /v1/offers, /accept, /cancel
     commands/status.rs     GET /v1/escrows/:id
     commands/reputation.rs  GET /v1/reputation/:address
     commands/receipt.rs    GET /v1/receipts/:id
     commands/message.rs   POST/GET messages
     commands/swap.rs       POST /v1/escrows/:id/swap (atomic swap settle)
     commands/vault.rs     POST/GET /v1/vaults, /vaults/:id/withdraw
    
     tx.rs                  Transaction assembly (unsigned)
```

## Key functions / components
| Name | Kind | File:Line | Purpose |
|------|------|-----------|---------|
| `Commands` | enum | `cli/src/main.rs` | All CLI subcommands (Create, Claim, Refund, Dispute, Cancel, Swap, Vault, Offer, Status, Reputation, Receipt, Msg, Messages, Config) |
| `OfferCommands` | enum | `cli/src/main.rs` | Offer subcommands (List, Create, Accept, Cancel) |
| `VaultCommands` | enum | `cli/src/main.rs` | Vault subcommands (Create, List, Get, Withdraw) |
| `commands::create::run()` | function | `cli/src/commands/create.rs` | Create escrow proposal via API |
| `commands::claim::run()` | function | `cli/src/commands/claim.rs` | Claim/release escrow |
| `commands::swap::run()` | function | `cli/src/commands/swap.rs` | Atomic swap settle via preimage |
| `commands::vault::create()` | function | `cli/src/commands/vault.rs` | Create time-locked vault |
| `commands::vault::list()` | function | `cli/src/commands/vault.rs` | List vaults by owner |
| `commands::vault::get()` | function | `cli/src/commands/vault.rs` | Get vault details |
| `commands::vault::withdraw()` | function | `cli/src/commands/vault.rs` | Withdraw from vault |
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

---

## Audit Status (2026-06-06 → 2026-06-16)

All CLI audit items are now **resolved**.

### Previously Blocking Issues (Now Fixed)

| ID | Issue | Status | Fix |
|----|-------|--------|-----|
| **U1** | CLI create uses dummy keys | ✅ Fixed | `cli/src/wallet.rs` with `sign_with_kaspawallet()` subprocess. `commands/create.rs` no longer hardcodes keys — user supplies via flag or kaspawallet. |
| **U3** | No wallet signing in CLI | ✅ Fixed | `cli/src/wallet.rs`: `sign_with_kaspawallet()`, `parse_hex_key()`, `kaspawallet_available()`. Used by create, claim, refund, swap, vault withdraw. |
| **Q1** | `.unwrap()` in production | ✅ Fixed | Zero `.unwrap()` calls remain in `cli/src/`. All replaced with proper error propagation. |
| **Q2/Q3** | Magic number `200` in `tx.rs` | ✅ Fixed | `shared::constants::FEE_DENOMINATOR` used throughout. |
| **Q4** | `trade_hash` not validated | ✅ Fixed | `daglock_shared::validate_trade_hash()` on all create paths. |

### Architecture

```
main.rs (clap dispatch)
    
     commands/create.rs     POST /v1/escrows (now with real wallet keys)
     commands/claim.rs      POST /v1/escrows/:id/settle, /refund, /dispute, /cancel
     commands/offer.rs      POST /v1/offers, /accept, /cancel
     commands/status.rs     GET /v1/escrows/:id
     commands/reputation.rs  GET /v1/reputation/:address
     commands/receipt.rs    GET /v1/receipts/:id
     commands/message.rs   POST/GET messages
     commands/swap.rs       POST /v1/escrows/:id/swap (atomic swap settle)
     commands/vault.rs     POST/GET /v1/vaults, /vaults/:id/withdraw
     commands/evidence.rs  POST /v1/evidence (dispute evidence)
    
     wallet.rs              sign_with_kaspawallet() subprocess
     tx.rs                  Transaction assembly (unsigned)
```

