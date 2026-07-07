# Findings: DAA-Block Timing Research

## What `tx.time` vs `this.age` Means

| Expression | Script Opcode | What It Checks | Kaspa Semantic |
|------------|---------------|----------------|----------------|
| `require(tx.time >= X)` | `OP_CHECKLOCKTIMEVERIFY` | `tx.nLockTime >= X` | **Absolute Unix timestamp deadline** |
| `require(this.age >= X)` | `OP_CHECKSEQUENCEVERIFY` | input `nSequence` implies ≥X maturity | **Relative DAA-block maturity** |

They are **different tools for different jobs** — NOT interchangeable.

## Current Usage in DagLock Contracts

All 19 `tx.time` usages across 10 `.sil` files are for **absolute deadlines** (refund after timestamp, auto-settle after timestamp, emergency after timestamp). This is the correct use of `tx.time`. The timeouts are always constructor parameters, never hardcoded.

## OfficeForge's Pattern

OfficeForge uses **both** based on need:
- `this.age >= disputeWindow` — relative cooldown for dispute window (blocks from funding)
- `this.age >= arbiterDeadline` — relative timeout for arbiter ruling
- `tx.time` is used where absolute deadlines make sense

Their dispute window is relative to funding, which makes `this.age` appropriate. Our escrows use absolute timestamps for refund deadlines.

## Kaspa Timing Properties

| Property | Unix Timestamp | DAA Score |
|----------|---------------|-----------|
| Predictability | ~constant forward flow | Exactly 1 increment per block (~1/sec avg) |
| Manipulable? | Miners can shift ±2h | No — strictly requires real PoW |
| On-chain opcode | `OP_CLTV` (`tx.time`) | `OP_CSV` (`this.age`) |
| Absolute or relative? | **Absolute** | **Relative** (maturity) |

## Recommendation

**Keep `tx.time` for escrow contracts** — absolute deadlines are the correct semantic for refund/auto-settle/emergency paths.

**Consider switching vault contracts to `this.age`** — vaults are about minimum holding periods ("must hold N blocks"), which is a relative maturity, not an absolute deadline. This would improve vault security (no timestamp manipulation possible).

**Fix the subscription contract** — `intervalSeconds` is passed but never enforced on-chain. The contract relies entirely on mutual signatures for timing.

**Fix the indexer mismatch** — `expiration_daa_score` (DAA-based) doesn't align with `tx.time` (timestamp-based) in the covenant. This can cause confusing off-chain vs on-chain status drift.

## What Would Change

If we convert vaults to `this.age`:
- `daglock_vault.sil`: 3 `tx.time` → `this.age`, rename `timeout` → `lockDuration`
- `daglock_vault_softlock.sil`: 1 change
- `daglock_vault_multisig.sil`: 2 changes
- Template hashes change for all three
- Constructor params change from absolute timestamps to block counts
- Indexer `expiration_daa_score` alignment improves
