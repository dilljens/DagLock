# Glossary

Project-specific terms and acronyms.

| Term | Definition | Context |
|------|------------|---------|
| **DagLock** | Trustless escrow & atomic swap protocol on Kaspa L1 via SilverScript covenants | Project name, everywhere |
| **Covenant** | A compiled Kaspa script that enforces spending conditions on UTXOs | `contracts/src/daglock.sil` |
| **SilverScript** | Language for writing covenants compiled to Kaspa script bytecode | `contracts/src/*.sil` |
| **Arbiter covenant** | DagLock variant with optional mediator/jury dispute paths | `contracts/src/daglock_arbiter.sil` |
| **KRC-20** | Kaspa token standard (like ERC-20 but UTXO-based) | `daglock_krc20.sil`, token escrows |
| **KCC-20** | Kaspa Covenant Contract standard for KRC-20 token ownership | `daglock_krc20.sil` ICC pattern |
| **ICC** | Inter-Covenant Communication | KRC-20 escrow design |
| **UTXO** | Unspent Transaction Output | Every escrow = one UTXO |
| **Template hash** | BLAKE2b-160 hash of covenant prefix+suffix | `contracts/src/lib.rs` |
| **wRPC** | Kaspa's WebSocket RPC protocol | `indexer/src/listener.rs`, `indexer/src/verification.rs` |
| **WrpcVerifier** | On-chain UTXO verification via wRPC (used at settlement time) | `indexer/src/verification.rs` |
| **MockVerifier** | Dev-mode verifier that always succeeds (no wRPC needed) | `indexer/src/verification.rs` |
| **DAA score** | Difficulty-Adjusted Average score (Kaspa block height) | Expiration logic |
| **sompi** | Smallest unit of KAS (10^-8) | Amounts in API types |
| **Treasury** | DagLock fee address -- receives 0.5% on settlement | `treasuryKey` in covenant |
| **Beta reputation** | Academic standard (Josang 2002): (successes+1)/(total+2) | `indexer/src/db/queries/reputation.rs` |
| **Recency weighting** | Last 90 days weighted 2x in reputation formula | `calculate_reputation_score()` |
| **Wash trading signal** | trading_concentration: fraction of volume with single counterparty | Reputation response |
| **Vouching** | Web of Trust: vouch for an address's reliability | `indexer/src/api/vouches.rs` |
| **Jury** | Community dispute resolution via randomly selected jurors | `indexer/src/api/jury.rs` |
| **Evidence** | Signed proof submitted during a dispute | `indexer/src/api/evidence.rs` |
| **Dispute mode** | How an escrow handles disputes: standard/mediator/jury | `indexer/src/types.rs`, create escrow form |
| **Escrow messaging** | Encrypted chat thread tied to an escrow (AES-256-GCM) | `indexer/src/api/messages.rs` |
| **Wash trading** | trading_concentration metric: fraction of volume with a single counterparty | Reputation response, `queries/reputation.rs` |
| **Mediator** | Optional third-party dispute resolver (single person) | `contracts/src/daglock_arbiter.sil` |
| **Toccata** | Kaspa mainnet hard fork (June 5-20, 2026) | `docs/ROADMAP.md` |
| **KasWare** | Kaspa web extension wallet | Browser signing integration |
| **Kaspium** | Kaspa mobile wallet | Mobile signing integration |
