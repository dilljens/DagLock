# DagLock — Technical Handoff

> Everything you need to know to explain DagLock to anyone: the big picture, the architecture, the trust model, and where all the gaps are.
>
> If someone asks "how does DagLock work?", start at section 1. If they ask about a specific component, jump to that section.

---

## 1. DagLock in One Sentence

**DagLock is a trustless escrow protocol for Kaspa.** Two people who don't trust each other can lock KAS or KRC-20 tokens into a UTXO governed by a covenant. The covenant enforces the terms — both must agree, or timeout refunds the depositor, or a hash preimage reveals for atomic swaps. No admin keys, no intermediaries, no custody.

The indexer, web UI, Telegram bot, and CLI are just convenience layers. The covenant is the only thing that controls funds.

---

## 2. The Mental Model: On-Chain vs Off-Chain

This is the single most important distinction to understand.

### On-Chain (Trustless)

| What | Where | How |
|------|-------|-----|
| Escrow funds | Kaspa UTXO | Locked by SilverScript covenant |
| Release / refund / swap | Kaspa transaction | Covenant validates the spending tx |
| Treasury fee (0.5%) | Kaspa output | Hardcoded in covenant, enforced by consensus |
| Template hash | Kaspa script hash | BLAKE2b-160 fingerprint for UTXO detection |
| KRC-20 token ownership | KCC-20 covenant branch | ICC pattern, DagLock covenant authorizes transfers |

**The covenant is immutable.** Once deployed, nobody — not you, not a hacker, not the Kaspa devs — can change its rules. The rules are the code.

### Off-Chain (Centralized / Indexer-Managed)

| What | Where | How |
|------|-------|-----|
| Offers (discovery board) | Indexer SQLite | Just metadata — "I want to trade X for Y" |
| Reputation scores | Indexer SQLite | Derived from on-chain trade history |
| Encrypted messages | Indexer SQLite | AES-256-GCM per-escrow threads |
| Jury cases | Indexer SQLite | Juror selection, voting, outcomes |
| Settlement receipts | Indexer SQLite | BLAKE2b-hashed verification data |
| Telegram identity links | Indexer SQLite | "This address is @dillon on Telegram" |
| Vouching / Web of Trust | Indexer SQLite | "I vouch for this address" |

**The indexer is replaceable.** If it goes down, funds are safe. If it returns bad data, users can verify against the covenant on-chain. If someone runs a competing indexer, users can switch.

---

## 3. The Covenant Layer (The Core Innovation)

### Three SilverScript Contracts

**`daglock.sil`** — The standard escrow. 60 lines. Three spending paths:

- **Release** — Both buyer and seller sign. Money splits: deposit minus 0.5% to recipient, 0.5% to treasury. The recipient address in output 0 is **not** checked by the covenant — only the amount is. This is by design: the recipient doesn't need to sign, so whoever constructs the tx controls the destination. The other party must verify before signing.
- **Swap** — Someone reveals a secret `S` where `SHA-256(S) == tradeHash`. Same amount split. Used for atomic swaps (hash time-locked contracts). The `tradeHash` parameter comes from the escrow creator.
- **Refund / refundAfterTimeout** — After the deadline, depositor reclaims full amount. No fee. On the arbiter variant, the mediator must co-sign to prevent dispute bypass.

**`daglock_arbiter.sil`** — Same as daglock.sil plus two dispute paths:
- **disputeSellerWins** — Mediator + seller sign. Seller gets deposit minus 0.5% minus mediator fee.
- **disputeBuyerWins** — Mediator + buyer sign. Buyer gets refund minus fees.

**Key difference from daglock.sil:** The refund path requires the mediator to co-sign (refundAfterTimeout). The buyer cannot take funds alone after timeout — the mediator must verify first. No fee on the refund path. If arbiterKey is all zeros, dispute paths are unreachable and refund is also unreachable. Use daglock.sil for trustless timeout refunds.

If `arbiterKey` is all zeros, dispute paths are unreachable. This is how standard escrows disable them.

**`daglock_vault.sil`** — Time-locked self-custody. One path: owner signs after timeout. No fee.

### How a Covenant Actually Works (in Kaspa Terms)

