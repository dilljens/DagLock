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
