# CLI

**Source**: `cli/src/`  **Updated**: `2026-06-05`  (10 files)

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

## Audit Findings (2026-06-06)

### High-Priority Usability Issues (Block Real Usage)

| ID | Finding | Location | Fix Required |
|----|---------|----------|--------------|
| **U1** | **CLI create uses dummy keys** — Hardcodes `buyer_key = [1u8; 32]`, `seller_key = [2u8; 32]`. Prints unsigned tx but keys are fake. User can't create valid escrows. | `commands/create.rs:17-22` | Remove hardcoded keys. Integrate `kaspawallet sign --transaction <hex>` subprocess. |
| **U3** | **No wallet integration in CLI** — `assemble_create_escrow()` returns unsigned tx hex but no `kaspawallet sign` invocation or KasWare integration. Manual copy-paste required. | `tx.rs`, `commands/create.rs` | Add `cli/src/wallet.rs` with `sign_with_kaspawallet()`. Use in create, claim, refund, swap, vault withdraw. |
| **Q1** | **`.unwrap()` in production** — `commands/create.rs:46` unwraps hex decode. | `commands/create.rs:46` | Replace with proper error handling. |

### Code Quality Issues

| ID | Finding | Impact |
|----|---------|--------|
| **Q2/Q3** | Magic number `200` in `tx.rs` (fee calculation) — no shared constant | Use `shared::constants::FEE_DENOMINATOR` |
| **Q4** | `trade_hash` handling — CLI accepts optional string, no validation | Use `shared::validation::validate_trade_hash` |

### Fix Plan (Phase 2 — Usability)

1. **Task 9 (U1):** CLI create with real wallet keys — `kaspawallet sign` subprocess
2. **Task 11 (U3):** CLI wallet module (`cli/src/wallet.rs`) — shared signing logic for all commands
3. **Task 25 (Q1):** Remove `.unwrap()` in production code
4. **Task 26 (Q2/Q3):** Use shared `FEE_DENOMINATOR` constant everywhere
5. **Task 27 (Q4):** Use `TradeHash` newtype with `FromStr` validation

### Dependencies

- Requires `shared` crate (Phase 0, Task 1) for `FEE_DENOMINATOR` and `validate_trade_hash`
- Requires `kaspawallet` binary installed on user's system
- Indexer must have real UTXO verification (S1) for settlement to work end-to-end

### Verification

- [ ] `cargo test -p daglock-cli` passes
- [ ] Manual: `daglock-cli create --amount 100 --counterparty <addr>` → prompts for wallet → `kaspawallet sign` → broadcasts → settle via CLI
- [ ] Manual: `daglock-cli swap --id <id> --preimage <hex>` → signs → broadcasts → settles
- [ ] Manual: `daglock-cli vault withdraw --id <id>` → signs → broadcasts → unlocks