1. Someone calls the compile API (or runs the SilverScript compiler locally) with parameters like `buyerKey`, `sellerKey`, `timeout`, `treasuryKey`
2. The compiler produces bytecode and a **template hash** (BLAKE2b-160 fingerprint)
3. The escrow creator funds a P2SH output with that script hash — this creates a UTXO locked by the covenant
4. To spend it, a transaction must satisfy one of the entrypoint functions
5. The indexer detects DagLock UTXOs by matching template hashes against new UTXOs on the Kaspa ledger (when wRPC listener is implemented)

### The Fee

`feeAmount = inputValue / 200` — hardcoded 0.5%. Integer division. Goes to `treasuryPubKey` (public key you control) in output 1. Output 0 is the settlement amount. The covenant checks both output values and the treasury's script pubkey.

The refund path has **no fee** — the depositor gets everything back.

---

## 4. The Indexer Layer (The REST API)

### What It Does

The indexer is a Rust/Axum daemon that:

1. Serves a REST API (30+ endpoints)
2. Stores data in SQLite (or PostgreSQL)
3. Optionally connects to a Kaspa node via wRPC to auto-detect on-chain UTXOs **(stub — not yet implemented)**
4. Verifies Schnorr signatures to authenticate API calls
5. Encrypts escrow messages with AES-256-GCM
6. Manages jury selection and voting
7. Broadcasts WebSocket events

### Authentication

The API uses **Schnorr signature verification** (BIP-340 style). Users prove wallet ownership by signing a message:

```
sign("settle:esc_abc123")  → 64-byte hex signature in X-Daglock-Signature header
```

KasWare wallet has a "Sign Message" feature for this. The flow:

```
Header: X-Daglock-Address: kaspa:qp0l70zd5x85ttwd6jv7g3s3a8llzj96d8dncn4zmhv4tlzx5k2jyqh70xmfj
Header: X-Daglock-Signature: <64-byte hex>
Header: X-Daglock-Message: settle:esc_abc123
```

The verification flow:
1. Parse the bech32m address → extract the 32-byte x-only public key
2. Hash the message using Kaspa's `PersonalMessageSigningHash` (SHA-256d with "Kaspa Personal Message" prefix)
3. Verify the 64-byte Schnorr signature against the pubkey + hash

**Mock mode is default** (`--mock-auth`). Any hex string passes. This panics at startup if combined with `--network mainnet`. For mainnet, must use `--mock-auth false`.

### Key API Endpoints

| Endpoint | Purpose | Auth? |
|----------|---------|-------|
| `POST /v1/escrows` | Create escrow (record) | No (just metadata) |
| `POST /v1/escrows/:id/settle` | Mark settled | Yes (buyer or seller) |
| `POST /v1/escrows/:id/refund` | Mark refunded | Yes (buyer only) |
| `POST /v1/escrows/:id/dispute` | Dispute | No |
| `POST /v1/escrows/:id/cancel` | Cancel (pre-funding) | Yes (creator) |
| `POST /v1/escrows/:id/swap` | Atomic swap settle | Yes (preimage verification) |
| `GET /v1/offers` | Browse offers | No |
| `POST /v1/offers` | Create offer | No |
| `POST /v1/offers/:id/accept` | Accept offer | No |
| `GET /v1/reputation/:address` | Check reputation | No |
| `POST /v1/compile` | Compile covenant | No |
| `POST /v1/identity` | Link Telegram | Yes (prove wallet ownership) |
| `POST /v1/escrows/:id/messages` | Send encrypted msg | Yes (party or juror) |
| `POST /v1/jury/register` | Become a juror | Yes |
| `POST /v1/jury/cases/:id/vote` | Cast jury vote | Yes |

### On-Chain Verification (Verified at Settlement)

The indexer does NOT scan blocks for UTXO detection. Instead, verification happens at settlement time:
when a user calls `POST /v1/escrows/:id/settle`, the `WrpcVerifier` checks the UTXO exists on-chain
via wRPC. This is the Kaspa-native pattern — the user initiates every action.

**What's needed:** Connect to a Kaspa node via wRPC, subscribe to new UTXOs, match against known template hashes, update escrow state automatically. Without this, users must manually tell the indexer about on-chain events.

---

## 5. The Frontend Surfaces

### Web UI (`web/`)
- React + Vite + TypeScript
- Single-page app with Action panels (create offer, create escrow, dispute, etc.)
- Detects KasWare browser extension for signing
- Dark theme, responsive
- All API calls go to `VITE_API_URL` (env var)

