# DagLock Security Model

> Threat analysis, vulnerability classes, and audit checklist for the DagLock SilverScript covenant.

---

## 1. Trust Model

DagLock is designed to be **trustless**: no party needs to trust any other party, the DagLock team, or an intermediary. Trust is placed only in:

1. **Kaspa consensus** — the BlockDAG's proof-of-work and finality guarantees
2. **SilverScript compiler** — correctness of `silverscript-lang` compiler output
3. **Kaspa script VM** — correct execution of opcodes (KIP-17/KIP-20)
4. **secp256k1 cryptography** — ECDSA signature security

**What DagLock does NOT require trust for:**
-  DagLock team (no admin keys, no upgrade mechanism)
-  Oracle providers (no external data feeds)
-  Relayers or validators (all logic is on-chain)
-  Counterparty (covenant enforces terms automatically)

---

## 2. Threat Model

### 2.1 Contract-Level Threats

| # | Threat | Impact | Likelihood | Mitigation |
|---|---|---|---|---|
| T1 | **Replay attack** — Spend the same UTXO twice | Double-spend of escrow funds | Low (Kaspa UTXO model naturally prevents) | Kaspa's UTXO set enforces single-spend. Redundant check in covenant is not needed. |
| T2 | **Output inflation** — Craft release tx with extra outputs draining value | Theft of deposited funds | Medium | Covenant checks `tx.outputs[0].value` and `tx.outputs[1].value` explicitly. Any extra outputs carry 0 or change. |
| T3 | **Fee manipulation** — Change `feeAmount` calculation | Undercut treasury fee | Low | `feeAmount = value / 200` is hardcoded in compiled bytecode. Cannot be altered. |
| T4 | **Premature refund** — Claim refund before `expirationBlock` | Seller loses funds without receiving assets | High | Covenant requires `tx.blockHeight >= expirationBlock`. Cannot be bypassed. |
| T5 | **Signature malleability** — Use different encoding of valid signature | Spam / indexer confusion | Low | SilverScript uses Kaspa's canonical signature encoding. |
| T6 | **Hash collision** — Find preimage for SHA-256 trade hash | Steal funds from atomic swap | Negligible | SHA-256 collision resistance is ~2^128. |
| T7 | **Script mass overflow** — Covenant bytecode exceeds block mass limit | Cannot spend escrow | Medium | Benchmark during Phase 1. If mass is too high, split into simpler covenants. |

### 2.2 Indexer-Level Threats

| # | Threat | Impact | Likelihood | Mitigation |
|---|---|---|---|---|
| T8 | **False positive detection** — Non-DagLock tx matches template hash | Incorrect escrow state in API | Low | Template hash is 20 bytes BLAKE2b — collision probability is negligible. |
| T9 | **Indexer desync** — wRPC disconnection causes missed blocks | Missing escrow events | Medium | Track DAA score in DB. On reconnect, replay from last processed score. |
| T10 | **Front-running** — Third party claims escrow before intended recipient | Loss of funds | See below | See Section 3. |

### 2.3 Infrastructure Threats

| # | Threat | Impact | Likelihood | Mitigation |
|---|---|---|---|---|
| T11 | **DNS hijack** — Attacker controls daglock.io | Phish user signatures | Low | Strict Content-Security-Policy; HSTS preload; don't host private keys. |
| T12 | **DB compromise** — Escrow metadata leaked | Privacy loss | Medium | DB contains only on-chain public data (addresses, amounts, status). No secrets. |
| T13 | **KasWare extension compromise** — Malicious wallet signs bad tx | User funds lost | Low | Out of DagLock's scope. Users must verify KasWare extension integrity. |

---

## 3. Front-Running Analysis

### 3.1 On the Release Path

When a counterparty signs a release transaction, they reveal the unsigned tx to the network. In theory, a miner could see the tx in the mempool and attempt to replace the recipient address with their own.

**Why this fails for DagLock:**
- The release transaction includes the **counterparty's signature** over the specific outputs. Changing the recipient address invalidates the signature.
- The covenant checks `tx.outputs[0].value` and `tx.outputs[1].scriptPubKey` — but **not** `tx.outputs[0].scriptPubKey` (the recipient). This is by design: the recipient need not sign, so only the covenant enforces the output.
- **Therefore:** the party constructing the release tx controls the recipient. The counterparty verifies and **only signs after verifying the recipient address matches the trade link**. The signed tx is broadcast immediately — no time for replacement.

### 3.2 On the Refund Path

Refund transactions can only be created by the depositor (buyer) and only after `expirationBlock`. No front-running opportunity — the refund is permissioned by signature.

### 3.3 On Deployment

The lock transaction has no spending conditions beyond standard P2SH. Anyone could theoretically send funds to the same P2SH address with the same parameters, creating a duplicate escrow.

