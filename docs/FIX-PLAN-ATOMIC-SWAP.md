# Fix Plan: Atomic Swap Gaps

## Problem Summary

The `trade_hash` (SHA-256 of the swap secret) exists in the covenant contract but is never stored in the escrow database or exposed through the API:

1. Users can't set a `trade_hash` when creating an escrow
2. The swap endpoint accepts any preimage without cryptographic verification
3. No way to generate a secret/hash pair for atomic swaps
4. The swap form doesn't show the expected hash

---

## Step-by-Step Implementation

### Step 1: Add `trade_hash` column to escrows table

New migration `011_add_trade_hash.sql`:
```sql
ALTER TABLE escrows ADD COLUMN trade_hash TEXT;
```

Update `schema.rs` to call the migration.
Update `queries.rs`: `row_to_escrow` reads `trade_hash`, `create_escrow` inserts it.

---

### Step 2: Add `trade_hash` to Rust types

In `types.rs`, add to `Escrow` struct:
```rust
pub trade_hash: Option<String>,
```

Add to `CreateEscrowRequest`:
```rust
#[serde(default)]
pub trade_hash: Option<String>,
```

---

### Step 3: Fix `atomic_swap` to verify preimage against stored hash

In `escrows.rs`, the swap endpoint currently accepts any preimage. Fix:
```rust
if let Some(ref expected_hash) = current.trade_hash {
    let hash = sha256(preimage_bytes);
    if hash != hex::decode(expected_hash)? {
        return Err(/* preimage_mismatch */);
    }
}
```

Add `sha2 = "0.10"` to `indexer/Cargo.toml`.

---

### Step 4: Add `POST /v1/swap/generate` endpoint

New file `api/swap.rs`:
```rust
pub async fn generate() -> Json<Value> {
    let secret = rand::random::<[u8; 32]>();
    let hash = sha256(&secret);
    Json(json!({"secret": hex::encode(&secret), "hash": hex::encode(&hash)}))
}
```

Add route in `mod.rs`.

---

### Step 5: Update CreateEscrowForm

Add `trade_hash` input + "Generate Secret" button.
Include `trade_hash` in the create escrow request body.

---

### Step 6: Update SwapForm

Show the escrow's `trade_hash` before submitting preimage.

---

## Estimated Effort: ~2 days
| Steps | Time |
|-------|------|
| 1-3 (backend) | 1 day |
| 4 (generate endpoint) | 0.5 days |
| 5-6 (frontend) | 1 day |
