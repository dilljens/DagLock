# DagLock Protocol Specification

> Exact semantics of the DagLock SilverScript covenants: transaction structure, parameter encoding, script hash derivation, and spending rules. Covers both native KAS and KRC-20 token escrows.

---

## 1. Covenant Overview

DagLock uses **SilverScript covenants** — compiled Kaspa scripts that enforce spending conditions on UTXOs. The covenant is parameterized at deployment time and executes entirely within Kaspa's native script engine.

Two covenants exist:

| Covenant | Asset | File |
|---|---|---|
| **DagLock KAS** | Native KAS | `daglock.sil` |
| **DagLock KRC-20** | KRC-20 tokens | `daglock_krc20.sil` |

---

## 2. DagLock KAS Covenant (`daglock.sil`)

### 2.1 Contract Source

```javascript
pragma silverscript ^0.1.0;

contract DagLock(
    pubkey buyerPubKey,
    pubkey sellerPubKey,
    byte[32] tradeHash,       // SHA-256 of atomic-swap secret (zeroes if unused)
    int timeout,               // Unix timestamp after which refund is allowed
    pubkey treasuryPubKey      // DagLock fee treasury address
) {
    // Path A: Mutual Release — both parties sign
    entrypoint function release(sig buyerSig, sig sellerSig) {
        require(checkSig(buyerSig, buyerPubKey));
        require(checkSig(sellerSig, sellerPubKey));

        int inputValue = tx.inputs[this.activeInputIndex].value;
        int feeAmount = inputValue / 200;
        int sendAmount = inputValue - feeAmount;

        require(tx.outputs[0].value == sendAmount);

        byte[34] treasuryScript = new ScriptPubKeyP2PK(treasuryPubKey);
        require(tx.outputs[1].value == feeAmount);
        require(tx.outputs[1].scriptPubKey == treasuryScript);
    }

    // Path B: Atomic Swap — hash preimage reveal
    entrypoint function swap(byte[] secret) {
        require(sha256(secret) == tradeHash);

        int inputValue = tx.inputs[this.activeInputIndex].value;
        int feeAmount = inputValue / 200;
        int sendAmount = inputValue - feeAmount;

        require(tx.outputs[0].value == sendAmount);

        byte[34] treasuryScript = new ScriptPubKeyP2PK(treasuryPubKey);
        require(tx.outputs[1].value == feeAmount);
        require(tx.outputs[1].scriptPubKey == treasuryScript);
    }

    // Path C: Timeout Refund — depositor reclaims, no fee
    entrypoint function refund(sig buyerSig) {
        require(tx.time >= timeout);
        require(checkSig(buyerSig, buyerPubKey));

        int inputValue = tx.inputs[this.activeInputIndex].value;
        require(tx.outputs[0].value == inputValue);
    }
}
```

### 2.2 Parameter Encoding

When deployed, constructor parameters are encoded into the redeem script:

```
[buyerPubKey (32 bytes)] [sellerPubKey (32 bytes)] [tradeHash (32 bytes)]
[timeout (8 bytes varint)] [treasuryPubKey (32 bytes)]
Total param size: 136 bytes
```

### 2.3 Entrypoint Selectors

| Selector | Entrypoint | Args | When Used |
|---|---|---|---|
| 0 | `release` | `(sig buyerSig, sig sellerSig)` | Both parties sign to settle |
| 1 | `swap` | `(byte[] secret)` | Hash preimage revealed for atomic swap |
| 2 | `refund` | `(sig buyerSig)` | Depositor reclaims after timeout |

---

## 3. DagLock KRC-20 Covenant (`daglock_krc20.sil`)

### 3.1 Architecture: Inter-Covenant Communication (ICC)

KRC-20 tokens use the KCC-20 standard. DagLockKRC20 does NOT hold token balances internally. Instead, it **owns** a KCC-20 branch via `IDENTIFIER_COVENANT_ID` ownership, and the KCC-20 contract itself enforces that DagLockKRC20 must authorize any transfer.