**Mitigation:** The indexer detects duplicate UTXOs and marks them as separate escrow instances. The trade link references a specific `(tx_id, output_index)` pair, so there is no ambiguity about which UTXO is being traded.

---

## 4. Audit Checklist

Before mainnet launch, each condition below must be verified:

### 4.1 Contract

- [ ] **All spending paths tested** — release (mutual sigs), release (preimage), refund
- [ ] **Negative tests pass** — premature refund rejected, wrong sig rejected, fee mismatch rejected
- [ ] **Script mass measured** — release path mass < `MAX_SCRIPT_PUBLIC_KEY_MASS`
- [ ] **Template hash deterministic** — same source always produces same hash
- [ ] **No admin key** — verify no hidden `OP_CHECKSIG` for treasury key in spending paths
- [ ] **Fee calculation integer-safe** — `value / 200` cannot overflow in Kaspa script i64
- [ ] **Multi-sig edge cases** — if both buyer and seller pubkeys are identical, multisig still works

### 4.2 Indexer

- [ ] **Disconnect-reconnect test** — indexer resumes from correct DAA score
- [ ] **Empty block handling** — no crash on block with 0 transactions
- [ ] **Duplicate detection** — same escrow parameters in different UTXOs are separate escrows
- [ ] **Rate limiting** — API returns 429 under load
- [ ] **Migration tests** — SQLx up/down migrations work atomically

### 4.3 WASM SDK

- [ ] **Transaction serialization matches Kaspa consensus** — use `kaspawallet` to validate assembled txs
- [ ] **KasWare integration works** — `window.kasware.request({ method: 'signTransaction' })` succeeds
- [ ] **Trade link encryption** — link does not leak secret preimage
- [ ] **No private key exposure** — SDK never touches private keys

### 4.4 Web UI

- [ ] **XSS prevention** — all user-supplied data (addresses, amounts) sanitized
- [ ] **CSRF protection** — no state-changing endpoints that rely on cookies
- [ ] **Wallet address verification** — user confirms address before signing
- [ ] **Error display** — network errors, rejected transactions shown clearly

---

## 5. Bug Bounty Program (Post-Launch)

**Scope:** Compiled `daglock.sil` covenant on Kaspa mainnet
**Reward:** Up to $10,000 in KAS equivalent
**Classification:**

| Severity | Criteria | Reward |
|---|---|---|
| Critical | Direct theft of user funds | $10,000 |
| High | Permanent lock of user funds | $5,000 |
| Medium | Bypass fee mechanism | $1,000 |
| Low | Indexer data inconsistency | $250 |

**Exclusions:** KasWare wallet bugs, Kaspa core consensus bugs, phishing attacks.

---

## 6. Audit Findings — 2026-06-06 Pre-Mainnet Review

A comprehensive codebase audit was performed on June 6, 2026, covering contracts, indexer, CLI, WASM SDK, web UI, and Telegram bot. The following issues were identified and must be resolved before mainnet launch.

### 6.1 Critical / High Severity

| ID | Finding | Location | Status |
|----|---------|----------|--------|
| **S1** | **MockVerifier used in production** — `WrpcVerifier.verify_utxo_exists()` always returns `Ok(true)` even with real wRPC client. No actual on-chain UTXO verification occurs at settlement/refund. Attackers can settle/refund escrows that never existed on-chain. | `indexer/src/verification.rs`, `indexer/src/main.rs` | 🔴 Open |
| **S2** | **KRC-20 fee validation only boolean** — `feePaid` loop checks if *any* output pays treasury, not the *correct amount* (0.5%). Fee can be 1 sompi. Comment says "enforced off-chain" but off-chain checks are bypassable. | `contracts/src/daglock_krc20.sil:54-64` | 🔴 Open |
| **S3** | **KRC-20 doesn't verify KCC-20 output ownership** — `release()`/`swap()` don't verify the KCC-20 output transitions to the correct new owner (seller). Only checks fee paid. KCC-20's `checkSigs()` validates DagLock authorization but DagLock doesn't validate KCC-20 output state. | `contracts/src/daglock_krc20.sil` | 🔴 Open |

### 6.2 Medium Severity

| ID | Finding | Location | Status |
|----|---------|----------|--------|
| **S4** | **trade_hash not validated on escrow creation** — Could be wrong length/format. Atomic swap preimage verified off-chain in API *before* settling, but covenant also checks it. | `indexer/src/api/escrows.rs:182-197` | 🟠 Open |
| **S5** | **No replay protection on signed messages** — Signed messages (`settle:{id}`, `refund:{id}`) include escrow ID but no nonce/timestamp. Captured signature could be replayed if escrow recreated with same ID. | `indexer/src/auth.rs` | 🟠 Open |
| **S6** | **Bot stores addresses in plaintext /tmp** — `/tmp/daglock-users.json` world-readable on shared systems. No encryption at rest. | `bot/src/index.js:20-34` | 🟠 Open |
| **S7** | **Dockerfile runs as root** — No `USER` directive in final stage. Container runs as root. | `Dockerfile` | 🟠 Open |

