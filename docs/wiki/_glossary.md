# Glossary

Project-specific terms and acronyms.

| Term | Definition | Context |
|------|------------|---------|
| **DagLock** | Trustless escrow & atomic swap protocol on Kaspa L1 via SilverScript covenants | Project name, everywhere |
| **Covenant** | A compiled Kaspa script that enforces spending conditions on UTXOs | `contracts/src/daglock.sil` |
| **SilverScript** | Language for writing covenants compiled to Kaspa script bytecode | `contracts/src/*.sil`, `silverscript-lang` dependency |
| **KIP-17 / KIP-20** | Kaspa Improvement Proposals for UTXO covenants | Protocol spec, `docs/PROTOCOL.md` |
| **KRC-20** | Kaspa token standard (like ERC-20 but UTXO-based) | `daglock_krc20.sil`, token escrows |
| **KCC-20** | Kaspa Covenant Contract standard for KRC-20 token ownership | `daglock_krc20.sil` ICC pattern |
| **ICC** | Inter-Covenant Communication — one covenant owning another via covenant ID | KRC-20 escrow design |
| **UTXO** | Unspent Transaction Output — the fundamental unit of Kaspa balance | Every escrow = one UTXO |
| **P2SH** | Pay-to-Script-Hash — address type for covenant scripts | Template hash detection |
| **Template hash** | BLAKE2b-160 hash of covenant prefix+suffix — identifies any DagLock UTXO regardless of params | `contracts/src/lib.rs` `template_parts_and_hash()` |
| **wRPC** | Kaspa's WebSocket RPC protocol for node communication | `indexer/src/listener.rs` |
| **DAA score** | Difficulty-Adjusted Average score — Kaspa's block height metric | `EscrowStatus`, expiration logic |
| **sompi** | Smallest unit of KAS (10^-8 KAS, like satoshis for BTC) | Amounts in API types |
| **Entrypoint** | A named spending path in a covenant (release, swap, refund) | `contracts/src/daglock.sil` |
| **Trade hash** | SHA-256 hash of atomic-swap secret — zeroed if no swap | Constructor param in `.sil` |
| **Treasury** | DagLock fee address — receives 0.5% on settlement | `treasuryKey` in covenant |
| **Reconciliation** | Background loop that marks expired escrows in DB | `indexer/src/listener.rs` |
| **Offer board** | Public listing of proposed (unfunded) escrow offers | `indexer/src/api/offers.rs` |
| **Reputation** | On-chain derived metrics per address (trade count, volume, age) | `indexer/src/api/reputation.rs` |
| **Receipt** | Cryptographic proof of completed trade with verification data | `indexer/src/api/receipts.rs` |
| **Atomic swap** | Two parties exchange assets using hash preimage reveal | `swap` entrypoint |
| **Toccata** | Kaspa mainnet hard fork (June 5–20, 2026) — enables covenants | `docs/ROADMAP.md` |
| **KasWare** | Kaspa web extension wallet | Browser signing integration |
| **Kaspium** | Kaspa mobile wallet | Mobile signing integration |
