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
| 6 | Never use non-atomic updates for lifecycle transitions | Race conditions can settle/refund the same escrow twice | Use `settle_escrow_atomic()` or `refund_escrow_atomic()` |
| 7 | Never skip address validation on create | Invalid addresses stored in DB cause failed settlements | Validate with `validate_kaspa_address()` |

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
- Lifecycle transitions: use atomic queries with `WHERE status = 'active'`.

### Authentication
- Lifecycle endpoints require `X-Daglock-*` headers.
- Use `AuthContext::from_headers()` to extract auth context.
- Use `verify_settle_authorization()` or `verify_refund_authorization()`.
- Mock verifiers for testing; real crypto for production.

### CLI
- Subcommands: `clap` derive macros with `#[derive(Subcommand)]` enum.
- Each command: separate file in `cli/src/commands/<name>.rs`.
- Functions: `pub async fn run(api_url: String, ...) -> anyhow::Result<()>`.
- Amounts: use `kas_to_sompi()` for string-to-integer conversion.

### Testing
- Unit tests: `#[cfg(test)] mod tests` inline in source file.
- Integration tests: `tests/` directory in crate root.
- Contract tests: verify all spending paths + negative cases.
- Template hash tests: verify determinism and 20-byte length.
- Edge case tests: atomic operations, validation, fee calculation.

### Code clarity
- Comments explain WHY not WHAT.
- Delete dead code. No magic numbers.
- Naming: reveal intent at call site. Booleans: `is_*`, `has_*`, `can_*`.

### Concurrency
- Use `tokio::spawn` for background tasks (reconciliation loop, listener).
- Pass `Pool<Sqlite>` by clone (it's `Arc` internally).
- No blocking calls in async context.
- Use atomic queries for lifecycle transitions.

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
| Status enums | String-serializable with `as_str()`/`parse_status()` | `EscrowStatus` |
| API responses | `#[derive(Serialize)]` struct | `EscrowListResponse` |
| API requests | `#[derive(Deserialize)]` struct | `CreateEscrowRequest` |

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

### Atomic operations
**Pattern**: Single query with `WHERE status = 'active'`  **Example**: `indexer/src/db/queries.rs`
**Rule**: Lifecycle transitions must be atomic to prevent race conditions.

### Authentication
**Pattern**: Header extraction + trait-based verification  **Example**: `indexer/src/auth.rs`
**Rule**: Extract `AuthContext::from_headers()`, verify with `SignatureVerifier` trait.

### Contract source
**Pattern**: `pragma silverscript ^0.1.0` + constructor params as `byte[32]`/`int`  **Example**: `contracts/src/daglock.sil`
**Rule**: Constructor uses `byte[32]` (not `pubkey`) to work around compiler strict typing. Cast to `pubkey()` inside function bodies.

### Message encryption
**Pattern**: `AES-256-GCM` via `indexer/src/crypto.rs`  **Example**: `indexer/src/api/messages.rs`, `indexer/src/crypto.rs`
**Rule**: Messages encrypted on write, decrypted on read. Key from `DAGLOCK_MESSAGE_KEY` env var (64 hex chars). Dev fallback is deterministic blake2b hash.

### Jury system
**Pattern**: Score-gated registration (10+ trades, 3.0+ score) + randomized selection  **Example**: `indexer/src/api/jury.rs`
**Rule**: Top N*2 by reliability score -> random N from pool. Threshold varies by escrow value: 2/3 (<10K KAS), 3/5 (10K-100K), 5/9 (>100K). 72h timeout defaults to seller_wins.

### Vouch scoring (EigenTrust-lite)
**Pattern**: Weighted average of voucher reputations  **Example**: `indexer/src/db/queries.rs` `calculate_vouch_score()`
**Rule**: Each vouch contributes `voucher_score / 5.0` weight. Vouchers with 0 trades get score=1.0, weight=0.2. Vouches expire after 6 months.

### Route definition
**Pattern**: `axum 0.7` uses `:id` not `{id}` for path params  **Example**: `indexer/src/api/mod.rs`
**Rule**: Axum 0.7 uses `:id` syntax. Axum 0.8+ uses `{id}`. This project pins axum 0.7.9.

### Message format for auth
**Pattern**: Auth messages are `{action}:{escrow_id}`  **Example**: `indexer/src/auth.rs`
**Rule**: settle:`id`, refund:`id`, dispute:`id`, cancel:`id`, evidence:`id`, vote:`id`, vouch:`addr`, messages:`id`

### Error handling
**Pattern**: Internal errors use generic messages, not e.to_string()  **Example**: `indexer/src/api/escrows.rs`
**Rule**: All `internal_error` responses use static strings. `e.to_string()` only used in auth/verification contexts where the caller needs to know why a sig was rejected.

### UX: User-facing validation
**Pattern**: Inline validation with trim(), onBlur feedback, and confirmation dialogs  **Example**: `web/src/ui.tsx` `ValidatedInput`, `ConfirmDialog`
**Rule**: All address inputs must be trimmed, validated on blur with green/red feedback. Destructive actions (cancel, refund, dispute) require a confirmation dialog.

### Offer expiry
**Pattern**: `expires_at` timestamp on offers, auto-expired by reconciliation loop  **Example**: `indexer/src/db/queries.rs` `reconcile_expired_offers()`
**Rule**: Offer creation form includes an expiry dropdown (24h/3d/7d/30d). The background listener marks expired offers automatically.

### Dispute mode
**Pattern**: `dispute_mode` field on escrow (standard/mediator/jury)  **Example**: `indexer/src/types.rs`, create escrow form
**Rule**: When creating an escrow, the creator selects how disputes are resolved. `standard` = timeout refund only, `mediator` = specific mediator address, `jury` = community vote.
