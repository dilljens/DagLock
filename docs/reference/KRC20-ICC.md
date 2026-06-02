# DagLock KRC-20: Inter-Covenant Communication Pattern

> How DagLockKRC20 and KCC-20 contracts cooperate in the same transaction.

---

## The Pattern

DagLockKRC20 does NOT hold token balances. Instead, it **owns** a KCC-20 branch via covenant-ID, and the KCC-20 contract itself enforces that DagLockKRC20 must authorize any transfer.

```
┌──────────────────────────────────────────────────────────┐
│                    Settlement Transaction                │
│                                                          │
│  INPUTS:                                                 │
│  ┌──────────────────────────────────────────────┐        │
│  │ Input 0: KCC-20 Branch                       │        │
│  │   State: {                                    │        │
│  │     ownerIdentifier: <DagLock covenant ID>    │        │
│  │     identifierType: IDENTIFIER_COVENANT_ID    │        │
│  │     amount: 100000 (NACHO)                    │        │
│  │     isMinter: false                           │        │
│  │   }                                           │        │
│  └──────────────────────────────────────────────┘        │
│                                                          │
│  ┌──────────────────────────────────────────────┐        │
│  │ Input 1: DagLockKRC20 UTXO                   │        │
│  │   Covenant ID = <D>                          │        │
│  │   Contains escrow terms:                     │        │
│  │     buyerPubKey, sellerPubKey, timeout       │        │
│  └──────────────────────────────────────────────┘        │
│                                                          │
│  OUTPUTS:                                                │
│  ┌──────────────────────────────────────────────┐        │
│  │ Output 0: KCC-20 Branch (new)                │        │
│  │   ownerIdentifier: sellerPubKey               │        │
│  │   amount: 100000 (same, conserved)            │        │
│  └──────────────────────────────────────────────┘        │
│  ┌──────────────────────────────────────────────┐        │
│  │ Output 1: KAS fee → DagLock Treasury         │        │
│  └──────────────────────────────────────────────┘        │
└──────────────────────────────────────────────────────────┘

KCC-20 transfer() checks:
  checkSigs(): OpInputCovenantId(1) == Input 0's ownerIdentifier
               → Input 1 is DagLockKRC20 with covenant ID D ✓
  checkAmounts(): amount_in == amount_out → 100000 == 100000 ✓

DagLockKRC20 release() checks:
  checkSig(buyerSig, buyerPubKey) ✓
  checkSig(sellerSig, sellerPubKey) ✓
  Treasury fee output exists ✓

Both contracts satisfied → transaction confirmed.
```

---

## Deployment Lifecycle

```
STEP 1: Deploy DagLockKRC20
─────────────────────────────
Creator → DagLockKRC20(buyer, seller, tradeHash, timeout,
                       treasury, templateMeta, kcc20CovenantId)
        → Covenant ID = D

STEP 2: Create KCC-20 Escrow Branch
────────────────────────────────────
Depositor → KCC-20.transfer()
          → New KCC-20 output:
              ownerIdentifier = D
              identifierType = IDENTIFIER_COVENANT_ID (0x02)
              amount = 100000 (NACHO tokens)
              isMinter = false

STEP 3: Settlement (Release/Swap)
──────────────────────────────────
Transaction with BOTH inputs (KCC-20 branch + DagLockKRC20 UTXO)
  → KCC-20 validates: DagLockKRC20 authorizes this transfer
  → DagLockKRC20 validates: escrow conditions met
  → Both pass → new KCC-20 branch created (seller-owned) + fee to treasury

STEP 4: Refund (Timeout)
─────────────────────────
Transaction with BOTH inputs (KCC-20 branch + DagLockKRC20 UTXO)
  → KCC-20 validates: DagLockKRC20 authorizes this transfer
  → DagLockKRC20 validates: timeout passed + buyer signed
  → Both pass → new KCC-20 branch created (buyer-owned)
```

