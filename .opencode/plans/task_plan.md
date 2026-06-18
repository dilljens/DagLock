# On-Chain Reputation Covenant

**Status:** Design phase (not started)
**Started:** 2026-06-18
**Target:** Testnet deploy before mainnet (June 30)

---

## Overview

A standalone SilverScript covenant that records trade outcomes on-chain. Independent of DagLock escrow covenants — any Kaspa user or dApp can submit signed trade receipts. Reputation data becomes portable across the entire Kaspa ecosystem.

---

## Phase 1: Covenant Design & Implementation

- [x] 1.1 — Write `contracts/src/daglock_reputation.sil` (~50 lines)
  - Entrypoints: `recordTrade` (signed receipt → UTXO)
  - Per-trade UTXOs, both-party Schnorr signature verification
- [x] 1.2 — Write execution tests (4 tests passing, 215 total)
  - recordTrade with both sigs succeeds, different keys differ, hash deterministic
- [x] 1.4 — Document covenant ABI and template hash
  - Template hash: `65c54102c64a331414b602760cbd76efac3d69df`
  - Added to indexer config, wiki docs, AGENTS.md
- [ ] 1.3 — Deploy to testnet (requires synced kaspad node — deferred until mainnet)

## Phase 2: Indexer Integration

- [x] 2.1 — Add reputation covenant template hash to indexer config
- [ ] 2.2 — Auto-submit receipts after escrow settlement (indexer signs using treasury key)
- [ ] 2.3 — Add `/v1/reputation/on-chain/:address` endpoint
- [ ] 2.4 — Update existing `/v1/reputation/:address` to include `on_chain: true`
- [ ] 2.5 — Backfill script (`scripts/reputation-submitter.py`)

## Phase 3: Client Library & Standards (Week 3)

- [ ] 3.1 — TypeScript client library (`@daglock/reputation`) — read UTXOs, compute Beta score
- [ ] 3.2 — KIP draft describing:
  - Receipt JSON format (signed data structure)
  - Covenant entrypoints
  - Address identification (BLAKE2b-160 of pubkey)
  - Beta reputation formula
- [ ] 3.3 — Python library (`daglock-reputation`) for non-JS clients
- [ ] 3.4 — Documentation: "How to Integrate Reputation" on DocsPage

## Phase 4: Community & Adoption (Ongoing)

- [ ] 4.1 — Share KIP draft with Kaspa builder community
- [ ] 4.2 — Reach out to 1+ other Kaspa dApp projects for adoption
- [ ] 4.3 — Onboard at least one external project before mainnet
- [ ] 4.4 — Deploy covenant to mainnet on June 30 alongside DagLock

---

## Design Decisions (Locked)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| UTXO model | Per-trade UTXOs | Simplest covenant, no merging logic, Kaspa UTXO enumeration works well |
| Signature scheme | Both parties sign via their wallets | Most trustless. Indexer never handles private keys. |
| Fee | Network gas only (~1 cent) | Keeps it free and permissionless. Prevents spam via signature requirement. |
| Indexer DB | Dual — covenant when available, DB fallback | No breakage for existing users. Graceful migration. |
| Backfill | Script (`scripts/reputation-submitter.py`) | One-time catch-up for existing trades before mainnet. |
| Formula | Beta (Josang 2002) + recency weighting | Same formula users already see on the Reputation page. Consistent. |
| Template hash | Documented like other covenants | Added to config, same pattern as KAS/KRC-20 templates. |

---

## Risks

| Risk | Mitigation |
|------|-----------|
| Covenant state grows too large (many UTXOs) | Per-trade UTXOs are small (~100 bytes each). UTXO set pruning on Kaspa handles old spent outputs. 1M trades ≈ 100MB. |
| No one adopts the standard | Start with DagLock auto-submitting (our users get value immediately). External adoption is bonus. |
| Signature verification too expensive | Schnorr batch verification exists. One signature verification per trade is <$0.01. |
| Replay attacks | Nonce field in receipt + covenant tracks used nonces. |