### Telegram Bot (`bot/`)
- Node.js + grammY
- Commands: `/create`, `/claim`, `/offers`, `/reputation`, `/receipt`, `/dispute`, `/msg`, `/list`
- Deep links: `t.me/DagLock_bot?start=claim_esc_abc123`
- Calls the same REST API as the web UI

### CLI (`cli/`)
- Rust + clap
- Subcommands: create, claim, status, offer, reputation, receipt, message
- `--api-url` to point at any indexer
- Same REST API

---

## 6. The Data Model

### Escrow Lifecycle

```
CREATE → PENDING_CONFIRMATION → ACTIVE → SETTLED (or REFUNDED or DISPUTED)
                                        → CANCELLED (pre-funding)
                                        → EXPIRED

DISPUTED → SETTLED (seller wins)
         → REFUNDED (buyer wins)
```

### Offer Lifecycle

```
PROPOSED → ACCEPTED → (escrow created, offer deprecated)
         → CANCELLED
         → EXPIRED
```

### Reputation

Uses Beta reputation formula: `(successes + 1) / (total + 2)`. Scaled to 1-5. Recency-weighted (last 90 days counts 2x). Includes wash trading signal (trading concentration with a single counterparty).

### Jury

Random selection of N jurors from registered pool. Score-weighted (more reliable jurors more likely selected). Threshold-based voting. Juror count scales with escrow amount.

---

## 7. KRC-20 Token Escrows (The Complex Part)

KRC-20 tokens use a different covenant (`daglock_krc20.sil`) that follows the **Inter-Covenant Communication (ICC)** pattern:

1. A KRC-20 token balance is owned by a covenant ID, not a user address
2. The KCC-20 branch stores: `ownerIdentifier` (32 bytes), `identifierType` (0x02 = COVENANT_ID), `amount`
3. DagLockKRC20 owns the KCC-20 branch via its own covenant ID
4. To transfer tokens, the KCC-20 covenant checks that DagLockKRC20 authorized the transfer
5. DagLockKRC20 enforces the escrow conditions (signatures, timeout) before authorizing

This is the most technically complex part of the project. It depends on SilverScript features that may not be fully available yet on testnet.

---

## 8. Deployment Architecture

```
                    
                      Cloudflare DNS          
                      daglock.com             
                      test.daglock.com        
                    

Mainnet:
  daglock.com (Cloudflare Pages) → api.daglock.com (Railway)
    SPA serves static files          Indexer + SQLite volume
    VITE_API_URL=api.daglock.com     --network mainnet --allow-mainnet

Testnet:
  test.daglock.com (Cloudflare Pages) → test-api.daglock.com (Railway)
    Same build, different env var       Same Docker image, different flags
    VITE_API_URL=test-api.daglock.com   --network testnet-12
```

Current state (June 5):
- Indexer runs at `daglock-production.up.railway.app` on `testnet-12` (offline mode)
- Web UI at `daglock.com` → `VITE_API_URL=https://daglock-production.up.railway.app`
- No mainnet indexer deployed yet (waiting for Toccata ~June 30)

---

## 9. Trust Model (What to Trust and What Not To)

### Trusted (Cryptographic Guarantees)

1. **Kaspa consensus** — the BlockDAG is secure proof-of-work
2. **SilverScript compiler** — correctly compiles covenants to bytecode
3. **Kaspa script VM** — correctly executes covenant rules
4. **secp256k1 / Schnorr** — signatures are unforgeable
5. **Covenant immutability** — once deployed, rules cannot change

### NOT Trusted (Centralized / Verify Yourself)

1. **Indexer data** — could be wrong or malicious. The covenant is the source of truth.
2. **Web UI** — could be phished. Verify the URL and what you're signing.
3. **Telegram bot** — same as web UI. Verify before signing.
4. **Reputation scores** — indexer-derived, not on-chain. Could be manipulated.

### The One Thing You Must Always Do

**Verify the transaction before signing it.** The covenant doesn't check who output 0 goes to. If you sign a release tx without checking the recipient address, the person who constructed the tx can send funds to themselves. This is the biggest practical risk.

---

## 10. Current Status