---

## What DagLockKRC20 Does NOT Do

| Concern | Handled By | Mechanism |
|---|---|---|
| Token conservation (amount in = out) | KCC-20 | `checkAmounts()` — non-minter branch |
| Ownership authorization | KCC-20 | `checkSigs()` — `OpInputCovenantId == ownerIdentifier` |
| Token type validation | KCC-20 | All branches share same covenant ID A |
| No-minter-creation | KCC-20 | `checkMintingTransfer()` — non-minter can't create minters |
| Escrow conditions (signatures, timeout) | DagLockKRC20 | `release()`, `swap()`, `refund()` entrypoints |
| Treasury fee | DagLockKRC20 | Loop over tx.outputs checking scriptPubKey |

---

## Template Metadata

Like KCC20Minter, DagLockKRC20 stores KCC-20 template metadata to validate KCC-20 inputs/outputs:

| Param | Purpose |
|---|---|
| `kcc20TemplatePrefixLen` | Byte length of KCC-20 bytecode before state fields |
| `kcc20TemplateSuffixLen` | Byte length of KCC-20 bytecode after state fields |
| `kcc20ExpectedTemplateHash` | `blake2b(prefix || suffix)` for the KCC-20 contract |
| `kcc20TemplatePrefix` | The prefix bytes themselves |
| `kcc20TemplateSuffix` | The suffix bytes themselves |
| `kcc20CovenantId` | The KCC-20 contract's covenant ID (to locate inputs/outputs) |

These are fixed per token ticker (e.g., all NACHO branches share the same KCC-20 covenant ID and template).

---

## Open Questions (Need Compiler Verification)

1. **Can `entrypoint function` use `OpCovInputIdx`?** The existing KCC-20 examples use `#[covenant]` declarations, not raw entrypoints. The `OpCovInputIdx` opcode may only be available in covenant context.

2. **For loop in entrypoint?** The `release()` function iterates over `tx.outputs` to find the treasury output. For loops require a compile-time unroll bound. The `8` parameter is the max unroll count.

3. **KCC-20 input needs to be a covenant UTXO.** The `pay_to_script_hash_script(&kcc20_script)` for the KCC-20 branch has a non-null covenant binding. The DagLockKRC20 similarly needs covenant bindings.

4. **`witnesses` parameter in KCC-20's `transfer()`.** KCC-20 expects a `byte[] witnesses` argument that maps covenant branches to their authorizing inputs. The app constructing the transaction must supply the correct witness index pointing to the DagLockKRC20 input.

---

## Fallback: Pure Entrypoint Approach

If the ICC pattern is too complex for v1 (requires verified compiler support for `OpCovInputIdx` in entrypoints), the fallback is simpler:

Instead of DagLockKRC20 "owning" a KCC-20 branch in the formal sense, DagLockKRC20 stores token state directly and validates it manually. This duplicates some KCC-20 logic but avoids inter-covenant complexity.

The tradeoff:

| Approach | Pros | Cons |
|---|---|---|
| **ICC (KCC-20 + DagLockKRC20)** | Clean separation of concerns. KCC-20 handles tokens, DagLock handles escrow. Reuses existing KCC-20 contract. | Requires compiler support for ICC. More complex tx construction. |
| **Direct (DagLockKRC20 handles everything)** | Simpler to implement. No inter-covenant coordination. | Duplicates token validation logic. Less composable. Breaks if KCC-20 standard changes. |

**Recommendation:** Build the direct approach first (Phase 0). Add ICC in Phase 2 once the compiler support is verified.

---

## Revised Phase 0 Deliverable

For Phase 0, ship TWO KRC-20 approaches:

1. **`daglock_krc20.sil`** — ICC pattern (aspirational, depends on compiler)
2. **`daglock_krc20_direct.sil`** — Direct pattern (fallback, guaranteed to work)

Test both. Use whichever compiles and passes TxScriptEngine tests.
