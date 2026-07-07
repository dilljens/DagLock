# Plan: On-Chain Kasia Chat for DagLock

> **Goal:** Replace DagLock's server-side AES-256-GCM encrypted messaging with an E2E on-chain chat protocol (Kasia-compatible). Message ciphertext stored off-chain, hashes anchored on Kaspa. Dedicated chat key that cannot move funds. Dispute reveal flow for jury.
>
> **Status:** Plan created — pre-resolved decisions documented. Awaiting execution.

---

## Requirements

- [ ] **R1** E2E encryption — per-escrow ECDH key exchange, server relays ciphertext only
- [ ] **R2** Dedicated chat keypair — generated per deal, CANNOT move funds (separate key from funding)
- [ ] **R3** On-chain hash anchoring — blake2b(ciphertext) as Kaspa tx payload
- [ ] **R4** Thread integrity — messages ordered by on-chain DAA score, not server timestamp
- [ ] **R5** Dispute reveal — party can reveal chat private key to jury, decrypted copy wiped post-close
- [ ] **R6** Recovery — chat key included in recovery sheet (.txt download)
- [ ] **R7** Web UI — new chat component with send/receive/disclose UX
- [ ] **R8** Bot commands — Telegram messaging via bot

---

## Pre-resolved Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| **Encryption** | X25519 ECDH + AES-256-GCM per-message nonce | Proven, post-quantum-optional, Kaspa's secp256k1 can also work — X25519 is simpler |
| **On-chain anchoring** | Hybrid — batch hash jobs every 5 min, not per-message tx | Reduces cost ~100x vs per-message txs; still tamper-proof via DAA ordering |
| **Chat key** | Ed25519 keypair generated in-browser per escrow | Ed25519 is widely supported, cheap to generate, separate from KAS Schnorr keys |
| **Chat key storage** | On recovery sheet (.txt), NOT in indexer DB | Zero-knowledge server; follows OfficeForge model |
| **Message storage** | Encrypted ciphertext in existing `escrow_messages` table (new E2E, not server AES) | Reuses existing infrastructure, minimal migration |
| **Hash anchoring tx** | Simple KAS send-to-self with `payload` field using KasWare/Kaspium | ~0.00001 KAS per batch; no covenant needed |
| **Payload format** | `Prefix(4) + blake2b(ciphertext_concat, 32) + escrow_id(16) + seq(4)` = 56 bytes | Compact, self-describing, enough for verification |
| **Batch interval** | Every 5 minutes OR every 10 messages, whichever comes first | Balances tx cost vs security |
| **Dispute reveal** | Party clicks "Reveal chat to jury" → chat private key encrypted with jury's public key | Jury can decrypt; original parties still hold the key |
| **Post-close wipe** | Server deletes ciphertext after close; decryption capability already lost | Chat key never on server, so "wipe" = delete ciphertext |
| **Photo/video** | Deferred — text only at launch | Constraint from user |
| **Surfaces** | Web dashboard + Telegram bot | Constraint from user |

---

## Track A: E2E Encryption Core `[ ]`

**Description:** Replace `crypto.rs` AES-256-GCM server-side encryption with per-escrow X25519 ECDH key exchange. Generate shared secret per escrow, encrypt messages client-side.

**Timebox:** 1-2 weeks

### Phase A1: Key Exchange Protocol `[ ]` [3-4 days]
- [ ] Design key exchange flow: 
  - Escrow creation → each party generates Ed25519 chat keypair in-browser
  - Chat pubkey submitted to `POST /v1/escrows` as optional field
  - Indexer stores `chat_pubkey_buyer` and `chat_pubkey_seller` on escrow row
  - When both pubkeys are present → server derives shared secret (or parties do it client-side)
