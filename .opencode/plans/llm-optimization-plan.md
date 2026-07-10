# LLM Optimization Plan

> **Goal:** Make DagLock maximally effective for AI agents to understand, test, debug, and improve.
>
> **Rationale:** 99.9% of future attention to this codebase will be LLM attention (Karpathy). Optimizing for LLMs now compounds every future interaction.

---

## Current State Assessment

| Area | Status | Score |
|------|--------|-------|
| **Codebase knowledge graph** | ✅ `codebase-memory-mcp` indexed, 5.2K nodes, 12.5K edges | 9/10 |
| **Agent instructions** | ✅ `AGENTS.md` with architecture, deployment, VPS info | 7/10 |
| **Wiki (cold memory)** | ✅ `docs/wiki/` with glossary, index, standards | 7/10 |
| **Structured logging** | ⚠️ Request tracing (Q5) done, but no structured JSON | 4/10 |
| **Error types** | ⚠️ `ApiErrorCode` enum exists, but lacks context fields | 5/10 |
| **Testing** | ❌ Example-based tests only, no property-based testing | 3/10 |
| **LLM entrypoint** | ❌ No `llm.txt` or condensed single-page summary | 0/10 |
| **Test workflows** | ❌ No explicit agent test instructions | 2/10 |
| **Debug tooling** | ❌ No structured bug reports, no MCP debug tools | 1/10 |

---

## Track A: LLM Entry Point (Quick Wins)

### A1: Create `llm.txt`

A single file at the repo root that LLMs can ingest in one shot. Based on the ReadMe.LLM pattern:

```
# DagLock — Trustless Escrow on Kaspa

## Architecture
Monorepo: contracts (SilverScript) → indexer (Rust axum) → web (React) → bot (Node.js/grammY)

## Key Files
- contracts/src/daglock.sil — KAS escrow covenant (6 entrypoints)
- contracts/src/daglock_krc20.sil — KRC-20 token escrow (4 entrypoints)
- indexer/src/main.rs — Entry point, verifier wiring
- indexer/src/api/offers.rs — Offer board CRUD
- indexer/src/verification.rs — UTXO verification (MockVerifier / WrpcVerifier / RestVerifier)
- bot/src/index.js — Telegram bot command handlers

## Build & Test
cargo test --workspace  # 293 Rust tests
cd web && npm test       # 44 Web tests
cd bot && npm test       # 39 Bot tests

## Key Rules
1. Never .unwrap() outside #[cfg(test)]
2. Never hardcode addresses/keys in covenant source
3. Never skip fee validation in release/swap paths
4. Never change fee denominator (200) without updating all paths
5. Never expose private keys in bot/CLI/WASM

## Database
SQLite at /opt/daglock-indexer/daglock.db (VPS)
Schema in indexer/src/db/schema.rs
Migrations in indexer/src/db/migrations/

## VPS
ssh ubuntu@40.160.241.74 (pw: raspi9000)
systemd: daglock-indexer, daglock-bot
```

**Effort:** 15 min

---

### A2: Enhance `AGENTS.md` with Testing Workflows

Add a section to `AGENTS.md` that instructs agents on what to do after changes:

```markdown
## Agent Workflows

### After any code change
1. Run `cargo test --workspace` if Rust changed
2. Run `cd web && npm test` if web changed
3. Run `cd bot && npm test` if bot changed
4. Fix any test failures before proposing the change

### When debugging a failure
1. Read the full error output (not just the summary)
2. Check `sudo journalctl -u daglock-indexer -n 50` for server-side errors
3. Check `sudo journalctl -u daglock-bot -n 50` for bot-side errors
4. Trace the error path: what function emitted it → what called it → what inputs

### Before editing a covenant
1. Read the full `.sil` file
2. Check all entrypoints and their requires
3. Run covenant tests: `cargo test -p daglock-contracts`
4. Verify all existing tests still pass
```

**Effort:** 20 min

---

## Track B: LLM-Friendly Testing

### B1: Property-Based Tests for Covenants (Proptest)

**Why:** Property-based testing (PBT) is 23-37% more effective for LLMs than example-based TDD. Properties (invariants) are easier for LLMs to define correctly than specific input-output pairs.

**Current state:** 104 example-based execution tests. Great coverage of specific paths, but no invariant testing.

**Properties to add for `daglock.sil`:**

