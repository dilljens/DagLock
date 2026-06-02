# DagLock SilverScript Cheat Sheet

> Subset of SilverScript that DagLock actually uses. Read this first.
> Full reference: `SILVERSCRIPT-TUTORIAL.md` and `BUILTINS.md` in this directory.

---

## Contract Structure (Exact Pattern)

```javascript
pragma silverscript ^0.1.0;

contract ContractName(
    paramType paramName,    // constructor parameters
    ...
) {
    // No mutable fields needed for simple escrow.
    // For stateful patterns, see COVENANT-DECL.md.

    entrypoint function entrypointName(params...) {
        // spending logic
    }
}
```

**Rules:**
- One contract per file
- At least one `entrypoint function`
- Constructor params become part of the compiled redeem script (immutable)
- Multiple entrypoints → compiler auto-generates selectors (0, 1, 2...)

---

## Types DagLock Uses

| Type | Size | DagLock Use |
|---|---|---|
| `int` | 64-bit signed | Amounts (sompi), fees, timestamps |
| `bool` | 1 bit | Condition results |
| `byte[]` | variable | Arbitrary data, preimage for atomic swaps |
| `byte[32]` | 32 bytes | Blake2b and SHA-256 hashes |
| `byte[34]` | 34 bytes | P2PK scriptPubKey |
| `pubkey` | 32 bytes | Compressed secp256k1 public key |
| `sig` | 65 bytes | ECDSA signature |

---

## Builtins DagLock Uses

### Signatures

```javascript
require(checkSig(sig, pubkey));  // Verify ECDSA signature → bool
```

### Hashing

```javascript
byte[32] hash = blake2b(data);   // BLAKE2b → byte[32]
byte[32] hash = sha256(data);    // SHA-256 → byte[32]
```

### ScriptPubKey Construction

```javascript
byte[34] p2pk = new ScriptPubKeyP2PK(pubkey);             // Pay-to-Public-Key
byte[35] p2sh = new ScriptPubKeyP2SHFromRedeemScript(script); // Pay-to-Script-Hash
```

### Transaction Introspection

```javascript
// Current input
int inputIdx = this.activeInputIndex;
int inputValue = tx.inputs[this.activeInputIndex].value;
byte[] inputScript = tx.inputs[this.activeInputIndex].scriptPubKey;

// Outputs
int outputValue = tx.outputs[i].value;
byte[] outputScript = tx.outputs[i].scriptPubKey;

// Transaction metadata
int version = tx.version;
int locktime = tx.locktime;      // Unix timestamp
int txTime = tx.time;            // Transaction time (Unix timestamp)

// UTXO age
int age = this.age;              // Seconds since UTXO creation
```

### Time Comparisons

```javascript
require(tx.time >= timeout);     // Transaction must be after deadline
require(this.age >= period);     // UTXO must be at least X seconds old
```

### Control Flow

```javascript
require(condition);              // Fail if false
require(condition, "message");   // Fail with message
if (condition) { ... } else { ... }
```

---

## Common Patterns

### Multi-Signature (Two Separate Checks)

```javascript
entrypoint function mutual_release(sig buyerSig, sig sellerSig) {
    require(checkSig(buyerSig, buyerPubKey));
    require(checkSig(sellerSig, sellerPubKey));
}
```

### Fee Extraction (Fixed Percentage)

```javascript
int inputValue = tx.inputs[this.activeInputIndex].value;
int feeAmount = inputValue / 200;         // 0.5%
int sendAmount = inputValue - feeAmount;
int minerFee = 1000;                      // Reserve for network fee

require(tx.outputs[0].value == sendAmount - minerFee);  // Main output
require(tx.outputs[1].value == feeAmount);               // Treasury output
```

### Enforce Recipient Address

```javascript
byte[34] recipientScript = new ScriptPubKeyP2PK(recipientPubKey);
require(tx.outputs[0].scriptPubKey == recipientScript);
```

### Hash Preimage Check (Atomic Swap)

```javascript
entrypoint function swap(byte[] secret) {
    require(sha256(secret) == expectedHash);
    // ... fee extraction, output checks ...
}
```

### Time-Locked Refund

```javascript
entrypoint function refund(sig depositorSig) {
    require(tx.time >= timeout);              // Deadline passed
    require(checkSig(depositorSig, depositorPubKey));
    // Full amount back, no fee
}
```

---

## Transaction Assembly (Rust)

```rust
use silverscript_lang::compiler::{compile_contract, CompileOptions};

let source = std::fs::read_to_string("daglock.sil")?;
let constructor_args = vec![
    buyer_pk.into(),
    seller_pk.into(),
    trade_hash.into(),
    timeout.into(),
    treasury_pk.into(),
];

let compiled = compile_contract(&source, &constructor_args, CompileOptions::default())?;

// Build signature script for a specific entrypoint
// Selector 0 = release, 1 = swap, 2 = refund (order matches source)
let sigscript = compiled.build_sig_script("release", vec![buyer_sig.into(), seller_sig.into()])?;
// → <buyer_sig> <seller_sig> <0>

let sigscript = compiled.build_sig_script("refund", vec![buyer_sig.into()])?;
// → <buyer_sig> <2>

let sigscript = compiled.build_sig_script("swap", vec![secret.into()])?;
// → <secret> <1>
```

---

## Compiler CLI

```bash
# Compile with constructor args
silverc daglock.sil --constructor-args args.json -o daglock.json

# Output: daglock.json contains:
#   .script           - compiled bytecode (array of hex bytes)
#   .abi              - entrypoint function signatures
#   .contract_name    - "DagLock"
#   .compiler_version - e.g. "0.1.0"
```

---

## Template Hash Extraction (Custom Tool)

The compiled bytecode embeds constructor parameters. To get a stable template hash:

```
1. Compile with all-zero constructor params → bytecode_zero
2. Compile with real constructor params → bytecode_real
3. Diff the two byte by byte to find prefix_len (bytes before first difference)
4. The parameters start at prefix_len. They have known fixed sizes:
     pubkey = 32 bytes
     pubkey = 32 bytes
     byte[32] = 32 bytes
     int = 8 bytes (varint)
     pubkey = 32 bytes
     Total param_size = 136 bytes
5. suffix_start = prefix_len + param_size
6. template_bytes = bytecode[0..prefix_len] ++ bytecode[suffix_start..]
7. template_hash = blake2b(template_bytes)[0..20]  (first 20 bytes)
```

---

## DagLock Entrypoint Selectors

| Selector | Entrypoint | Parameters | Use case |
|---|---|---|---|
| 0 | `release` | `(sig buyerSig, sig sellerSig)` | Both parties sign to settle |
| 1 | `swap` | `(byte[] secret)` | Hash preimage revealed for atomic swap |
| 2 | `refund` | `(sig buyerSig)` | Depositor reclaims after timeout |
