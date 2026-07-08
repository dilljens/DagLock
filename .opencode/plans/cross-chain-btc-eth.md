# Plan: Cross-Chain Atomic Swaps (BTC/ETH)

> **Goal:** Enable trustless KAS↔BTC and KAS↔ETH atomic swaps using HTLC (Hash Time-Locked Contract).
>
> **Status:** Plan created. Research complete. Awaiting execution decision.
>
> **Effort:** Kaspa→Ethereum (~6 weeks) · Kaspa→Bitcoin (~10 weeks) · Both (~12 weeks)

---

## How Cross-Chain Atomic Swaps Work

```
Alice (has KAS, wants BTC)          Bob (has BTC, wants KAS)
        │                                    │
        │ 1. Generates secret S               │
        │    Computes H = SHA-256(S)           │
        │    Sends H to Bob                    │
        │──────────────────────────────────────│
        │                                    │
        │ 2. Creates BTC/ETH HTLC:           │
        │    "If preimage H revealed,         │
        │     pay Alice; else refund T1"      │
        │◀════════════════════════════════════│
        │                                    │
        │ 3. Creates KAS escrow:              │
        │    "If preimage H revealed,         │
        │     pay Bob; else refund T2"        │
        │════════════════════════════════════▶│
        │    (T2 > T1 — Bob can't double-claim)│
        │                                    │
        │ 4. Alice claims BTC by revealing S  │
        │◀════════════════════════════════════│
        │                                    │
        │ 5. Bob claims KAS using same S      │
        │════════════════════════════════════▶│
```

**Key property:** Either both swaps complete or both refund. This is guaranteed by the hashlock (same preimage) and asymmetric timeouts (T1 < T2).

---

## What DagLock Already Has

| Component | Status | File |
|-----------|--------|------|
| SHA-256 hash preimage in covenant | ✅ | `daglock.sil::swap(secret)` — `require(sha256(secret) == tradeHash)` |
| Secret generation | ✅ | `POST /v1/swap/generate` — 32 random bytes + SHA-256 |
| Timeout/refund path | ✅ | `daglock.sil::refund(buyerSig)` — absolute timelock |
| Atomic swap wizard | ✅ | Web UI with 6-step guided flow |
| KAS-only escrow creation | ✅ | Full lifecycle |

## What Needs Building

### Kaspa Side — Minimal Changes

The existing `daglock.sil` covenant is already a valid HTLC for cross-chain use:
- `swap(secret)` — hashlock (reveal preimage → release)
- `refund(buyerSig)` — timelock (after timeout → refund)

**No covenant changes needed** for the Kaspa half. However, we should add:
- A `cross_chain_escrow_id` field to escrows to link the two halves
- A `POST /v1/escrows/:id/link-chain` endpoint that records the external chain details

### Kaspa → Bitcoin (10 weeks)

#### Why Bitcoin is Hard

| Aspect | Challenge |
|--------|-----------|
| Hash function mismatch | Kaspa uses `SHA-256(S)`, Bitcoin uses `HASH160(S)` = `RIPEMD160(SHA-256(S))`. The inner SHA-256 is the same, so the same 32-byte secret works on both — but Bitcoin wraps it in RIPEMD-160 |
| UTXO model | Both are UTXO-based, but Bitcoin confirmation is ~10 min/block vs Kaspa's ~1 sec/block |
| No native covenants | Bitcoin uses P2SH scripts, not SilverScript. Different development model |
| Node requirement | Need Bitcoin access — full node (~600 GB) or ESPLORA API (lighter) |

#### Bitcoin HTLC Script

```bitcoin
OP_IF
    // Redeem with preimage (hashlock path)
    OP_HASH160 <HASH160(preimage)> OP_EQUALVERIFY
    <recipient_pubkey> OP_CHECKSIG
OP_ELSE
    // Refund after timeout (timelock path)
    <timeout> OP_CHECKLOCKTIMEVERIFY OP_DROP
    <sender_pubkey> OP_CHECKSIG
OP_ENDIF
```

This compiles to a P2SH address. The secret `S` is the same 32 bytes on both chains; Bitcoin just applies an outer RIPEMD-160 wrapper.