```rust
// Property: After any settlement, the sum of outputs always equals input - fee
proptest! {
    #[test]
    fn settle_preserves_value_invariant(amount in 200_000u64..10_000_000_000u64) {
        let result = test_release(true, true, false, amount);
        prop_assert!(result.is_ok());
        let tx = result.unwrap();
        let input_value = amount;
        let fee = amount / 200;
        let output_sum: u64 = tx.outputs.iter().map(|o| o.value).sum();
        prop_assert_eq!(output_sum, input_value);
    }
}
```

**Properties to test across all covenants:**

| Property | Covenant | What it guards |
|----------|----------|---------------|
| `output_sum == input - fee` | All | Value conservation |
| `fee == input / 200` | All | Fee correctness |
| `MIN_OUT` enforced on all outputs | All | Dust protection |
| `seller_output >= amount - fee` | daglock.sil | Seller gets paid |
| `buyer_output >= amount` (refund) | daglock.sil | Buyer can reclaim |
| `remaining == input - installment` | subscription.sil | Installment math |
| `milestone_sum <= total` | milestone.sil | No overallocation |
| `no_unauthorized_spending` | All | Security property |
| Output scripts match expected parties | All | Destination safety |

**Files to create:**
- `contracts/tests/proptest_daglock.rs` — Property tests for base covenant
- `contracts/tests/proptest_subscription.rs` — Property tests for subscription
- `contracts/tests/proptest_milestone.rs` — Property tests for milestone

**Dependency:** Add `proptest = "1"` to contracts/Cargo.toml

**Effort:** 3-5 days

---

### B2: Snapshot Testing with `insta`

**Why:** Snapshot tests produce deterministic golden files that LLMs can inspect. Regenerating snapshots with `--update-snapshots` lets the LLM review the diff.

**Current state:** No snapshot tests.

**Where to add:**

| Test | What it captures |
|------|-----------------|
| Compilation output bytes | Ensures covenant bytecode doesn't change unexpectedly |
| Offer list JSON response | API contract stability |
| Escrow lifecycle state transitions | State machine correctness |
| Error response formats | API error contract |

**Files to modify:**
- Add `insta` to indexer/Cargo.toml and contracts/Cargo.toml
- Add snapshot assertions to existing test functions
- Store snapshots in `contracts/tests/snapshots/` and `indexer/tests/snapshots/`

**Effort:** 2 days

---

### B3: Structured Test Files for LLMs

**Problem:** Current test files mix concerns and lack clear structure. LLMs spend extra tokens navigating them.

**Pattern to follow:**
```rust
//! Module-level doc: what this test file covers, what patterns it uses
//!
//! # Properties tested
//! - Value conservation: sum of outputs == input - fee
//! - Fee correctness: fee == input / 200
//!
//! # Test categories
//! 1. Happy path — valid signatures, correct amounts
//! 2. Edge cases — boundary values, zero amounts
//! 3. Error cases — wrong signatures, expired timeouts
//! 4. Security properties — unauthorized spending, double-spend

// ── Helpers ─────────────────────────────────────
// (shared setup code)

// ── Happy path tests ────────────────────────────
// Tests with all conditions met

// ── Edge case tests ─────────────────────────────
// Boundary conditions, corner cases

// ── Error case tests ────────────────────────────
// Expected failure modes

// ── Property-based tests ───────────────────────
// Invariant checks with random inputs
```

**Effort:** 1 day (can do incrementally as tests are modified)

---

## Track C: LLM-Friendly Debugging

### C1: Structured JSON Logging

**Current state:** Free-text `tracing::info!()` and `tracing::warn!()` calls with no structured fields.

**Target state:** All log events include structured fields that LLMs can parse:

```rust
// Before:
tracing::info!("UTXO found for escrow {} — amount: {}", escrow.id, amount);

// After:
tracing::info!(
    event = "utxo_verified",
    escrow_id = %escrow.id,
    utxo_tx_id = %tx_id_hex,
    utxo_index = output_index,
    utxo_amount = amount,
    verifier = "WrpcVerifier",
);
```

**Key events to structure:**

| Event | Fields |
|-------|--------|
| `escrow_created` | escrow_id, buyer, seller, amount, asset, template_hash |
| `escrow_settled` | escrow_id, seller_output, fee, method |
| `offer_created` | offer_id, creator, side, base_asset, amount |
| `utxo_verified` | escrow_id, tx_id, output_index, amount, verifier |
| `auth_verification` | address, action, result, escrow_id |
| `verification_error` | escrow_id, error_type, details |
| `rate_limited` | ip, tier, count, window |

