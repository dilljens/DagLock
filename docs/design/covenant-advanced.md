# DagLock Advanced Covenant — Time Extension + Partial Swap

**Status:** Implemented in SilverScript. Needs audit before mainnet deployment.

**Template Hash:** `98afa7aa6703712d27c48e61b3439e02a38b8eac`

## Overview

Extends the base `daglock.sil` covenant with two additional spending paths:

### 1. `extendTimeout(buyerSig, sellerSig, newTimeout, newCovenantScript)`

Both parties sign to re-lock funds with a later timeout.

**Parameters:**
- `buyerSig`: buyer's Schnorr signature
- `sellerSig`: seller's Schnorr signature
- `newTimeout`: new Unix timestamp (must be > original timeout)
- `newCovenantScript`: pre-computed DagLockAdvanced script with the new timeout

**Outputs:**
- Output 0: full amount - fee, locked in new covenant (re-lock)
- Output 1: fee to treasury

**Flow:**
1. Both parties agree on a new timeout
2. They pre-compute the new covenant address with the updated timeout
3. They both sign the transaction
4. Transaction re-locks funds in the new covenant UTXO
5. The old covenant UTXO is spent (gone)

### 2. `swap_partial(secret, amountToSeller)`

Atomic swap of a portion of locked funds. The remainder returns to the buyer.

**Parameters:**
- `secret`: preimage that SHA-256 hashes to `tradeHash`
- `amountToSeller`: amount in sompi to send to the counterparty

**Outputs:**
- Output 0: amountToSeller - fee → counterparty
- Output 1: fee → treasury
- Output 2: inputValue - amountToSeller + fee → buyer (change)

**Fee: 0.5% on the settled portion only** (not on the full amount)

## Security Considerations

- **Extend timeout**: The new covenant script must be passed in and validated by the covenant. If the script doesn't match the expected format, funds could be locked incorrectly.
- **Partial swap**: Fee on partial amount means smaller treasury income. Dust prevention: minimum settlement amount of 1000 sompi.
- Both paths add `require(amount > 0)` checks to prevent zero-value outputs.

## Migration

| File | Change |
|------|--------|
| `contracts/src/daglock_advanced.sil` | **New** — covenant source |
| `contracts/src/lib.rs` | Added `compile_daglock_advanced()`, source fn, template hash |
| `contracts/tests/` | Add execution tests for extendTimeout + swap_partial |
| `indexer/src/config.rs` | Register `daglock_advanced_template` |
| `indexer/src/listener.rs` | Add template hash to UTXO matcher |
