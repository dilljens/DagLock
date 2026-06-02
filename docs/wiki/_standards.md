# Coding Standards

## Rules — What you must NEVER do

Breaking these causes bugs hard to find.

| # | Rule | Why | Check |
|---|------|-----|-------|
| 1 | Never `.unwrap()` outside `#[cfg(test)]` | Panics crash the indexer/CLI in production | `grep -rn '\.unwrap()' indexer/src/ cli/src/` should only hit test code |
| 2 | Never hardcode addresses/keys in covenant source | All params must come from constructor — covenants are parameterized | `daglock.sil` and `daglock_krc20.sil` contain no literal addresses |
| 3 | Never skip fee validation in release/swap paths | Treasury output must be checked — underpaying fees breaks protocol economics | Every `release`/`swap` entrypoint checks `outputs[1].value == feeAmount` |
| 4 | Never expose private keys in bot/CLI/WASM | Only unsigned tx assembled — signing happens in KasWare/kaspawallet | No `signTransaction` calls outside wallet integration |
| 5 | Never change the fee denominator (200) without updating all paths | Fee is hardcoded in compiled bytecode — inconsistent values = broken covenant | `inputValue / 200` appears in all 3 entrypoints of both `.sil` files |

## Practices — How to write NEW code

Standards for code you add or refactor. Existing code may not comply.

### Error handling
- Use `thiserror` for custom error types in Rust crates.
- Propagate errors with `?` and context (`anyhow` in binaries, `thiserror` in libraries).
- Never discard errors silently — log or propagate.

### API types
- All REST API types: `#[derive(Debug, Clone, Serialize, Deserialize)]`
- All enum variants: `#[serde(rename_all = "snake_case")]`
- Use `ApiError::new(code, message)` for error responses.

### Database
- Migrations: `include_str!("migrations/NNN_name.sql")` in `schema.rs`.
- New columns: add `ensure_*` function with `PRAGMA table_info` check for idempotent ALTER TABLE.
- Queries: return `Result<T, sqlx::Error>` — map to API errors in handler.

### CLI
- Subcommands: `clap` derive macros with `#[derive(Subcommand)]` enum.
- Each command: separate file in `cli/src/commands/<name>.rs`.
- Functions: `pub async fn run(api_url: String, ...) -> anyhow::Result<()>`.

### Testing
- Unit tests: `#[cfg(test)] mod tests` inline in source file.
- Integration tests: `tests/` directory in crate root.
- Contract tests: verify all spending paths + negative cases.
- Template hash tests: verify determinism and 20-byte length.

### Code clarity
- Comments explain WHY not WHAT.
- Delete dead code. No magic numbers.
- Naming: reveal intent at call site. Booleans: `is_*`, `has_*`, `can_*`.

### Concurrency
- Use `tokio::spawn` for background tasks (reconciliation loop, listener).
- Pass `Pool<Sqlite>` by clone (it's `Arc` internally).
- No blocking calls in async context.

## Patterns — How code IS written

Detected conventions. Match these so new code fits in.

### Error handling
**Pattern**: `thiserror` enums + `ApiError::new()` wrapper  **Example**: `indexer/src/types.rs`
**Rule**: Domain errors → thiserror; HTTP errors → ApiError with code + message

### Module structure
**Import style**: `mod.rs` barrel pattern in indexer (`api/mod.rs`, `db/mod.rs`)
**Barrel / re-export pattern**: Public items re-exported from `mod.rs`
**One export per file**: no — modules group related handlers (e.g., `api/escrows.rs` has list, create, get_by_id, settle, refund, dispute, cancel, stats)

### Type conventions
| Kind | Convention | Example |
|------|-----------|---------|
| ID types | `type X = String` newtype | `EscrowId`, `OfferId`, `Address`, `TxId` |
| Status enums | String-serializable with `as_str()`/`from_str()` | `EscrowStatus`, `OfferStatus` |
| API responses | `#[derive(Serialize)]` struct | `EscrowListResponse`, `OfferListResponse` |
| API requests | `#[derive(Deserialize)]` struct | `CreateEscrowRequest`, `CreateOfferRequest` |

### Naming
| Kind | Style | Example |
|------|-------|---------|
| Files | snake_case | `daglock_execution_tests.rs`, `schema.rs` |
| Functions | snake_case | `compile_daglock()`, `template_parts_and_hash()` |
| Constants | SCREAMING_SNAKE | `RELEASE`, `SWAP`, `REFUND` |
| SQL migrations | `NNN_name.sql` | `001_create_escrows.sql` |

### Concurrency
**Pattern**: `tokio::spawn` for background loops  **Example**: `indexer/src/listener.rs`
**Rule**: Background tasks receive owned data (`String`, `Pool<Sqlite>` clone). No references across spawn boundaries.

### Contract source
**Pattern**: `pragma silverscript ^0.1.0` + constructor params as `byte[32]`/`int`  **Example**: `contracts/src/daglock.sil`
**Rule**: Constructor uses `byte[32]` (not `pubkey`) to work around compiler strict typing. Cast to `pubkey()` inside function bodies.