### Kaspa → Ethereum (6 weeks — Recommended First)

#### Why Ethereum is Easier

| Aspect | Advantage |
|--------|-----------|
| Hash function | Solidity's `sha256()` is an exact match for Kaspa's — same preimage, same hash |
| Smart contract model | Solidity HTLC is ~20 lines, simple to audit |
| Node access | Infura/Alchemy free tier is sufficient for monitoring |
| Confirmation | ~12 sec/block, closer to Kaspa's speed |
| Gas | Claim/refund transactions are cheap (~$2-5) |

#### Solidity HTLC Contract

```solidity
contract HTLC {
    bytes32 public hashlock;
    uint256 public timelock;
    address payable public sender;
    address payable public recipient;

    event Claimed(bytes32 preimage);
    event Refunded();

    constructor(bytes32 _hashlock, address payable _recipient, uint256 _timelock) payable {
        sender = payable(msg.sender);
        hashlock = _hashlock;
        recipient = _recipient;
        timelock = _timelock;
    }

    function claim(bytes calldata _preimage) external {
        require(sha256(_preimage) == hashlock, "Invalid preimage");
        require(block.timestamp < timelock, "Timelock expired");
        recipient.transfer(address(this).balance);
        emit Claimed(sha256(_preimage));
    }

    function refund() external {
        require(block.timestamp >= timelock, "Not yet expired");
        sender.transfer(address(this).balance);
        emit Refunded();
    }
}
```

---

## Architecture: The Relayer

```
┌──────────────────────────────────────────────────┐
│              DagLock Indexer (existing)           │
│  ┌────────────────────┐  ┌─────────────────────┐ │
│  │ Kaspa Monitor      │  │ Cross-Chain Table   │ │
│  │ (wRPC — exists)    │  │ (new DB table)      │ │
│  └─────────┬──────────┘  └──────────┬──────────┘ │
│            │                        │             │
│  ┌─────────▼────────────────────────▼─────────┐  │
│  │         Coordination Engine (NEW)          │  │
│  │  - Match Kaspa escrows to external HTLCs   │  │
│  │  - Detect preimage on external chain       │  │
│  │  - Submit preimage to Kaspa swap path      │  │
│  │  - Handle asymmetric timeouts              │  │
│  │  - Manage relayer wallet (BTC fees/ETH gas)│  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  ┌──────────────────┐  ┌──────────────────────┐  │
│  │ Bitcoin Monitor  │  │ Ethereum Monitor     │  │
│  │ (BDK/ESPLORA)   │  │ (alloy-rs/Infura)    │  │
│  └──────────────────┘  └──────────────────────┘  │
└──────────────────────────────────────────────────┘
```

### New Table: `cross_chain_swaps`

```sql
CREATE TABLE IF NOT EXISTS cross_chain_swaps (
    id TEXT PRIMARY KEY,
    escrow_id TEXT NOT NULL REFERENCES escrows(id),
    chain TEXT NOT NULL,              -- "bitcoin" | "ethereum"
    chain_escrow_id TEXT,             -- BTC txid or ETH contract address
    initiator_address TEXT,           -- Kaspa address of party who started it
    counterparty_address TEXT,        -- External chain address
    hashlock_hex TEXT NOT NULL,       -- SHA-256 preimage hash (shared across chains)
    amount_sompi INTEGER NOT NULL,    -- KAS amount locked
    external_amount TEXT NOT NULL,    -- BTC satoshis or ETH wei
    status TEXT NOT NULL,             -- "pending", "htlc_created", "claimed", "refunded"
    timeout_seconds INTEGER NOT NULL, -- KAS-side timeout
    external_timeout INTEGER,         -- BTC/ETH block timestamp or block height
    created_at INTEGER NOT NULL,
    claimed_at INTEGER,
    refunded_at INTEGER
);
```

### Relayer Wallet

- **Kaspa**: Indexer already has a hot wallet via `--anchor-wallet-key` — reuse for cross-chain
- **Bitcoin**: BDK (Bitcoin Dev Kit) with ESPLORA backend — no full node needed. Hot wallet for claim/refund fees
- **Ethereum**: alloy-rs with Infura/Alchemy — JSON-RPC for contract deployment/claim/refund. Hot wallet for gas

