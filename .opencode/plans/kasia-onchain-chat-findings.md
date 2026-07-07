# Findings: Kasia On-Chain Chat

## Requirements Discovery

**Goal:** Replace DagLock's server-side AES-256-GCM encrypted messaging with an E2E on-chain chat protocol (Kasia-compatible). Message ciphertext stored off-chain, hashes anchored on Kaspa transactions. Dedicated chat key.

**Independent workstreams:** 6 tracks (A-F) — E2E encryption, chat key separation, on-chain anchoring, dispute reveal, web UI, bot commands.

**Key constraints:**
- Hybrid approach: ciphertext off-chain, hashes on-chain (not every message = a tx)
- Text only at launch (no photos/videos)
- Web dashboard + Telegram bot surfaces
- Chat key must be separate from funding key (cannot move funds)
- Server must never hold decryption keys

## Architecture Research

### Current Messaging Architecture

| Component | Current | Target |
|-----------|---------|--------|
| Encryption | Server-side AES-256-GCM (key in env var) | Client-side E2E (server relays ciphertext) |
| Key | Single server key for all escrows | Per-escrow Ed25519 keypair, X25519 ECDH shared secret |
| On-chain | None | blake2b(ciphertext) → Kaspa tx payload |
| Chat key | None (uses funding key for auth) | Dedicated Ed25519 keypair, cannot spend funds |
| Dispute reveal | Server decrypts for authorized users | Party reveals chat private key to jury |
| Recovery | None | Chat key on recovery sheet (.txt) |
| Media | Not supported | Deferred |
| Surface | Web API + web UI | Web + Telegram bot (web-only for reading) |

### Kaspa Transaction Payload

Kaspa transactions support a `payload` field (up to ~25 KB). This is the mechanism for anchoring message hashes:
- No OP_RETURN needed — Kaspa has first-class payload support
- Cost: ~0.00001 KAS per anchor tx (sending 1000 sompi to self)
- Payload is visible on any Kaspa explorer
- Payload format: `prefix(4) + merkle_root(32) + escrow_id(16) + count(4)` = 56 bytes

### Chat Key Design

- Escrow/Ed25519 for chat (not Kaspa's Schnorr) — Ed25519 is standard for messaging
- Chat key signs `sha256(ciphertext || seq)` — proves authorship
- Covenant only accepts Schnorr signatures (secp256k1) — chat key physically cannot spend UTXOs
- Chat pubkey stored on escrow record; private key only in browser + recovery sheet

### Encryption Flow

```
Party A (browser)                     Server                    Party B (browser)
     │                                  │                           │
     │  POST chat_pubkey                │                           │
     │ ───────────────────────────────► │                           │
     │                                  │  store chat_pubkey_A      │
     │                                  │  (wait for B's pubkey)    │
     │                                  │                           │
     │                                  │  return chat_pubkey_B     │
     │  ◄─────────────────────────────  │  (to A when available)    │
     │                                  │                           │
     │  derive shared_secret via ECDH   │                           │
     │  (privkey_A × pubkey_B)         │                           │
     │                                  │                           │
     │  encrypt message with AES-256-   │                           │
     │  GCM using shared_secret         │                           │
     │                                  │                           │
     │  POST ciphertext + nonce         │                           │
     │ ───────────────────────────────► │                           │
     │                                  │  store ciphertext         │
     │                                  │  (cannot decrypt)         │
     │                                  │  compute hash =           │
     │                                  │  blake2b(ciphertext)      │
     │                                  │  add to anchor batch      │
     │                                  │                           │
     │                                  │  relay ciphertext         │
     │                                  │ ────────────────────────► │
     │                                  │  (or party B polls)       │
     │                                  │                           │
     │                                  │  anchor batch every 5 min │
     │                                  │  → send Kaspa tx with     │
     │                                  │  merkle_root in payload   │
```

### Security Model

| Threat | Mitigation |
|--------|------------|
| Server modifies messages | Hash anchoring proves original content; E2E prevents decryption |
| Server forges messages | Chat key signatures verify authorship; E2E prevents reading |
| Third party reads messages | E2E encryption; server never has key |
| Party denies sending message | Ed25519 signature proves authorship |
| Party forges screenshots | On-chain hash proves message existed at given DAA time |
| Key compromise (chat) | Chat key cannot spend funds — only read/author messages |
| Key compromise (funding) | Cannot read messages — uses different key type |

### On-Chain Anchor Payload Format

```
Offset  Size  Field
0       4     Magic prefix: 0x444C4148 ("DLAH" = DagLock Anchor Hash)
4       32    blake2b(Merkle root of batch message hashes)
36      16    Escrow ID (first 16 chars of UUID, ASCII)
52      4     Message count in batch (u32 LE)
56            Total: 56 bytes
```

## Open Questions → Resolved

- **Q:** Should we use X25519 or secp256k1 for ECDH?
  → **A:** X25519 — simpler, widely supported, not tied to Kaspa's curve. Chat keys are Ed25519, so X25519 ECDH is natural with the same keypair (Ed25519 keys can be used for X25519 ECDH via the X25519 transform).

- **Q:** Per-message tx or batch anchoring?
  → **A:** Batch every 5 min or every 10 messages. ~100x cheaper, same tamper-proof guarantee via Merkle root.

- **Q:** Should bot support decryption?
  → **A:** No — bot never holds chat key. Users click through to web to read/reply. Optional opt-in later.

- **Q:** Migration from old messages?
  → **A:** Old messages stay AES-encrypted in DB. New escrows use E2E. No migration needed — users can view old messages via existing server-decrypt path.
