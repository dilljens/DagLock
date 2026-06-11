# Automated On-Chain Verdict Execution

**Status:** Design sketch (not implemented)

**Author:** DagLock Engineering

**Date:** 2026-06-10

## Problem

When a jury reaches a verdict (seller_wins or buyer_wins), the verdict is only recorded in the indexer database. The actual fund movement requires the winning party to manually:
1. Create an on-chain transaction calling `disputeSellerWins()` or `disputeBuyerWins()`
2. Get the treasury/mediator to co-sign
3. Broadcast the transaction

This creates friction — if the winner doesn't know how to broadcast covenant transactions, the verdict is meaningless.

## Solution: Treasury-Key Automated Broadcast

The indexer holds the **DagLock treasury key** (the arbiter key for jury cases). After a jury verdict is reached:

1. The indexer detects verdict via a background loop (`poll_for_verdicts()`)
2. It constructs the appropriate `disputeSellerWins` or `disputeBuyerWins` transaction
3. It signs with the treasury key
4. It broadcasts the transaction via wRPC

### Security Requirements

1. **The treasury key must be stored encrypted at rest** (e.g., via `DAGLOCK_TREASURY_KEY` env var, loaded only at startup)
2. **Manual confirmation for large values** (>100K KAS): the verdict broadcast waits for operator approval via a CLI command
3. **Idempotency**: the loop must not double-broadcast. Track `broadcast_tx_id` on the jury case record
4. **Failure handling**: if the on-chain UTXO is already spent (mutual release happened during voting), skip gracefully

### Flow

```
jury verdict reached (seller_wins)
  │
  ├── auto-execution enabled? (config flag --auto-execute-verdicts)
  │     ├── Yes → check if escrow UTXO still exists
  │     │           ├── Yes → construct disputeSellerWins tx
  │     │           │         → treasury signs
  │     │           │         → broadcast via wRPC
  │     │           │         → record tx_id on jury case
  │     │           └── No  → mark case as "already_settled"
  │     └── No  → leave case as "decided" — manual only
  │
  └── notify both parties via WebSocket/webhook
```

### Juror State Updates

After broadcasting, update juror stats:
- `total_cases_assigned` (done at creation)
- `total_cases_voted` (done at vote time)
- `reliability_score` — increase for jurors who voted with the majority, decrease for minority

### Configuration

```toml
[auto_execute]
enabled = false        # opt-in by operator
min_daa_confirmations = 10
max_value_kas = 100000 # require manual approval above this
```

### Risks

| Risk | Mitigation |
|------|-----------|
| Treasury key compromised | Air-gap, HSM, or multi-sig treasury |
| Double-broadcast race | Idempotency check: `WHERE status = 'decided' AND broadcast_tx_id IS NULL` |
| Wrong verdict executed | Treasury signs per-jury-case, not blanket; verify outcome matches DB |
| Gas/dust TX costs | Use SIGHASH_SINGLE to minimize fees |