---

## Phase Plan

## Track A: Kaspa→Ethereum (Recommended First — ~6 weeks)

### Phase A1: Solidity HTLC Contract `[ ]` [1 week]
- [ ] Write `contracts/eth/HTLC.sol` with claim() and refund() paths
- [ ] Write Foundry/Hardhat tests for all paths
- [ ] Deploy on Ethereum Sepolia testnet for testing
- **Checkpoint:** Contract deployed and verified on Sepolia explorer
- **Fallback:** Use OpenZeppelin's HTLC implementation

### Phase A2: Ethereum Monitor `[ ]` [1.5 weeks]
- [ ] Create `relayer/src/ethereum_monitor.rs` using `alloy-rs`
- [ ] Watch for HTLC contract deployment events from Indexer
- [ ] Detect `Claimed` events (preimage revealed → relay to Kaspa)
- [ ] Detect `Refunded` events (timeout → mark Kaspa escrow for refund)
- [ ] Handle Ethereum reorgs (wait 12 confirmations)
- **Checkpoint:** Relayer detects a Sepolia HTLC claim within 30 seconds
- **Fallback:** Poll-only mode (every 30s) if WebSocket unavailable

### Phase A3: Kaspa Side Integration `[ ]` [1 week]
- [ ] Add `escrow_id` ↔ `chain_contract` mapping in new cross_chain_swaps table
- [ ] `POST /v1/cross-chain/initiate` — creates Kaspa escrow with `tradeHash` + records external chain info
- [ ] Modify escrow flow: cross-chain escrows use longer timeout (allow time for BTC/ETH confirmations)
- [ ] Add auth for relayer to call swap on behalf of counterparty
- **Checkpoint:** Cross-chain escrow created via API, listed in DB table

### Phase A4: Coordination Engine `[ ]` [1.5 weeks]
- [ ] Implement state machine: `pending → htlc_created → claimed → settled`
- [ ] On `Claimed` event from ETH: submit preimage to `POST /v1/escrows/:id/swap`
- [ ] On timeout: trigger ETH refund transaction
- [ ] Handle edge cases: both claim simultaneously, reorgs, gas failures
- **Checkpoint:** Full KAS↔ETH swap completes on Sepolia + testnet-11
- **Fallback:** Manual claim via web UI (copy-paste preimage)

### Phase A5: Web UI `[ ]` [1 week]
- [ ] New tab on swap page: "Cross-Chain" (KAS→ETH / KAS→BTC)
- [ ] Shows ETH contract address, block explorer link
- [ ] Status: "Waiting for ETH HTLC" → "Waiting for confirmation" → "Complete"
- [ ] Relayer health indicator
- **Checkpoint:** Cross-chain swap wizard on web UI works end-to-end
- **Fallback:** API-only (no UI), CLI tool for initiating swaps

---

## Track B: Kaspa→Bitcoin (~6 weeks, starts after A)

### Phase B1: Bitcoin HTLC Script `[ ]` [1 week]
- [ ] Write Bitcoin script (P2SH) using `rust-bitcoin`
- [ ] Generate P2SH address from script
- [ ] Build faucet/claim/refund transactions using BDK
- **Checkpoint:** Bitcoin testnet HTLC created and claimed via BDK

### Phase B2: Bitcoin Monitor `[ ]` [1.5 weeks]
- [ ] Create `relayer/src/bitcoin_monitor.rs` using BDK + ESPLORA
- [ ] Watch P2SH UTXOs for the HTLC address
- [ ] Detect spending transaction (preimage revelation or timeout)
- **Checkpoint:** Relayer detects a Bitcoin testnet HTLC spend within 1 block
- **Fallback:** Poll ESPLORA every 30s

### Phase B3: Coordination Engine Extension `[ ]` [1.5 weeks]
- [ ] Extend state machine with Bitcoin-specific timeout rules (6 confirmations ≈ 60 min)
- [ ] On preimage reveal on BTC: relay to Kaspa swap endpoint
- [ ] On timeout: broadcast BTC refund transaction
- **Checkpoint:** Full KAS↔BTC swap completes on testnet
- **Fallback:** Manual preimage relay (detect + alert, human decides)