| Layer | Status | Notes |
|-------|--------|-------|
| Covenants (KAS, KRC-20, Arbiter, Vault) |  Done | Compiled, tested, audited |
| Indexer API |  Done | 30+ endpoints |
| Web UI |  Done | Full dashboard |
| Telegram Bot |  Done | All commands |
| CLI |  Done | All commands |
| wRPC Listener |  Stub | Not connected to Kaspa node |
| Auth (Schnorr verification) |  Done | Mock by default, real available |
| Testnet Deployment |  Done | Railway + CF Pages live |
| Mainnet Deployment |  June 30 | Waiting for Toccata hard fork |

### Deployed Features (testnet)

- [x] Create/manage escrows (KAS)
- [x] Offer board — counterparty discovery
- [x] Encrypted messaging per-escrow
- [x] Reputation scores
- [x] Settlement receipts
- [x] Jury dispute resolution
- [x] Vouching / Web of Trust
- [x] Telegram identity linking
- [x] Evidence logging
- [x] Covenant compilation via API
- [x] Atomic swap (hash preimage) endpoint
- [x] Vault covenant (time-locked self-custody)

### Not Yet Built

- [ ] **wRPC listener** — block scanning for auto-detection (deferred — verification at settlement time is sufficient)
- [ ] **Atomic swap wizard UI** — the swap entrypoint works, but there's no guided UI
- [ ] **Volume-based fee rebates** — deferred until a whale asks for a discount
- [ ] **Price oracle** — CoinGecko KAS/USD at escrow creation
- [ ] **Batch escrow UI** — multi-UTXO deals for whales
- [ ] **Analytics dashboard** — public stats

---

## 11. Key Numbers

| Quantity | Value |
|----------|-------|
| Fee | 0.5% (1/200), hardcoded |
| Auth signature size | 64 bytes (Schnorr) |
| Template hash size | 20 bytes (BLAKE2b-160) |
| Pubkey size | 32 bytes (x-only Schnorr) |
| Max escrow amount | 1M KAS (API-enforced) |
| Tests | 102+ across all crates |
| REST endpoints | 30+ |

---

## 12. Talking Points for Different Audiences

### For a Kaspa community member

> "DagLock lets you escrow KAS or KRC-20 tokens without trusting anyone. You lock funds into a covenant on Kaspa L1. The covenant enforces the rules — both parties agree, or you get a refund after timeout. No admin keys, no middleman. There's a Telegram bot and web UI. Testnet is live now at test.daglock.com. Mainnet launches with Toccata."

### For a developer

> "Three SilverScript covenants: daglock (standard escrow), daglock_arbiter (dispute resolution), daglock_vault (time-locked savings). They're compiled to Kaspa script bytecode and deployed as P2SH UTXOs. The indexer is a Rust/Axum API with SQLite, Schnorr auth, AES-GCM message encryption. Web UI is React/Vite. Bot is Node.js/grammY. The wRPC listener that connects to Kaspa nodes for auto-detection is the main unfinished piece."

### For an investor / non-technical

> "Trustless escrow for Kaspa. Think of it as a smart contract for P2P crypto trades — but it's not a contract, it's a covenant built into the UTXO itself. The team takes 0.5% per trade. The code is open source. Mainnet launch targeted for June 30 when Kaspa's Toccata hard fork enables covenants."

### For someone evaluating security

> "The covenant is 60 lines of SilverScript, which compiles to Kaspa script bytecode. It has three spending paths, each tested with negative test cases. The fee is hardcoded — can't be changed after deployment. No admin keys. No upgrade mechanism. The indexer is convenience only — it can't touch funds. The biggest risk is users signing transactions without verifying what they're signing, which is true of every crypto protocol."

---

## 13. Common Confusions (Avoid These)

**"The indexer holds the funds."** — No. The covenant on Kaspa holds the funds. The indexer just tracks metadata. It doesn't have keys or control.

**"The fee can be changed."** — No. `inputValue / 200` is compiled into the covenant bytecode. Once deployed, it's frozen.

**"DagLock is a smart contract platform."** — No. It's one protocol using one set of covenants on Kaspa. Not a platform.

**"KYC is required."** — No. No signup, no accounts, no personal data. Just wallet signatures.

**"The treasury key means you can steal funds."** — No. The treasury only receives the 0.5% fee output. It cannot spend the escrow UTXO. Only the buyer, seller, or swap preimage can do that.

**"Atomic swaps require a wizard UI."** — The covenant supports them. The on-chain mechanism works. The UI just doesn't have a pretty guided flow yet.

**"It's not on mainnet so it doesn't work."** — It works on Testnet 12 right now. Mainnet requires Toccata which activates ~June 30.