- [ ] Key exchange: ECDH using X25519 (or secp256k1 to match Kaspa's curve — TBD)
  - If secp256k1: `shared_secret = SHA256(privkey_a * pubkey_b || escrow_id)` — ties to specific escrow
- [ ] WASM SDK: add function to generate Ed25519 chat keypair + compute shared secret
- [ ] Web: integrate key generation into escrow creation form
- [ ] Bot: generate chat keypair when creating escrow
- ✅ **Checkpoint:** Two browser instances can derive the same shared secret for an escrow
- ⚙ **Fallback:** Server computes shared secret (weaker but simpler); parties send raw pubkeys

### Phase A2: Client-Side Encryption `[ ]` [3-5 days]
- [ ] WASM/JS: encrypt message with shared secret + random nonce → AES-256-GCM ciphertext
- [ ] WASM/JS: compute `blake2b(ciphertext, 32)` for on-chain anchoring
- [ ] Web UI chat input uses client-side encryption before POST to `/v1/messages`
- [ ] Update `SendMessageRequest`: accept `content_enc` + `nonce` instead of plaintext `content`
- [ ] Remove server-side `crypto::encrypt_message()` call from messages API
- [ ] Server stores ciphertext as-is, never has decryption key
- [ ] Server-side `crypto::decrypt_message()` no longer usable — only client decrypts
- ✅ **Checkpoint:** Message ciphertext in DB cannot be decrypted by server (key never stored)
- ⚙ **Fallback:** Client-side encrypt but also send plaintext for dev mode (`--mock-encryption`)

### Phase A3: Client-Side Decryption `[ ]` [2-3 days]
- [ ] `GET /v1/messages` returns ciphertext + nonce (as before)
- [ ] Web UI decrypts each message client-side using shared secret
- [ ] Failed decryption shows `[encrypted — reveal to view]` (dispute flow covers this)
- [ ] Bot: messages shown as `[encrypted — use web dashboard to read]` (bot can't decrypt)
- [ ] Indexer: no decryption logic needed anymore — just stores + relays ciphertext
- ✅ **Checkpoint:** Messages visible in web UI, invisible on server, invisible in bot
- ⚙ **Fallback:** Bot stores decryption key for convenience (user opts in, like OfficeForge's recovery)

---

## Track B: Chat Key Separation `[ ]`

**Description:** Generate per-escrow Ed25519 keypair in-browser. Chat key signs messages but cannot spend covenant funds. Stored on recovery sheet only (not server).

**Timebox:** 1 week

### Phase B1: Chat Key Generation `[ ]` [2-3 days]
- [ ] WASM SDK: `generateChatKeypair()` → `{ pubkey: Uint8Array, privkey: Uint8Array }`
- [ ] Web: generate on escrow creation + on joining
- [ ] Chat pubkey submitted to server on create/join
- [ ] On submission: sign `chat:{escrow_id}:{pubkey}` with funding key to prove ownership
- [ ] Server stores `chat_pubkey` on escrow record (not private key)
- ✅ **Checkpoint:** Chat pubkey on escrow, chat privkey only in browser/recovery sheet
- ⚙ **Fallback:** Generate server-side, encrypt with user's funding key for storage

### Phase B2: Recovery Sheet `[ ]` [1-2 days]
- [ ] On escrow creation: download `.txt` with chat private key + escrow ID + chat pubkey
- [ ] On join: same recovery sheet download
- [ ] Recovery flow: paste chat key from sheet into "Restore chat" form
- [ ] Server never sees chat private key (except encrypted during dispute reveal)
- ✅ **Checkpoint:** Can lose browser, rejoin from recovery sheet, see full chat history
- ⚙ **Fallback:** Chat key encrypted with funding key, stored in DB as backup

### Phase B3: Chat Key Signature `[ ]` [1-2 days]
- [ ] Each message signed with chat key: `ed25519_sign(chat_privkey, sha256(ciphertext || seq))`
- [ ] Signature included in message POST: `chat_sig` field
- [ ] Server verifies `chat_pubkey` matches `chat_sig` (not funding key auth)
- [ ] Chat key CANNOT spend covenant UTXOs — enforced by key type (Ed25519 ≠ Schnorr) + covenant only accepts Schnorr
- ✅ **Checkpoint:** Message can be attributed to correct party via chat key, not funding key
- ⚙ **Fallback:** Skip chat key signatures; rely on server auth headers

---

## Track C: On-Chain Hash Anchoring `[ ]`

**Description:** Periodically anchor batch of message hashes as Kaspa transactions. Proves message existence at a point in time (DAA score). Prevents forgery/backdating.

**Timebox:** 1-2 weeks

### Phase C1: Batch Anchor Service `[ ]` [3-5 days]
- [ ] New service `indexer/src/services/anchor.rs`:
  - Collects unanchored message hashes every 5 min (or every 10 msgs)
  - Computes Merkle root of batch: `blake2b(hash_1 || hash_2 || ... || hash_N)`
  - Constructs Kaspa tx: send ~1000 sompi to self with `payload = prefix(4) + root(32) + escrow_id(16) + count(4)`
  - Total payload: 56 bytes — well within 25 KB limit
- [ ] Uses `--anchor-wallet-key` (a hot wallet key that can only send dust)
- [ ] If no Kaspa node available: log warning, skip batch (graceful degradation)
- [ ] Records `anchor_tx_id` and `anchor_daa_score` on message batch
- ✅ **Checkpoint:** `explorer.kaspa.org` shows anchor transactions with readable payload
- ⚙ **Fallback:** Skip on-chain anchoring entirely; E2E encryption alone is still better than current

### Phase C2: Anchor Verification `[ ]` [2-3 days]
- [ ] Indexer: fetch anchor tx payload via wRPC / explorer API
- [ ] Verify: decode payload, recompute Merkle root from stored messages, compare
- [ ] If mismatch: flag integrity alert (possible server compromise)
- [ ] Web UI: show "🔗 On-chain anchored at DAA X" on message thread
- [ ] Explorer link to anchor transaction
- ✅ **Checkpoint:** Can verify message integrity against on-chain anchor
- ⚙ **Fallback:** Manual verification page at `daglock.com/verify/:escrow_id`

### Phase C3: DAA-Based Ordering `[ ]` [1-2 days]
- [ ] Each message has `anchor_seq` field (order within anchor batch)
- [ ] Display messages sorted by anchor DAA score + anchor_seq
- [ ] If unanchored: sort by server `created_at` (fallback)
- ✅ **Checkpoint:** Message ordering matches on-chain DAA sequence
- ⚙ **Fallback:** Server timestamp ordering (same as now)

---

## Track D: Dispute Reveal Flow `[ ]`

**Description:** During a dispute, one party can voluntarily reveal the chat private key to the jury. The jury reads the decrypted thread and makes a ruling. After case closes, decrypted copy is wiped.

**Timebox:** 1 week

### Phase D1: Reveal Protocol `[ ]` [2-3 days]
- [ ] `POST /v1/escrows/:id/messages/reveal` — party encrypts chat private key with jury's public key (or a per-case reveal key)
- [ ] Server stores `revealed_chat_key_enc` on jury case (encrypted for jury only)
- [ ] Server decrypts all messages using revealed key, stores decrypted copies in `mediation_evidence` table
- [ ] Decrypted copies accessible only to assigned jurors
- [ ] After case resolution: delete `revealed_chat_key_enc` and all decrypted copies
- ✅ **Checkpoint:** Jury can read messages after reveal; server cannot without jury key
- ⚙ **Fallback:** Party pastes chat key into a text field; server decrypts (weaker — server learns key)

### Phase D2: Jury Chat Reader `[ ]` [2-3 days]
- [ ] Jury case page: "Chat evidence" section, visible only after at least one party reveals
- [ ] Shows decrypted message thread with timestamps
- [ ] Links to on-chain anchor verification
- [ ] Jurors can reference specific messages in their vote
- [ ] Bot: `/case_chat <case_id>` — shows chat evidence (if revealed)
- ✅ **Checkpoint:** Jury sees full decrypted chat with on-chain proof of integrity
- ⚙ **Fallback:** Chat shown as web page; bot not supported

### Phase D3: Post-Close Wipe `[ ]` [1-2 days]
- [ ] On case resolution: delete all decrypted message copies from `mediation_evidence` table
- [ ] Delete `revealed_chat_key_enc` from jury case
- [ ] Re-encrypt messages in `escrow_messages`? No — they were never decrypted server-side; only the revealed copy was
- [ ] Log wipe event for audit
- ✅ **Checkpoint:** Post-resolution, no decrypted chat data exists on server
- ⚙ **Fallback:** Mark as deleted (soft delete), hard delete after 30 days

---

## Track E: Web UI — New Chat Component `[ ]`

**Description:** Replace the current basic message display with a full chat component. Inline decryption, send UI, anchor status, reveal button.

**Timebox:** 1 week

### Phase E1: Chat Component `[ ]` [3-5 days]
- [ ] New component `web/src/components/OnChainChat.tsx`
  - Message list with bubbles (buyer=right, seller=left)
  - Client-side encryption on send
  - Client-side decryption on receive
  - "🔗 Anchored" badge per message
  - Timestamps (DAA score + wall time)
- [ ] Input area: text input + send button
  - On submit: encrypt with shared secret, POST ciphertext, compute hash for anchoring
- [ ] "Reveal to jury" button (active only when escrow is disputed)
  - Shows warning: "This will give the jury access to read your chat history"
  - On click: encrypts chat private key with jury's public key → POST to reveal endpoint
- [ ] Recovery: "Restore chat" option — paste recovery sheet key
- [ ] Empty state: "No messages yet. Messages are E2E encrypted."
- ✅ **Checkpoint:** Send/receive works E2E; anchor badge visible
- ⚙ **Fallback:** Keep current server-side encryption, add client-side as option

### Phase E2: Escrow Creation Integration `[ ]` [1-2 days]
- [ ] Create form: generate chat keypair in background (no user action needed)
- [ ] Display chat pubkey in escrow detail
- [ ] Recovery sheet download includes chat private key
- ✅ **Checkpoint:** Chat keys generated on every new escrow
- ⚙ **Fallback:** Generate on first message instead of on creation

---

## Track F: Bot Commands — Telegram Messaging `[ ]`

**Description:** Add Telegram bot support for sending/receiving chat messages. Bot cannot decrypt messages (key never leaves user's device). User gets notification: "New encrypted message — view on web dashboard."

**Timebox:** 3-5 days

### Phase F1: Bot Message Notifications `[ ]` [1-2 days]
- [ ] New bot notification: "📩 New message from {party} on escrow {id}. View and reply: https://daglock.com/escrows/{id}"
- [ ] Sent on new message insert (same as existing email notification pattern)
- [ ] No decryption in bot (bot never holds chat key)
- [ ] `/messages <escrow_id>` — shows "View your encrypted messages: https://daglock.com/escrows/{id}"

### Phase F2: Bot Deep Links `[ ]` [1-2 days]
- [ ] Deep links from bot open the web chat directly to the escrow
- [ ] `/msg <escrow_id> <text>` — opens web dashboard to send message (bot can't encrypt locally)
- [ ] Alternative: bot stores a per-user decryption key on user's Telegram-linked identity (opt-in)
- ✅ **Checkpoint:** Bot notifies on new message; user clicks through to web to read/reply
- ⚙ **Fallback:** Bot does nothing for messages (just email + web)

---

## Execution Strategy

```
Priority 1 (Foundation — E2E must come first):
  Track A — E2E Encryption Core (1-2 weeks)
  Track B — Chat Key Separation (1 week)         ← parallel with A2

Priority 2 (Infrastructure):
  Track C — On-Chain Hash Anchoring (1-2 weeks)   ← depends on A
  Track E — Web UI Chat Component (1 week)        ← depends on A

Priority 3 (Dispute + Bot):
  Track D — Dispute Reveal Flow (1 week)           ← depends on A+B+E
  Track F — Bot Commands (3-5 days)               ← depends on A+E
```

Tracks within the same priority can run in parallel if they touch different files. Track A must finish first since E2E encryption is the foundation everything else builds on.

---

## Anti-scope (what this plan does NOT include)

- Photo/video/media support (deferred per user decision)
- Full Kasia protocol compatibility (we use our own payload format)
- Custom messaging covenant (no new covenant — just Kaspa tx payloads)
- Mobile native app
- Decentralized storage for media (IPFS/Arweave — deferred)
- Group chat / multi-party chat (only buyer+seller)
- Read receipts
- Typing indicators
- Message reactions / emoji

---

## Files to Create

| Track | New Files |
|-------|-----------|
| A | `wasm-sdk/src/chat.rs` (if WASM needed), or `web/src/crypto/chat-crypto.ts` |
| C | `indexer/src/services/anchor.rs` |
| D | `indexer/src/api/jury_reveal.rs` |
| E | `web/src/components/OnChainChat.tsx` |

## Files to Modify

| Track | Files |
|-------|-------|
| A | `indexer/src/crypto.rs` (deprecate AES), `indexer/src/api/messages.rs`, `indexer/src/types.rs`, `indexer/src/db/queries/messages.rs` |
| B | `indexer/src/types.rs` (chat_pubkey fields), `indexer/src/api/escrows.rs` (accept chat pubkey), `web/src/api.ts` |
| C | `indexer/src/config.rs` (anchor-wallet-key flag), `indexer/src/main.rs` (spawn anchor service), `indexer/src/db/schema.rs` (anchor columns) |
| D | `indexer/src/api/jury.rs`, `indexer/src/db/queries/jury.rs`, `web/src/pages/JuryPage.tsx` |
| E | `web/src/pages/EscrowsPage.tsx` (replace message view), `web/src/api.ts` |
| F | `bot/src/index.js`, `bot/src/lib/api.js` |
