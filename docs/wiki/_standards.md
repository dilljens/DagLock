# Rules

- **Rule #1: Never `.unwrap()` outside `#[cfg(test)]`**
  - Panics crash the indexer/CLI in production

Check: `grep -rn '\.unwrap()' indexer/src/ cli/src/` should only hit test code
- **Rule #2: Never hardcode addresses/keys in covenant source**
  - All params must come from constructor — covenants are parameterized

Check: `daglock.sil` and `daglock_krc20.sil` contain no literal addresses
- **Rule #3: Never skip fee validation in release/swap paths**
  - Treasury output must be checked — underpaying fees breaks protocol economics

Check: Every `release`/`swap` entrypoint checks `outputs[1].value == feeAmount`
- **Rule #4: Never expose private keys in bot/CLI/WASM**
  - Only unsigned tx assembled — signing happens in KasWare/kaspawallet

Check: No `signTransaction` calls outside wallet integration
- **Rule #5: Never change the fee denominator (200) without updating all paths**
  - Fee is hardcoded in compiled bytecode — inconsistent values = broken covenant

Check: `inputValue / 200` appears in all 3 entrypoints of both `.sil` files
- **Rule #6: Never use non-atomic updates for lifecycle transitions**
  - Race conditions can settle/refund the same escrow twice

Check: Use `settle_escrow_atomic()` or `refund_escrow_atomic()`
- **Rule #7: Never skip address validation on create**
  - Invalid addresses stored in DB cause failed settlements

Check: Validate with `validate_kaspa_address()`

# Commands

- `Test: shared`
  - Run: `cargo test -p daglock-shared` | Notes: 20 tests (constants + validation)
- `Test: contracts`
  - Run: `cargo test -p daglock-contracts` | Notes: 5 test files — TxScriptEngine execution tests
- `Test: indexer`
  - Run: `cargo test -p daglock-indexer` | Notes: 3 test files — unit + lifecycle + edge cases
- `Test: cli`
  - Run: `cargo test -p daglock-cli` | Notes: Config + arg parsing
- `Test: wasm-sdk`
  - Run: `cargo test -p daglock-wasm-sdk` | Notes: Native (not wasm) compilation tests
- `Test: web`
  - Run: `cd web && npm test` | Notes: 36 tests across 9 files (Vitest + RTL)
- `Test: bot`
  - Run: `cd bot && npm test` | Notes: API client + command handlers
- `Test: simulation`
  - Run: `python3 scripts/simulation.py --trades 30 --bots 3` | Notes: Mass trade generation + reputation testing
- `reputation-submitter.py`
  - Script to backfill settled escrows into the on-chain reputation covenant. Reads from indexer API, produces unsigned receipts.

# Decisions

- ****U1**: CLI create uses dummy keys**
  - Status: ✅ Fixed | Domain: cli
- ****U3**: No wallet signing in CLI**
  - Status: ✅ Fixed | Domain: cli
- ****Q1**: `.unwrap()` in production**
  - Status: ✅ Fixed | Domain: cli
- ****Q2/Q3**: Magic number `200` in `tx.rs`**
  - Status: ✅ Fixed | Domain: cli
- ****Q4**: `trade_hash` not validated**
  - Status: ✅ Fixed | Domain: cli
- ****U2** (web): WASM SDK missing `assemble_unsigned_tx` export**
  - Status: ❌ Open | Domain: wasm-sdk
- ****Q2/Q3**: Fee denominator not exposed**
  - Status: ✅ Fixed | Domain: wasm-sdk
- ****Q4**: `validate_trade_hash` not exported**
  - Status: ✅ Fixed | Domain: wasm-sdk
- ****U2**: Web CreateEscrowForm generates fake `lock_tx_id`**
  - Status: ❌ Open | Domain: web
- ****U7**: No web onboarding for first-time users**
  - Status: ✅ Fixed | Domain: web
- ****Q7**: Web API no request timeout**
  - Status: ✅ Fixed | Domain: web
- ****Q1**: `.expect()` on UUID in code**
  - Status: ✅ Fixed | Domain: web
- ****Q2/Q3**: Magic number `200` in fee calc**
  - Status: ✅ Fixed | Domain: web
- ****S6**: Bot stores addresses in plaintext /tmp**
  - Status: ✅ Fixed | Domain: bot
- ****U4**: Bot `/create` redirects to web**
  - Status: ✅ Fixed | Domain: bot
- ****Q8**: Bot API no retry/backoff**
  - Status: ✅ Fixed | Domain: bot
- ****A6**: Bot is Node.js while rest is Rust**
  - Status: ❌ Open | Domain: bot
- ****S2**: KRC-20 fee validation only boolean — feePaid loop checks if *any* output pays treasury, not the correct 0.5%**
  - Status: ✅ Fixed | Domain: contracts
- ****S3**: KRC-20 KCC-20 output ownership validation**
  - Status: ⏭️ Closed | Domain: contracts
- ****Phase 0**: Shared crate with FEE_DENOMINATOR**
  - Status: ✅ Done | Domain: contracts
- ****Q2/Q3**: Magic number `200` hardcoded in 3 covenant files — no single source of truth**
  - Status: Consistency risk; `FEE_DENOMINATOR` constant now in `shared/src/constants.rs` | Domain: contracts
- ****Q4**: `trade_hash` handling: KAS/Arbiter use `byte[32]`, KRC-20 uses `byte[32]` — consistent but API validation missing**
  - Status: `daglock_shared::validate_trade_hash()` now validates 64-hex-char input on escrow creation | Domain: contracts
- **Phase 1: On-chain Reputation Covenant**
  - Completed June 18, 2026. d6f4eb9: daglock_reputation.sil + tests. 0f1c62c: template hash in indexer config + backfill script.
