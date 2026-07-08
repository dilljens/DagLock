# Mainnet Readiness — Execution Plan

**Goal:** Get DagLock from current testnet state → mainnet-ready and deployed by June 30, 2026.

**Target date:** Toccata hard fork activation (~June 30)

---

## Requirements
- [ ] R1: All audit security items addressed (S1-S7)
- [ ] R2: All tests pass green (Rust, Web, Bot)
- [ ] R3: Mainnet indexer binary deployed on OVHcloud VPS
- [ ] R4: Indexer connected to a mainnet Kaspa wRPC endpoint
- [ ] R5: All 3 product surfaces work end-to-end (Web, CLI, Bot)
- [ ] R6: Announcement materials ready (demo video, Telegram post, tweet)
- [ ] R7: Monitoring in place (logs, error alerts)

---

## Track A: Covenant Security (S3) `[ ]`

**Description:** Investigate and fix S3 (KRC-20 output ownership validation) in `daglock_krc20.sil`.

### Phase A1: Investigate ICC Compiler Support `[ ]`
- [ ] Read SilverScript compiler for `readInputStateWithTemplate` or `OpCovInputIdx`
- [ ] Try to compile a test contract reading KCC-20 state
- [ ] Document findings
- ✅ Checkpoint: Can we write an entrypoint that reads KCC20State.ownerIdentifier from a covenant input?
- ⚙ Fallback: Cannot fix in covenant → off-chain verification (Phase A3)
- ⏱ Timebox: **2 hours**

### Phase A2: Implement S3 Fix (If Feasible) `[ ]`
- [ ] Add KCC-20 checks to `release()`, `swap()`, `refund()`
- [ ] Update execution tests for two-input transactions
- [ ] Regenerate template hash
- ✅ Checkpoint: `cargo test -p daglock-contracts --tests` passes
- ⚙ Fallback: Phase failed → off-chain fallback A3

### Phase A3: Off-Chain Verification Fallback `[ ]`
- [ ] Add template hash + seller pubkey verification in indexer
- [ ] Update SECURITY.md
- ✅ Checkpoint: Lifecycle tests pass

---

## Track B: Code Quality `[ ]`

### Phase B1: Remove `.unwrap()` in Production `[ ]`
- [ ] 7 UUID-gen sites → helper function
- [ ] 1 mutex → `unwrap_or_else(|e| e.into_inner())`
- [ ] 1 treasury key → proper error handling
- ✅ Checkpoint: No `.unwrap()` outside test code

### Phase B2: Fix Hardcoded Fee Denominator `[ ]`
- [ ] `escrows.rs:192` → `daglock_shared::FEE_DENOMINATOR`
- [ ] `offers.rs:276` → `daglock_shared::FEE_DENOMINATOR`
- ✅ Checkpoint: No `/ 200` in production code

### Phase B3: Fix Flaky Crypto Tests `[ ]`
- [ ] Fix `ENV_LOCK` mutex poisoning issue
- ✅ Checkpoint: `cargo test -p daglock-indexer --lib` crypto tests pass

### Phase B4: TradeHash Newtype `[ ]`
- [ ] `TradeHash([u8;32])` with `FromStr`, `Display`, serde
- [ ] Replace `validate_trade_hash()` return type
- ✅ Checkpoint: `cargo test --workspace` passes

---

## Track C: Web Polish `[ ]`

### Phase C1: Web Onboarding Modal (U7) `[ ]`
- [ ] Create `OnboardingModal` component
- [ ] First-visit detection via localStorage
- [ ] Walkthrough flow
- [ ] Tests
- ✅ Checkpoint: `cd web && npm test` passes

---

## Track D: Infrastructure & Deployment `[ ]`

### Phase D1: VPS Hardening `[ ]`
- [ ] `LimitNOFILE=65536` to systemd
- [ ] Create `daglock` user, switch from root
- ✅ Checkpoint: Indexer runs as daglock user, no fd errors

### Phase D2: Build & Deploy Mainnet Binary `[ ]`
- [ ] `cargo build --release -p daglock-indexer`
- [ ] scp to VPS, update systemd config
- ✅ Checkpoint: `/v1/health` shows mainnet

### Phase D3: Update Deploy Scripts `[ ]`
- [ ] `scripts/deploy-mainnet.sh`, `.env.example`, `railway.json`

### Phase D4: Full Smoke Test `[ ]`
- [ ] Web end-to-end
- [ ] CLI end-to-end
- [ ] Bot end-to-end
- [ ] API endpoints
- [ ] Atomic swap
- [ ] Vault

### Phase D5: wRPC Wiring `[ ]`
- [ ] Connect to external mainnet endpoint
- [ ] Verify `node_synced: true`
- ✅ Checkpoint: `/v1/health` shows node synced

---

## Track E: Launch Prep `[ ]`

### Phase E1: Demo Video `[ ]`
### Phase E2: Announcement Drafts `[ ]`
### Phase E3: Monitoring Setup `[ ]`

---

## Dependency Graph

```
A1 ──→ A2 ──→ template hashes ──→ D2
 │                                ↑
 └──→ A3 ──→ SECURITY.md ────────┘
                                  
B1 → B2 → B3 → B4 ──────────────→ D4
                                  
C1 ─────────────────────────────→ D4
                                  
D1 → D2 → D3 ──────────────────→ D4 → D5 → E1/E2/E3
```