**Add `tracing-stackdriver` or custom JSON layer** to output logs as NDJSON (newline-delimited JSON).

**Effort:** 2-3 days

---

### C2: Enriched Error Types

**Current state:** `ApiErrorCode` enum exists but error variants carry no context:

```rust
pub enum ApiErrorCode {
    InvalidState,      // ← what state? what was expected?
    RateLimited,       // ← how long to wait? what limit?
    VerificationFailed, // ← what was being verified? why?
}
```

**Target state:** Every error variant carries full context:

```rust
pub enum ApiErrorCode {
    InvalidState {
        expected: EscrowStatus,
        actual: EscrowStatus,
        escrow_id: String,
    },
    RateLimited {
        max_per_minute: u32,
        retry_after_seconds: u64,
        tier: ApiTier,
    },
    VerificationFailed {
        utxo_tx_id: String,
        utxo_index: u32,
        reason: String,
        escrow_id: String,
    },
}
```

**This helps LLMs because:**
- The error message itself contains ALL context needed for diagnosis
- No need to trace back to find what state the escrow was in
- LLMs can immediately suggest the correct fix

**Effort:** 2-3 days

---

### C3: Structured Bug Report Template

```markdown
---
name: Bug Report (LLM-Optimized)
about: Create a report with structured data for AI analysis
---

## Environment
- Network: [testnet-10 / testnet-11 / mainnet]
- Component: [indexer / bot / web / covenant]
- Version: [commit hash]
- Trace ID: [if available]

## Reproduction
1. Step-by-step to reproduce
2. Include exact inputs (amounts, addresses, signatures)
3. Expected output
4. Actual output (include full error JSON)

## Diagnostic Data
```json
{
  "request_id": "...",
  "error_code": "...",
  "escrow_id": "...",
  "context": { ... }
}
```

## Relevant Code
- File(s): [file paths with line numbers]
- Entrypoint: [function name]
```

**Effort:** 30 min

---

## Track D: MCP & Tooling

### D1: DagLock MCP Server (Future)

A custom MCP server exposing DagLock-specific operations:

| Tool | Purpose |
|------|---------|
| `query_escrow(id)` | Get escrow details without reading API code |
| `list_offers(filters)` | Search offers with structured results |
| `compile_covenant(params)` | Quick covenant compilation for verification |
| `check_utxo(tx_id, index)` | Verify UTXO existence on-chain |
| `get_reputation(address)` | Check address reputation |

**This would let LLMs interact with DagLock directly through MCP tools instead of crafting API calls.**

**Effort:** 3-5 days (deferred)

---

### D2: Playwright MCP for E2E Testing

**Problem:** E2E tests (40 Playwright tests) exist but aren't connected to MCP. LLMs can't run them or interpret results.

**Fix:** Install Playwright MCP server, enabling agents to:
- Run specific tests by name
- View test results and screenshots
- Debug failing tests by inspecting DOM

**Effort:** 1-2 days

---

## Priority & Effort Summary

| Priority | Item | Effort | Impact for LLMs |
|----------|------|--------|-----------------|
| **P0** | `llm.txt` + AGENTS.md workflows | 35 min | High — first thing every LLM reads |
| **P1** | Enriched error types (context fields) | 2-3 days | High — LLMs need context in errors |
| **P1** | Property-based tests for covenants | 3-5 days | High — PBT is 23-37% more effective than TDD for LLMs |
| **P2** | Structured JSON logging | 2-3 days | Medium — helps LLMs parse logs |
| **P2** | Snapshot testing (insta) | 2 days | Medium — golden files are LLM-friendly |
| **P2** | Bug report template | 30 min | Medium — structured data = better fixes |
| **P3** | Playwright MCP | 1-2 days | Low — nice to have |
| **P3** | DagLock MCP server | 3-5 days | Low — future investment |

---

## What's Already Done

DagLock already has several LLM-friendly features that many projects lack:

| Feature | Status |
|---------|--------|
| `codebase-memory-mcp` knowledge graph | ✅ Indexed, 5.2K nodes |
| `docs/wiki/` cold memory | ✅ Glossary, index, standards |
| `AGENTS.md` with VPS/architecture | ✅ Present |
| Request tracing (trace_id) | ✅ Q5 audit item |
| ApiErrorCode enum | ✅ U5 audit item |
| Comprehensive test suite | ✅ 376 tests |
| Structured test file names | ✅ By domain (`escrows.rs`, `offers.rs`) |
