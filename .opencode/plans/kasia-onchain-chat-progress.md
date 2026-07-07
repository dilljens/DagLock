# Progress: Kasia On-Chain Chat

## Session 2026-07-06

**Status:** Plan created. Ready for execution.

### Current State
- [x] Web research complete — OfficeForge Kasia protocol analyzed
- [x] DagLock's current messaging system audited (crypto.rs, messages API, queries, types, schema)
- [x] Kaspa protocol constraints researched (payload field, tx size limits, fees, wallet API)
- [x] Scope confirmed with user: hybrid anchoring, text-only, web + bot
- [x] `task_plan.md` created with 6 tracks, phases, checkpoints, fallbacks
- [x] `findings.md` created with architecture notes and pre-resolved decisions
- [ ] **Track A: E2E Encryption Core** — not started
- [ ] **Track B: Chat Key Separation** — not started
- [ ] **Track C: On-Chain Hash Anchoring** — not started
- [ ] **Track D: Dispute Reveal Flow** — not started
- [ ] **Track E: Web UI Chat Component** — not started
- [ ] **Track F: Bot Commands** — not started

### Decisions Made
- Encryption: X25519 ECDH + AES-256-GCM per-message (client-side)
- Chat key: Ed25519 keypair, generated in-browser per escrow
- Anchoring: Batch Merkle root every 5 min / 10 msgs → Kaspa tx payload
- Payload format: 56 bytes (prefix + merkle_root + escrow_id + count)
- Bot: No decryption support (click-through to web)
- Migration: Old messages stay AES-encrypted; new ones use E2E

### Next Action
Begin Track A Phase A1 (Key Exchange Protocol) when execution starts.
