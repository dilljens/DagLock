# KRC-20 Testnet Guide

## Prerequisites
- A running Kaspa Testnet 12 node with wRPC enabled (`--rpclisten-borsh=0.0.0.0:17110`)
- `silverc` compiler in PATH (from silverscript-lang)
- DagLock indexer with template hashes configured

## Template Hashes (Current)

```
KAS:   30876e3ea42d0e23bb0980f3fd97ae8807e9c70f
ARB:   0aefa1159f27a04fd2b9fa162386a6e8ce15d098
KRC20: 1782946f93219d54799f47b40c730d6771527c43
```

## Step 1: Deploy KCC-20 Token

The KCC-20 contract is the standard Kaspa token contract. Deploy it first:

```bash
# Compile KCC-20
silverc compile contracts/sil/KCC20.sil -o kcc20_compiled.json

# Deploy (broadcast to TN12)
# This creates the KCC-20 contract with covenant ID A
kaspawallet deploy-contract --compiled kcc20_compiled.json
# => Covenant ID: kcc20_<id>
```

## Step 2: Deploy DagLockKRC20

DagLockKRC20 needs the KCC-20 template metadata and covenant ID as constructor params:

```bash
# Extract KCC-20 template metadata
silverc template-meta kcc20_compiled.json
# => prefix_len, suffix_len, template_hash, prefix_hex, suffix_hex

# Deploy DagLockKRC20 with the KCC-20 metadata
silverc compile contracts/src/daglock_krc20.sil \
  --arg bytes:<buyer_pubkey> \
  --arg bytes:<seller_pubkey> \
  --arg bytes:<trade_hash_or_zeroes> \
  --arg int:<timeout_timestamp> \
  --arg bytes:<treasury_pubkey> \
  --arg int:<kcc20_prefix_len> \
  --arg int:<kcc20_suffix_len> \
  --arg bytes:<kcc20_template_hash> \
  --arg bytes:<kcc20_prefix> \
  --arg bytes:<kcc20_suffix> \
  --arg bytes:<kcc20_covenant_id> \
  -o daglock_krc20_compiled.json
```

## Step 3: Create Test Tokens

Mint test tokens using the KCC-20 minter:

```bash
# Mint 1,000,000 test tokens
kaspawallet call-contract \
  --covenant-id <kcc20_covenant_id> \
  --entrypoint mint \
  --arg bytes:<daglock_krc20_covenant_id> \
  --arg int:1000000 \
  --arg bool:false
```

## Step 4: Transfer Tokens to DagLockKRC20 Escrow

```bash
# Transfer tokens to a KCC-20 branch owned by DagLockKRC20
kaspawallet call-contract \
  --covenant-id <kcc20_covenant_id> \
  --entrypoint transfer \
  --arg bytes:<daglock_krc20_covenant_id> \
  --arg int:<amount> \
  --arg bytes:<daglock_krc20_covenant_id_as_identifier> \
  --arg byte:0x02  # IDENTIFIER_COVENANT_ID
```

## Step 5: Settle the KRC-20 Escrow

Create a transaction with TWO inputs:
1. The KCC-20 branch (token UTXO)
2. The DagLockKRC20 UTXO (authorization gate)

Both covenants must be satisfied for the transaction to succeed.

## ICC Pattern Reference

See `docs/reference/KRC20-ICC.md` for the full protocol specification.
