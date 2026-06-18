# On-Chain Reputation Covenant — Research & Decisions

## Why This Matters

Kaspa has no on-chain reputation standard. Ethereum has ENS, Lens Protocol, and various identity/reputation contracts. Kaspa has nothing equivalent.

DagLock's indexer-based reputation is useful but locked inside our database. Other dApps can't read it without trusting our API. An on-chain covenant fixes this.

## The Oracle Pattern

The autonomous Kaspa oracle referenced by the user proves:
- Covenants can maintain verifiable state on Kaspa
- Kaspa consensus validates covenant logic
- Updates are permissionless if you meet the covenant's conditions (~1 cent each)
- Data is available to anyone reading the UTXO set

Reputation is the same pattern: signed data → covenant validates → UTXO state → anyone reads.

## Receipt Format Design

The receipt must contain enough information for:
1. The covenant to validate authenticity (both signatures)
2. The reader to compute a Beta score (outcome + amount + timestamp)
3. Replay protection (nonce)

```json
{
  "version": 1,
  "buyer": "32-byte pubkey hex",
  "seller": "32-byte pubkey hex",
  "amount_sompi": 100000000,
  "outcome": 0,
  "timestamp": 1781760000,
  "nonce": "8-byte-hex"
}
```

Signatures: Both parties sign `blake2b(receipt_json)` with their Schnorr key. The covenant verifies both.

## UTXO Model: Per-Trade

Each trade receipt creates a new UTXO with the trade data embedded. Multiple trades for the same address create multiple UTXOs. On the read side, you scan all UTXOs matching an address hash and aggregate them.

**Pros:** Simple covenant (no state management). No UTXO merging logic. Works with Kaspa's existing UTXO enumeration.

**Cons:** More UTXOs on chain. Read side must scan and aggregate.

## Memory / Storage Estimates

| Scenario | UTXOs | Size | Annual growth |
|----------|-------|------|---------------|
| DagLock testnet (100 trades) | 100 | ~10KB | Negligible |
| DagLock mainnet (10K trades/year) | 10,000 | ~1MB | ~1MB/year |
| Ecosystem-wide (1M trades/year) | 1,000,000 | ~100MB | ~100MB/year |

Kaspa's UTXO set is currently ~2-3GB+ for mainnet. Reputation UTXOs would be a small fraction.

## Formula (Same as Current Indexer)

```
Beta raw = (settled + 1) / (trades + 2)
Centered = (Beta raw - 0.5) × 2
Volume bonus = ln(volume_kas / 1000 + 1) × 0.12
Age bonus = min(age_days / 365, 2) × 0.05
Score = 1 + (centered × 4) + volume_bonus + age_bonus
Clamped to [1.0, 5.0]
```

Recent trades (90 days) weighted 2x vs older trades.

## Similar Projects (Reference)

| Project | Chain | Model | Notes |
|---------|-------|-------|-------|
| ENS | Ethereum | NFT-based name registry | Different purpose but shows standards work |
| Lens Protocol | Polygon | Social graph on-chain | Proves on-chain reputation has value |
| EAS (Ethereum Attestation Service) | Ethereum | Schema-based attestations | Closest parallel — signed attestations stored on-chain |
