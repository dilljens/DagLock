# Verifying DagLock Covenants

> How to verify that the compiled covenant bytecode matches the open source.

## Why verify?

DagLock is trustless by design. The covenant (smart contract) holds user funds. You should verify that the deployed bytecode matches the audited source before locking funds into it.

## Method 1: Compile from source (recommended)

```bash
# Clone the repo
git clone https://github.com/dilljens/DagLock
cd DagLock

# Compile the covenant
cargo test -p daglock-contracts -- print_template_hashes --nocapture

# Output will include:
#   daglock_kas_template_hash=30876e3ea42d0e23bb0980f3fd97ae8807e9c70f
#   daglock_arbiter_template_hash=d6aea010040d361049483c62da2e6b35f6dc256c
#   daglock_krc20_template_hash=ae0946e4a9bd4a7585e6bf9135de38083cb11c85
#   daglock_vault_softlock_template_hash=ed57b9da957beaac387a0baa9a23c8c54d186964
#   daglock_vault_multisig_template_hash=caf0b46ea425159b80af81436fc8f8cfd4e62afa
#   daglock_vault_template_hash=b338c514b1ef79bf1b0739814bc0d567e8461cfb
```

Compare the output template hash with the one claimed by the indexer at `/v1/network`. If they match, the bytecode is identical.

## Method 2: POST /v1/compile

```bash
# Ask the indexer to compile a specific template
curl -s https://api.daglock.io/v1/compile \
  -H "Content-Type: application/json" \
  -d '{"template":"daglock","params":{"buyer_key":"0000000000000000000000000000000000000000000000000000000000000000","seller_key":"0000000000000000000000000000000000000000000000000000000000000000","trade_hash":"0000000000000000000000000000000000000000000000000000000000000000","timeout":"2000000000","treasury_key":"0000000000000000000000000000000000000000000000000000000000000000"}}'

# The response includes 'template_hash'. Compare with the one you compiled locally.
```

## What template hashes mean

The template hash is a BLAKE2b-160 fingerprint of the covenant's bytecode *excluding constructor parameters*. This means:

- **Same template hash = same covenant logic.** Any DagLock KAS escrow, regardless of buyer/seller/treasury keys, shares the same template hash.
- **Different templates = different hashes.** Standard escrow, arbiter escrow, and vault have different template hashes because their logic differs.
- **Zeroed arbiter key ≠ standard daglock.** Even though the arbiter covenant with zeroed key behaves identically to the standard one, the bytecode differs (the arbiter constructor has an extra parameter), so the template hash differs.

## How the indexer identifies UTXOs

When the indexer's wRPC listener scans a Kaspa block for DagLock UTXOs, it computes the BLAKE2b-160 hash of each output script and compares it against the configured template hashes. A match means "this is a DagLock covenant, track it."

This is the same mechanism wallets and explorers use to find relevant UTXOs without scanning every transaction.