This follows the same ICC pattern as KCC20Minter in the SilverScript repo. See `docs/reference/KRC20-ICC.md` for the full transaction structure and deployment lifecycle.

**KCC-20 handles:** Token conservation, ownership validation, minting prevention.
**DagLockKRC20 handles:** Escrow conditions (signatures, timeout) + treasury fee enforcement.

### 3.2 Contract Source

See `contracts/src/daglock_krc20.sil` for the current source. Key design decisions:

- Uses `entrypoint function` (simple pattern) rather than `#[covenant]` declarations (complex pattern). The declaration API may be needed for reading KCC-20 state via `readInputStateWithTemplate`, but the simple entrypoint approach is the starting point.
- Stores KCC-20 template metadata (prefix, suffix, expected hash) in constructor params — allows validation that KCC-20 outputs are genuine.
- Stores `kcc20CovenantId` to locate KCC-20 inputs/outputs in the transaction via `OpCovInputIdx`/`OpCovOutputIdx` (requires covenant context).
- Fee validation: iterates tx.outputs to find the treasury address rather than hardcoding a specific output index (since KCC-20 output topology varies).

### 3.3 Two Implementation Strategies

| Approach | File | Status |
|---|---|---|
| **ICC pattern** (DagLockKRC20 owns KCC-20 branch via covenant-ID) | `daglock_krc20.sil` | Written. Needs compiler verification for `OpCovInputIdx` in entrypoint context. |
| **Direct pattern** (DagLockKRC20 stores token state internally) | `daglock_krc20_direct.sil` | Fallback. Simpler but duplicates KCC-20 validation logic. To be written if ICC pattern fails to compile. |

**Recommendation:** Ship both. Use whichever compiles and passes TxScriptEngine tests.

### 3.4 KCC-20 State Layout (Fixed)

All KCC-20 branches share this state structure:

| Field | Type | Size | Description |
|---|---|---|---|
| `ownerIdentifier` | `byte[32]` | 32 bytes | Pubkey, script hash, or covenant ID |
| `identifierType` | `byte` | 1 byte | 0x00=PUBKEY, 0x01=SCRIPT_HASH, 0x02=COVENANT_ID |
| `amount` | `int` | 8 bytes | Token balance |
| `isMinter` | `bool` | 1 byte | Mint/burn authority |

DagLockKRC20 sets `identifierType = 0x02` and `ownerIdentifier = DagLockKRC20's covenant ID`.

---

## 4. Transaction Structure

### 4.1 Lock Transaction (Create Escrow)

```
Input(s):   [depositor's UTXO(s)]
Outputs:
  Output 0:  DagLock P2SH (value = KAS amount or KRC-20 commitment)
               scriptPubKey = P2SH(compiled_covenant(params))
  Output 1:  Change (if any)
```

### 4.2 Release Transaction (Path A — Settlement)

```
Inputs:
  Input 0:  DagLock UTXO
              sigScript = [buyerSig] [sellerSig] [selector-0] [covenant_bytecode]
Outputs:
  Output 0:  Recipient address (value = deposit_amount - feeAmount)
  Output 1:  Treasury address (value = feeAmount = deposit_amount / 200)
```

### 4.3 Refund Transaction (Path C — Timeout)

```
Inputs:
  Input 0:  DagLock UTXO
              sigScript = [buyerSig] [selector-2] [covenant_bytecode]
Outputs:
  Output 0:  Depositor's address (value = full_deposit_amount)
```

---

## 5. Fee Schedule

| Event | Fee (KAS) | Fee (KRC-20) | Notes |
|---|---|---|---|
| Mutual settlement | 0.5% (1/200) | Negotiated off-chain | Deducted from output value / separate KAS fee output |
| Atomic swap settlement | 0.5% (1/200) | Negotiated off-chain | Same as mutual |
| Timeout refund | 0% | 0% | No fee; depositor gets full amount |

