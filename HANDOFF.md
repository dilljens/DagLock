# DagLock Handoff

**Built:** 2026-06-10 — Two intensive planning/execution sessions across ~30 files.
**State:** `cargo build --workspace` ✅ 0 errors, 0 warnings  |  `cargo test --workspace` ✅ **154/154 tests pass**

---

## Table of Contents

1. [What Changed](#what-changed)
2. [Stuck-Fund Safety Analysis](#stuck-fund-safety-analysis)
3. [Architecture Decisions Made](#architecture-decisions-made)
4. [Integration Surface](#integration-surface)
5. [Remaining Work](#remaining-work)
6. [Running the Project](#running-the-project)

---

## What Changed

### Session 1 — Security + Vaults + WASM + Docs (20 files)

**Security fixes (must-have before mainnet):**

| Fix | File | What |
|-----|------|------|
| **Dispute signature verification** | `indexer/src/api/escrows.rs` | Was completely missing — anyone could dispute with a spoofed `X-Daglock-Address` header. Now verifies sig + replay-protected message. |
| **Cancel replay protection** | `indexer/src/auth.rs`, `indexer/src/api/escrows.rs` | Cancel used bare `verify_signature()` without nonce. Added `verify_cancel_authorization()` following settle/refund pattern with `verify_nonce()`. |
| **Evidence replay protection** | `indexer/src/api/evidence.rs` | Both `submit_evidence()` and `log_dispute_outcome()` now use `parse_message()` + `verify_nonce()`. |
| **Rate limiter mutex** | `indexer/src/ratelimit.rs` | `lock().unwrap()` → `lock().unwrap_or_else(\|e\| e.into_inner())` to recover from poisoned mutex instead of crashing the server. |
| **Duplicate migration** | `indexer/src/db/schema.rs` | `ensure_price_type_column()` was a duplicate of `ensure_escrow_price_type()`. Removed. |

**Vault infrastructure:**

| Feature | Files | What |
|---------|-------|------|
| **Vault softlock compile** | `indexer/src/api/compile.rs` | New compile endpoint for `daglock_vault_softlock` (password-recoverable vault) |
| **Vault multisig compile** | `indexer/src/api/compile.rs` | New compile endpoint for `daglock_vault_multisig` (N-of-M multisig vault) |
| **Treasury key fix** | `indexer/src/api/compile.rs` | `compile_vault_template()` now uses canonical treasury key instead of hardcoded zeros |
| **Vault type validation** | `indexer/src/api/vaults.rs` | Vault creation validates the vault type has a matching covenant template configured |
| **Password-withdraw endpoint** | `indexer/src/api/vaults.rs`, `indexer/src/api/mod.rs` | `POST /v1/vaults/:id/password-withdraw` — SHA-256 password verification for beneficiary vaults |
| **Config args** | `indexer/src/config.rs`, `indexer/src/api/mod.rs`, `indexer/src/main.rs` | Added `--daglock-vault-softlock-template` and `--daglock-vault-multisig-template` CLI args |
| **Auto-sweep broadcaster** | `indexer/src/listener.rs`, `indexer/src/main.rs`, `indexer/src/config.rs` | `spawn_vault_sweeper()` — background loop, opt-in via `--auto-sweep-vaults`. Constructs sweep tx using daglock_contracts. |
| **Sweep DB columns** | `indexer/src/types.rs`, `indexer/src/db/queries.rs`, `indexer/src/db/schema.rs` | `owner_pubkey_hex` and `sweep_tx_id` fields on vaults for sweep idempotency |
| **Vault transfer doc** | `indexer/src/api/vaults.rs` | Doc comment clarifying transfer is DB-only — on-chain covenant is immutable |

**WASM SDK:**

| Function | Purpose |
|----------|---------|
| `compile_vault()` | Compile `daglock_vault` covenant |
| `compile_vault_softlock()` | Compile password-recoverable vault |
| `compile_vault_multisig()` | Compile multisig vault |
| `compile_arbiter()` | Compile arbiter (mediator) covenant |

**Docs:**

| Item | Location |
|------|----------|
| **Fee model comments** | All 6 `.sil` files — preamble explains who pays the fee in each scenario |
| **"Who Pays the Fee?" table** | `docs/wiki/features/contracts.md` — 7 scenarios with payer + mechanics |
| **Auto-verdict execution design doc** | `docs/wiki/plans/auto-verdict-execution.md` |
| **7 HTML covenant flowcharts** | `docs/flowcharts/` — Mermaid diagrams for all 6 covenants + dispute state machine |

---

### Session 2 — Stuck-Fund Fixes (5 files)

**The arbiter stuck fund problem:**
If a mediator disappears (quits, goes offline) and the seller won't co-sign `release()`, the buyer's funds are stuck **forever** because `refundAfterTimeout` requires the mediator's signature.

**The multisig vault stuck fund problem:**
All configured keys must sign (N-of-N). If any one key (key2 or key3) is lost, funds are permanently frozen — even if the other signers still have their keys.

**Fixes applied:**

| Covenant | New Entrypoint | Condition | Purpose |
|----------|---------------|-----------|---------|
| `daglock_arbiter.sil` | `emergencyRefund(buyerSig)` | `tx.time >= timeout + 2_592_000` (30 day grace) | Buyer reclaims solo if mediator disappeared |
| `daglock_vault_multisig.sil` | `sweep(key1Sig)` | `tx.time >= timeout` | key1 sweeps alone if key2/key3 lost |

**Both changes include:** covenant source, lib.rs entrypoint constants, execution tests.

---

## Stuck-Fund Safety Analysis

Every covenant now has an escape path. Here's the full matrix:

| Covenant | Paths | Stuck Scenario | Escape | Severity Before |
|----------|-------|---------------|--------|-----------------|
| `daglock.sil` (KAS escrow) | 3 | Buyer loses key | `refund(buyerSig)` — buyer alone after timeout | ✅ Safe |
| `daglock_krc20.sil` (token escrow) | 3 | Buyer loses key | `refund(buyerSig)` — buyer alone after timeout | ✅ Safe |
| `daglock_arbiter.sil` | 6 | Mediator disappears + seller won't release | `emergencyRefund(buyerSig)` — buyer alone at `timeout + 30d` | 🔴 **FIXED** |
| `daglock_vault.sil` (basic) | 2 | Owner loses key | `sweep()` — anyone can sweep to owner's P2PK after timeout | ✅ Safe |
| `daglock_vault_softlock.sil` | 2 | Both lose keys | Two independent paths (password AND timeout) cover each other | ✅ Safe |
| `daglock_vault_multisig.sil` | 2 | key2/key3 lost | `sweep(key1Sig)` — key1 sweeps alone after timeout | 🔴 **FIXED** |

**Tradeoff acknowledged:** `sweep(key1Sig)` gives key1 unilateral access after timeout. If key1 is compromised, an attacker can wait for the timeout and drain the vault. The timeout window is the protection. For full security, use `withdraw()` (N-of-N). For recovery, use `sweep()`.

---

## Architecture Decisions Made

### Fee Model
- **0.5% flat fee** deducted from output[0] (the settlement output)
- The party receiving output[0] economically pays
- Both parties consent by signing SIGHASH_ALL after reviewing outputs
- No split fee — the market adjusts price to account for it
- Vault fee is lower (0.1%) — self-custody infrastructure vs trade facilitation

### Replay Protection
- Message format: `{action}:{escrow_id}:{timestamp}:{nonce_hex}`
- Nonce is 20-byte BLAKE2b-160 hash (40 hex chars)
- Timestamp window: ±5 minutes (`MAX_CLOCK_DRIFT_SECONDS = 300`)
- V1 format (no nonce) is backward-compatible but skips replay check
- Nonces stored in `auth_nonces` table

### Covenant Design Patterns
- **Constructor uses `byte[32]`** (not `pubkey`) to work around SilverScript compiler strict typing
- **No `||` or `&&` with `checkSig`** — SilverScript doesn't support boolean operators on signature verification results. Use nested `if` blocks or separate entrypoints instead.
- **Fee is hardcoded** as `inputValue / 200` (or `inputValue / 1000` for vaults) — changing requires updating ALL entrypoints
- **Template hash is BLAKE2b-160** (20 bytes), not SHA-256

### Emergency Paths
- Arbiter: 30-day fixed grace period (`timeout + 2_592_000` seconds)
- Vault sweep: no signature needed (public), funds hardcoded to owner's P2PK
- Multisig sweep: key1 signature only

---

## Integration Surface

### REST API Endpoints Added/Changed

| Method | Path | Auth | What |
|--------|------|------|------|
| POST | `/v1/vaults/:id/password-withdraw` | No (password-based) | Withdraw from softlock vault with password |
| POST | `/v1/escrows/:id/log-dispute-outcome` | X-Daglock-* headers | Renamed from `resolve-dispute` — informational only |
| POST | `/v1/compile` | No | New templates: `daglock_vault_softlock`, `daglock_vault_multisig` |

### CLI Flags Added

| Flag | Type | Default | Purpose |
|------|------|---------|---------|
| `--daglock-vault-softlock-template` | `Option<String>` | None | Template hash for softlock vault UTXO detection |
| `--daglock-vault-multisig-template` | `Option<String>` | None | Template hash for multisig vault UTXO detection |
| `--auto-sweep-vaults` | `bool` | false | Enable vault auto-sweep background loop |

### Web Frontend Changes

| File | What |
|------|------|
| `web/src/api.ts` | `resolveDispute` renamed to `logDisputeOutcome` |
| `web/src/components/jury.tsx` | Updated API call name + message format |
| `web/src/components/escrows.tsx` | Fee breakdown preview on create form; detail panel shows "Paid by recipient" |
| `web/src/components/offers.tsx` | Fee tooltip shows "paid by recipient on settlement" |

### Bot Changes

| File | What |
|------|------|
| `bot/src/index.js` | `/status` output shows "paid by recipient on settlement" |

---

## Remaining Work (For Future Sessions)

### High Priority

#### F1 — Listener UTXO Scanning
The wRPC listener runs `reconcile_expired_escrows()` every 10s but **never scans new blocks** for DagLock template hashes. Escrows stay `pending_confirmation` forever.
- Subscribe to `BlockAdded` notifications
- Scan outputs, compute BLAKE2b-160 hash, match against configured template hashes
- On match: update escrow `pending_confirmation` → `active`
- **Assets already built:** `check_template_match()` exists in listener.rs. Template hashes are already configurable.

#### F2 — Web Onboarding Modal (Audit item U7)
First-visit welcome modal: connect KasWare, address setup, testnet faucet link. Dismiss with localStorage.

#### F3 — Atomic Swap Wizard (Feature #8)
Replace basic SwapPage with 4-step guided wizard: generate secret → create hash-locked escrow → fund → settle.

### Medium Priority

#### F4 — OpenAPI Spec Regeneration
Current spec is hand-written (5.5KB), stale. Add `utoipa` annotations to all handlers for auto-generated docs.

#### F5 — Volume-Based Fee Tiers
Off-chain rebate system: `fee_tiers` table, monthly volume calculation per app_id, webhook event on threshold exceeded. No covenant changes.

### Low Priority

#### F6 — Dead Code Cleanup
24 `#[allow(dead_code)]` annotations across the indexer. Mostly harmless — API response types that are constructed via serde. ~6 truly dead functions (listed in session 2 plan).

---

## Running the Project

```bash
# Build everything
cargo build --workspace

# Run all tests (154 total)
cargo test --workspace

# Run the indexer with mock auth (dev mode)
cargo run -p daglock-indexer -- --mock-auth --network testnet-11 --database-url sqlite::memory:

# Run with vault auto-sweep
cargo run -p daglock-indexer -- --mock-auth --network testnet-11 \
  --database-url sqlite:daglock.db \
  --auto-sweep-vaults

# Print current template hashes
cargo test -p daglock-contracts -- --nocapture print_template_hashes

# View covenant flowcharts (open in browser)
open docs/flowcharts/index.html
```

### Test distribution

| Test group | Count | Command |
|-----------|-------|---------|
| CLI unit | 6 | `cargo test -p daglock-cli` |
| Contracts lib | 15 | `cargo test -p daglock-contracts --lib` |
| KAS execution | 7 | Same (integration test) |
| KRC-20 execution | 9 | Same |
| KRC-20 lib | 6 | Same |
| Arbiter execution | 17 | Same |
| Vault execution | 11 | Same |
| Indexer unit | 39 | `cargo test -p daglock-indexer --lib` |
| Integration API | 12 | `cargo test -p daglock-indexer --test api_tests` |
| Integration edge cases | 8 | `cargo test -p daglock-indexer --test edge_cases` |
| Shared validation | 20 | `cargo test -p daglock-shared` |
| WASM SDK | 5 | `cargo test -p daglock-wasm-sdk` |

---

## Key Files Reference

| Purpose | Path |
|---------|------|
| KAS escrow covenant | `contracts/src/daglock.sil` |
| KRC-20 token escrow | `contracts/src/daglock_krc20.sil` |
| Arbiter/mediator escrow (6 paths) | `contracts/src/daglock_arbiter.sil` |
| Basic vault (2 paths) | `contracts/src/daglock_vault.sil` |
| Password vault (2 paths) | `contracts/src/daglock_vault_softlock.sil` |
| Multisig vault (2 paths) | `contracts/src/daglock_vault_multisig.sil` |
| Covenant compilation API | `contracts/src/lib.rs` |
| Entrypoint constants | `contracts/src/lib.rs` (entrypoints module) |
| Arbiter execution tests | `contracts/tests/daglock_arbiter_tests.rs` |
| Vault + multisig execution tests | `contracts/tests/daglock_vault_tests.rs` |
| Auth + replay protection | `indexer/src/auth.rs` |
| Rate limiter | `indexer/src/ratelimit.rs` |
| REST API routes | `indexer/src/api/mod.rs` |
| Escrow handlers | `indexer/src/api/escrows.rs` |
| Evidence/dispute handlers | `indexer/src/api/evidence.rs` |
| Vault handlers | `indexer/src/api/vaults.rs` |
| Compile endpoints | `indexer/src/api/compile.rs` |
| App registration + API keys | `indexer/src/api/apps.rs` |
| DB schema + migrations | `indexer/src/db/schema.rs` |
| All DB queries | `indexer/src/db/queries.rs` |
| wRPC listener + vault sweeper | `indexer/src/listener.rs` |
| CLI config args | `indexer/src/config.rs` |
| Indexer entrypoint | `indexer/src/main.rs` |
| WASM SDK | `wasm-sdk/src/lib.rs` |
| Web UI entry | `web/src/App.tsx` |
| Web API client | `web/src/api.ts` |
| Bot entry | `bot/src/index.js` |
| Fee docs | `docs/wiki/features/contracts.md` |
| HTML flowcharts | `docs/flowcharts/*.html` |
| Auto-verdict design doc | `docs/wiki/plans/auto-verdict-execution.md` |