### Phase B4: Testnet + Docs `[ ]` [1 week]
- [ ] Deploy to testnet-11 + BTC testnet + ETH Sepolia
- [ ] Integration tests for all paths
- [ ] Documentation: how to initiate a cross-chain swap
- **Checkpoint:** Cross-chain swaps working on all three testnets
- **Fallback:** Single-chain test only (just Ethereum)

---

## Timeout Safety (Critical)

**Asymmetric timeouts prevent theft:**
```
BTC/ETH HTLC timeout: T1 = 24 hours from creation
KAS escrow timeout:   T2 = 48 hours from creation
                        T2 - T1 = 24 hours margin
```

Alice must be able to claim BTC/ETH before Bob can refund — otherwise Bob could wait for Alice to reveal S on one chain, then refund on the other and still claim.

The 24-hour margin accounts for:
- Bitcoin confirmation delays (6 blocks × 10 min = 1 hour)
- Ethereum confirmation delays (12 blocks × 12 sec = 2.4 min)
- Relayer processing time (1-5 min)
- Gas price spikes delaying transactions

**Enforced at the application layer** (not in covenant — the covenant only sees its own timeout).

---

## Security Considerations

| Risk | Severity | Mitigation |
|------|:--------:|------------|
| Asymmetric timeout violation | 🔴 Critical | Enforce T2 > T1 + 24h at API level |
| Bitcoin reorg | 🟡 Medium | Wait 6 confirmations before considering final |
| Ethereum reorg | 🟡 Medium | Wait 12 confirmations (rare, but possible) |
| Gas price spike | 🟡 Medium | Dynamic gas estimation, replace stuck txs |
| Relayer key compromise | 🔴 Critical | Separate hot wallet per chain, minimal balances |
| HTLC script bug (BTC) | 🔴 Critical | Formal verification, testnet testing |
| HTLC contract bug (ETH) | 🔴 Critical | OpenZeppelin audit patterns, immutable contract |
| Relayer downtime | 🟡 Medium | Redundant instances, manual fallback (web UI) |

---

## Files to Create

| Track | New Files |
|-------|-----------|
| A1 | `contracts/eth/HTLC.sol`, `contracts/eth/test/` |
| A2 | `relayer/src/ethereum_monitor.rs` |
| A3 | `indexer/src/api/cross_chain.rs`, `indexer/src/db/queries/cross_chain.rs` |
| A4 | `relayer/src/coordination.rs`, `relayer/src/main.rs`, `relayer/src/config.rs` |
| A5 | `web/src/pages/CrossChainPage.tsx` |
| B1 | `contracts/btc/htlc.rs` (rust-bitcoin script) |
| B2 | `relayer/src/bitcoin_monitor.rs` |
| B3 | Update `relayer/src/coordination.rs` |

## Files to Modify

| Track | Modified Files |
|-------|---------------|
| A3 | `indexer/src/db/schema.rs` (migrations), `indexer/src/api/mod.rs` (routes), `indexer/src/types.rs` |
| A4 | `indexer/src/main.rs` (spawn relayer) |
| A5 | `web/src/api.ts`, `web/src/App.tsx`, `web/src/layout/Sidebar.tsx` |

---

## Verdict: Should You Build This?

| Factor | Kaspa→Ethereum | Kaspa→Bitcoin |
|--------|:--------------:|:-------------:|
| Effort | ~6 weeks | ~10 weeks |
| Existing users who need it | Low (Kaspa has ETH traders) | Low |
| Competitive advantage | Medium (no one has this on Kaspa) | Medium |
| Risk | Medium | High |
| **Build now?** | ❌ Wait for user demand | ❌ Wait for user demand |

**Recommendation:** Don't start until at least one user asks for it. The Kaspa community is small enough that you'll hear about the need directly. When someone says "I want to swap KAS for BTC," then build it — starting with Ethereum (easier), then Bitcoin.

When that happens, start with Ethereum (4-6 weeks) — same SHA-256 hash, Solidity is easy, Infura is free. Add Bitcoin (6 more weeks) after the Ethereum code is proven.