**Volume-based rebates:** Traders exceeding volume thresholds receive off-chain rebates tracked by the indexer. The covenant fee remains 0.5% for auditability; rebates are sent as separate transactions from the treasury.

| Tier | 30-day Volume (KAS) | Effective Fee |
|---|---|---|
| Standard | < 100,000 | 0.50% |
| Silver | 100,000 – 1,000,000 | 0.35% |
| Gold | 1,000,000 – 10,000,000 | 0.25% |
| Platinum | > 10,000,000 | 0.15% |

---

## 6. Counterparty Discovery Protocol

### 6.1 Offer Lifecycle

```
               
 PROPOSED  ACCEPTED   LOCKED   SETTLED  
(no funds      (term          (funds         (complete)
 locked)        agreed)        on-chain)               
               
                                                       
      EXPIRED 
                        (timeout, no action)
```

### 6.2 Offer API

```
POST /v1/offers
{
  "side": "buy",
  "base_asset": "KAS",
  "quote_asset": "KRC20:NACHO",
  "amount": "500000000000",
  "counterparty_address": null,
  "expires_at": "2026-06-15T12:00:00Z"
}
```

An offer remains in PROPOSED state until a counterparty accepts it. No funds are locked during the proposal phase. Once accepted, the depositor funds the escrow on-chain.

---

## 7. Settlement Receipts

After a trade completes, a receipt is generated containing all on-chain verification data:

```json
{
  "receipt_id": "rct_abc123",
  "escrow_id": "esc_abc123",
  "status": "settled",
  "asset": "KAS",
  "amount": "5000.0",
  "fee": "25.0",
  "buyer_address": "kaspa:qz2q...",
  "seller_address": "kaspa:qz9x...",
  "lock_tx_id": "ab12cd34...",
  "settle_tx_id": "ef56gh78...",
  "lock_daa_score": 12500000,
  "settle_daa_score": 12500100,
  "settled_at": "2026-06-15T14:30:00Z",
  "verification": {
    "covenant_verified": true,
    "signatures_verified": true,
    "fee_compliant": true
  }
}
```

Receipts are tamper-evident: the receipt ID is the BLAKE2b hash of the receipt JSON.

---

## 8. Atomic Swap Protocol (UX-Abstracted)

The user-facing flow hides all cryptographic complexity:

```
User A (Buyer)                     User B (Seller)
                                        
       1. "Swap 5000 KAS for 100K NACHO"
          App generates random secret S  
          Computes H = SHA-256(S)        
     
                                        
       2. Deploys DagLock(tradeHash=H)  
          with 5000 KAS                 
          Shares escrow link            
     
                                        
                              3. Verifies terms
                                 Deploys counterparty escrow
                                 with same H, 100K NACHO
                                        
       4. Claims B's escrow by           
          revealing S on-chain          
     
                                        
                              5. Sees S on-chain
                                 Uses S to claim A's escrow
                                 Swap complete
```

The app manages secret generation, hash sharing, and dual-escrow coordination. Users see: "Swapping 5000 KAS for 100K NACHO... Step 2 of 5: Waiting for counterparty."

---

## 9. Protocol Constants

| Constant | Value | Rationale |
|---|---|---|
| Fee numerator | 1 | — |
| Fee denominator | 200 | 0.5% |
| Max expiration | ~30 days (Unix timestamp) | UI-enforced |
| Min KAS lock | 100 KAS (10,000,000 sompi) | Fee output above KIP-0009 dust |
| Min KRC-20 lock | Token-dependent | Above network dust threshold |
| Trade hash size | 32 bytes | SHA-256 output |
| Template hash size | 20 bytes | BLAKE2b-160 (P2SH length) |
| Ticker size | 8 bytes | KRC-20 ticker standard |
