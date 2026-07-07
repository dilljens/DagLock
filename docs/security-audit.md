# Security Audit Report — July 6, 2026

## CRITICAL (4 issues — fix immediately)

### C1: No-sig paths have no destination validation
**Files:** `daglock.sil`, `daglock_advanced.sil`
**Status:** ✅ FIXED

`auto_settle()` and `emergency_refund()` don't validate output[0].scriptPubKey. Since these paths require NO signature, a malicious third party could broadcast them with their own address as the recipient, stealing funds.

**Fix applied:** Added `byte[34] sellerScript/buyerScript = new ScriptPubKeyP2PK(pubkey(...))` and `require(tx.outputs[0].scriptPubKey == byte[](sellerScript/buyerScript))` to all 4 affected entrypoints.

### C2: Subscription covenant doesn't enforce timing + remaining balance broken
**File:** `daglock_subscription.sil`
**Status:** ✅ FIXED

Two issues:
1. `intervalSeconds` parameter existed but was NEVER CHECKED — recipient could claim all installments immediately
2. `claim()` didn't account for the remaining balance after a partial claim — the input holds `totalAmount` but outputs only accounted for `installmentAmount`, making the transaction invalid

**Fix applied:** Added `currentPeriod` constructor parameter. `claim()` now:
- Enforces `tx.time >= startTime + (currentPeriod * intervalSeconds)`
- Re-locks remaining balance in a new covenant with `currentPeriod + 1`
- Outputs: [netAmount to recipient] [fee to treasury] [remaining → re-lock if > 0]

### C3: Vault key compromise allows theft (defense-in-depth)
**Files:** `daglock_vault.sil`, `daglock_vault_multisig.sil`, `daglock_vault_softlock.sil`
**Status:** ✅ FIXED (vault only)

If a vault key is stolen, the thief could withdraw funds to their own address. The covenant didn't validate output[0].scriptPubKey on withdraw/early_exit/heir_withdraw paths.

**Fix applied to vault:** Added `require(tx.outputs[0].scriptPubKey == byte[](ownerScript/heirScript))` to all vault entrypoints. This ensures stolen keys can only send to the vault owner's address (the thief would need to also control that address).

**Note:** Our escrow release paths intentionally don't validate output[0] — both parties must sign, so they review the tx. Vaults enforce destination because a single compromised key could drain funds.

## HIGH (3 issues)

### H1: Subscription rate limiting
**File:** `daglock_subscription.sil`
**Status:** ❌ UNFIXED

See C2 above. The subscription contract needs timing enforcement before mainnet.

### H2: No minimum fee check on tiny escrows
**Files:** All escrow covenants

If `inputValue < 200`, `inputValue / 200 = 0`, so no fee is paid. Combined with MIN_OUT = 1000, this means any input less than ~200,200 sompi (0.002 KAS) pays zero fee. This is a small amount but could be used to spam the treasury.

**Fix:** Add `require(feeAmount > 0 || inputValue < MIN_OUT)` or just accept as is (dust-level amounts).

### H3: Integer division truncation in split paths
**Files:** `daglock.sil`, `daglock_advanced.sil`, `daglock_arbiter.sil`, `daglock_multi.sil`

Split paths use `distributable * buyerShareBasis / 10000`. Integer division truncates, so the sum of buyer + seller amounts may be 1 less than distributable. This 1 sompi stays in the UTXO or is lost.

**Severity:** LOW — 1 sompi is negligible ($0.000000001). Fix is `sellerAmount = distributable - buyerAmount` which is what most files do. Verified that `daglock_advanced.sil` uses this pattern correctly. `daglock.sil` also uses it correctly.

## MEDIUM (2 issues)

### M1: Relay/escrow ID entropy
**File:** `indexer/src/api/escrows.rs` — IDs are UUID prefix only

Escrow IDs use the first segment of a UUID (`format!("esc_{}", Uuid::new_v4().to_string().split('-').next())`). This gives 8 hex chars = 32 bits of entropy. Sufficient for preventing enumeration in practice but not cryptographic.

### M2: No input size limits on chat messages server-side
**File:** `indexer/src/api/messages.rs` — client-side encrypts, server stores ciphertext

The server validates `content_enc` is non-empty hex but doesn't limit the size of ciphertext. A user could POST a 1GB encrypted blob. This is mitigated by the 1MB body limit in `api/mod.rs:170` (`RequestBodyLimitLayer::new(1024 * 1024)`).

## LOW (4 issues)

### L1: No output[0] destination on multi-path escrow release
**File:** `daglock_multi.sil` release path validates all output scripts correctly. Verified.

### L2: Timestamp vs DAA mismatch on indexer side
The indexer's `expiration_daa_score` uses DAA blocks, while the covenant uses Unix timestamps. These can drift.

### L3: No CSRF protection on web API
The web API doesn't use CSRF tokens. Relies on signature-based auth which is not vulnerable to CSRF (attacker can't forge signatures).

### L4: Rate limiter doesn't distinguish auth failures from real requests
Brute-force attempts against the auth system count toward the rate limit. This means a legitimate user could be locked out if an attacker is brute-forcing their address.