### 6.3 Structural / Code Quality Issues (Not Direct Vulnerabilities)

| ID | Finding | Impact |
|----|---------|--------|
| **A1** | `EscrowVerifier` trait is synchronous but wRPC is async — real verification cannot work | Blocks S1 fix |
| **A2** | Migration `.ok()` silences schema failures — migrations 010-013 can fail silently | DB integrity risk |
| **A3** | `queries.rs` is 1843-line god module — escrow, reputation, jury, offers, vaults, messages, vouches, evidence, identity, mediator, receipts | Maintainability |
| **A4** | No full lifecycle integration test — create→lock→settle→receipt not tested end-to-end | Regression risk |
| **A5** | Handlers mix HTTP + business logic + DB — no service layer | Testability |
| **A6** | Bot is Node.js while rest is Rust — different dep chain, no shared types | Maintenance burden |
| **A7** | No OpenAPI spec — 28 endpoints documented only in code | Integration friction |
| **A8** | No template hash verification on create — accepts any template_hash | Could register fake covenants |
| **Q1** | `.unwrap()` in production code — violates `_standards.md` Rule #1 | Panic risk |
| **Q2/Q3** | Magic number 200 scattered in 5+ locations — no single source of truth | Consistency risk |
| **Q4** | `trade_hash` handling inconsistent — contracts use `byte[32]`, API uses optional string | Validation gaps |
| **Q5** | No structured request tracing — hard to debug production issues | Observability |
| **Q6** | Config validation gaps — `--wrpc-url` format, CORS origin not validated | Misconfiguration risk |
| **Q7** | Web API no request timeout — `fetch()` calls hang UI indefinitely | UX / DoS |
| **Q8** | Bot API no retry/backoff — single `fetch()` fails under load | Reliability |

### 6.4 Rules Violations (from `_standards.md`)

| Rule | Description | Status |
|------|-------------|--------|
| #1 | Never `.unwrap()` outside `#[cfg(test)]` | ❌ Violated |
| #2 | Never hardcode addresses/keys in covenant source | ✅ Compliant |
| #3 | Never skip fee validation in release/swap paths | ❌ Violated (KRC-20) |
| #4 | Never expose private keys in bot/CLI/WASM | ✅ Compliant |
| #5 | Never change fee denominator without updating all paths | ⚠️ Risk (200 in 5+ places) |
| #6 | Never use non-atomic updates for lifecycle transitions | ✅ Compliant |
| #7 | Never skip address validation on create | ✅ Compliant |

### 6.5 Remediation Plan

All findings are tracked in the **30-task fix plan** at `.pi/last-plan.md` and `docs/wiki/_index.md#audit-log`.

**Priority order for mainnet launch (June 30):**
1. **Phase 1 (Critical Security):** S1, S2, S3, S4, S5, S6, S7, A1, A2, A8, Q5
2. **Phase 2 (Usability):** U1, U2, U3, U4, U5, U6, U7
3. **Phase 3 (Structural):** A3, A4, A5, A7, Q2/Q3, Q4
4. **Phase 4 (Polish):** Q1, Q7, Q8

**Verification required before mainnet:**
- [ ] `cargo test --workspace` passes
- [ ] `cargo test -p daglock-contracts` — all covenant execution tests pass
- [ ] `cargo test -p daglock-indexer --test lifecycle_tests` — full lifecycle tests pass
- [ ] Manual testnet deployment with `--wrpc-url` — create escrow via web → KasWare sign → broadcast → settle → verify receipt
- [ ] Manual CLI test — `daglock-cli create` → `kaspawallet sign` → broadcast → settle via CLI
- [ ] Manual bot test — `/create` wizard → deep link → complete flow

---

## 7. Post-Launch Security Monitoring

After mainnet launch (June 30), the following monitoring must be active:

| Check | Frequency | Tool |
|-------|-----------|------|
| Indexer health endpoint (`/v1/health`) | Every 30s | Railway / custom |
| wRPC connection status | Every 10s | Indexer listener logs |
| Escrow settlement rate | Hourly | Dashboard / SQL |
| Error rate on `/v1/escrows/*/settle` | Real-time | Structured logs |
| Template hash match rate | Per block | Indexer listener |
| Bot command error rate | Real-time | Bot logs |

**Incident response:** If any settlement fails verification (S1 fix broken), immediately disable settlement endpoint and investigate. The `MockVerifier` fallback must be removed from production config.
