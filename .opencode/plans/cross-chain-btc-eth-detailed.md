# Detailed Plan: Cross-Chain Atomic Swaps

> **Goal:** Trustless KAS↔BTC and KAS↔ETH atomic swaps using HTLC.
>
> **Status:** Detailed design complete — ready for implementation when prioritized.
>
> **Total effort:** Ethereum ~6 weeks · Bitcoin +4 weeks · Both ~10 weeks

---

## 1. Full Solidity HTLC Contract (Ethereum)

### 1.1 Contract Source: `contracts/eth/HTLC.sol`

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title DagLockHTLC — Hash Time-Locked Contract for cross-chain atomic swaps.
 *
 * Locks ETH for a recipient who knows a SHA-256 preimage.
 * If the preimage is not revealed before `timelock`, the sender can refund.
 *
 * The SHA-256 hash is computed on-chain. This is compatible with DagLock's
 * Kaspa covenant, which also uses SHA-256 — the SAME 32-byte secret works
 * on both chains.
 */
contract DagLockHTLC {
    /// SHA-256 hash of the secret preimage (shared with Kaspa side)
    bytes32 public immutable hashlock;
    /// Unix timestamp after which refund is allowed
    uint256 public immutable timelock;
    /// The party who funded the contract (receives refund)
    address payable public immutable sender;
    /// The party who can claim by revealing the preimage
    address payable public immutable recipient;

    event Claimed(bytes32 indexed preimageHash);
    event Refunded();

    error AlreadyClaimed();
    error AlreadyRefunded();
    error InvalidPreimage();
    error TimelockExpired();
    error TimelockNotExpired();

    constructor(
        bytes32 _hashlock,
        address payable _recipient,
        uint256 _timelock
    ) payable {
        require(_hashlock != bytes32(0), "Hashlock cannot be zero");
        require(_recipient != address(0), "Invalid recipient");
        require(_timelock > block.timestamp, "Timelock must be in the future");
        require(msg.value > 0, "Must lock ETH");

        hashlock = _hashlock;
        recipient = _recipient;
        timelock = _timelock;
        sender = payable(msg.sender);
    }

    /**
     * @notice Claim locked ETH by revealing the SHA-256 preimage.
     * @param _preimage The 32-byte secret that SHA-256 hashes to `hashlock`.
     */
    function claim(bytes calldata _preimage) external {
        if (address(this).balance == 0) revert AlreadyClaimed();

        // Verify the preimage (SHA-256 must match — same hash function as Kaspa)
        if (sha256(_preimage) != hashlock) revert InvalidPreimage();

        // Must claim before timelock expires
        if (block.timestamp >= timelock) revert TimelockExpired();

        // Transfer entire balance to recipient
        uint256 amount = address(this).balance;
        (bool sent, ) = recipient.call{value: amount}("");
        require(sent, "Transfer failed");

        emit Claimed(sha256(_preimage));
    }

    /**
     * @notice Refund the sender after timelock has expired.
     */
    function refund() external {
        if (address(this).balance == 0) revert AlreadyRefunded();
        if (block.timestamp < timelock) revert TimelockNotExpired();

        uint256 amount = address(this).balance;
        (bool sent, ) = sender.call{value: amount}("");
        require(sent, "Transfer failed");

        emit Refunded();
    }

    /// @return true if the contract still holds ETH
    function isActive() external view returns (bool) {
        return address(this).balance > 0;
    }
}
```

### 1.2 Foundry Test: `contracts/eth/test/HTLC.t.sol`

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../HTLC.sol";

contract HTLCTest is Test {
    DagLockHTLC public htlc;
    bytes32 constant HASH = 0xabc123...; // SHA-256 of known secret

    address alice = makeAddr("alice");
    address bob = makeAddr("bob");
    uint256 constant LOCK_AMOUNT = 1 ether;
    uint256 constant TIMELOCK = 1000; // block.timestamp + 1000

    function setUp() public {
        vm.deal(alice, 10 ether);
        vm.deal(bob, 10 ether);

        vm.prank(alice);
        htlc = new DagLockHTLC{value: LOCK_AMOUNT}(HASH, payable(bob), block.timestamp + TIMELOCK);
    }

    function test_claim_succeeds_with_correct_preimage() public {
        vm.prank(bob);
        htlc.claim("known-secret-bytes");
        assertEq(address(htlc).balance, 0);
    }

    function test_claim_fails_with_wrong_preimage() public {
        vm.prank(bob);
        vm.expectRevert(DagLockHTLC.InvalidPreimage.selector);
        htlc.claim("wrong-secret");
    }

    function test_refund_succeeds_after_timelock() public {
        vm.warp(block.timestamp + TIMELOCK + 1);
        vm.prank(alice);
        htlc.refund();
        assertEq(address(htlc).balance, 0);
    }

    function test_refund_fails_before_timelock() public {
        vm.prank(alice);
        vm.expectRevert(DagLockHTLC.TimelockNotExpired.selector);
        htlc.refund();
    }

    function test_cannot_claim_after_refund() public {
        vm.warp(block.timestamp + TIMELOCK + 1);
        vm.prank(alice);
        htlc.refund();
        vm.prank(bob);
        vm.expectRevert(DagLockHTLC.AlreadyRefunded.selector);
        htlc.claim("known-secret-bytes");
    }
}
```

---

## 2. Full Bitcoin HTLC Script

### 2.1 Rust Code: `contracts/btc/htlc.rs`

```rust
//! Bitcoin HTLC script builder for cross-chain atomic swaps.
//!
//! Builds a P2SH-wrapped HTLC with the following script:
//!
//! OP_IF
//!     OP_HASH160 <hash160> OP_EQUALVERIFY
//!     <recipient_pubkey> OP_CHECKSIG
//! OP_ELSE
//!     <timelock> OP_CHECKLOCKTIMEVERIFY OP_DROP
//!     <sender_pubkey> OP_CHECKSIG
//! OP_ENDIF
//!
//! The `hash160` is RIPEMD160(SHA256(preimage)).
//! The same 32-byte secret works on Bitcoin (via HASH160) and Kaspa (via SHA256).

use bitcoin::blockdata::opcodes;
use bitcoin::blockdata::script::{Builder, Script};
use bitcoin::hashes::hash160;
use bitcoin::key::PublicKey;
use bitcoin::Network;

/// Build the HTLC redeem script.
/// Returns the raw script bytes and the P2SH address.
pub fn build_htlc_script(
    recipient_pk: &PublicKey,
    sender_pk: &PublicKey,
    preimage_hash: &[u8; 32],   // SHA-256 hash (from Kaspa side)
    timelock: u32,               // CLTV timelock (blocks or timestamp)
    is_block_height: bool,       // true = block height, false = timestamp
) -> Result<(Script, String), String> {
    // Bitcoin's HASH160 = RIPEMD160(SHA256(preimage))
    // The same 32-byte secret works — SHA-256 is applied on both chains
    // Bitcoin just wraps it in an outer RIPEMD-160
    let hash160 = hash160::Hash::hash(preimage_hash);

    let mut builder = Builder::new();
    builder = builder
        .push_opcode(opcodes::all::OP_IF)
        // Hashlock path: recipient reveals preimage
        .push_opcode(opcodes::all::OP_HASH160)
        .push_slice(&hash160[..])
        .push_opcode(opcodes::all::OP_EQUALVERIFY)
        .push_key(recipient_pk)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .push_opcode(opcodes::all::OP_ELSE)
        // Timelock path: sender refunds after timeout
        .push_int(timelock as i64);

    builder = if is_block_height {
        builder.push_opcode(opcodes::all::OP_CHECKLOCKTIMEVERIFY)
    } else {
        // For timestamp-based timelocks, also push OP_CHECKLOCKTIMEVERIFY
        // The integer is interpreted as timestamp if >= 500000000000
        builder.push_opcode(opcodes::all::OP_CHECKLOCKTIMEVERIFY)
    };

    builder = builder
        .push_opcode(opcodes::all::OP_DROP)
        .push_key(sender_pk)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .push_opcode(opcodes::all::OP_ENDIF);

    let redeem_script = builder.into_script()?;

    // Compute P2SH address from redeem script
    use bitcoin::address::Address;
    let p2sh = Address::p2sh(&redeem_script, Network::Testnet)?;

    Ok((redeem_script, p2sh.to_string()))
}
```

### 2.2 Funding Transaction (BDK)

```rust
// Build and broadcast the funding transaction that locks BTC in the HTLC.
// The output sends BTC to the P2SH address derived from the HTLC script.

use bdk::wallet::Wallet;
use bdk::database::MemoryDatabase;
use bdk::blockchain::EsploraBlockchain;
use bdk::SyncOptions;
use bdk::bitcoin::Network;
use bdk::template::Bip86;

fn fund_htlc(
    btc_amount: u64,           // satoshis
    p2sh_address: &str,       // the HTLC P2SH address
    sender_mnemonic: &str,    // sender's HD seed
) -> Result<String, Box<dyn std::error::Error>> {
    let wallet = Wallet::new(
        Bip86(sender_mnemonic, Network::Testnet),
        Some(MemoryDatabase::new()),
        EsploraBlockchain::new("https://blockstream.info/testnet/api"),
    )?;

    wallet.sync(SyncOptions::default(), None)?;

    let mut tx_builder = wallet.build_tx();
    tx_builder
        .add_recipient(p2sh_address.parse::<Address>()?.script_hash(), btc_amount)
        .fee_rate(FeeRate::from_sat_per_vb(5.0));

    let (mut psbt, _) = tx_builder.finish()?;
    wallet.sign(&mut psbt, Default::default())?;
    let tx = psbt.extract_tx();
    let txid = wallet.broadcast(tx)?;

    Ok(txid.to_string())
}
```

### 2.3 Claim Transaction (BDK)

```rust
// Claim BTC by spending the HTLC UTXO with the preimage.
// The witness stack must provide: <sig> <preimage> OP_TRUE

fn claim_htlc(
    htlc_txid: &str,
    htlc_vout: u32,
    htlc_amount: u64,
    redeem_script: &Script,
    preimage: &[u8; 32],
    recipient_mnemonic: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let wallet = Wallet::new(
        Bip86(recipient_mnemonic, Network::Testnet),
        Some(MemoryDatabase::new()),
        EsploraBlockchain::new("https://blockstream.info/testnet/api"),
    )?;

    let mut tx_builder = wallet.build_tx();
    tx_builder
        .add_utxo(OutPoint::new(htlc_txid.parse()?, htlc_vout))
        .add_recipient(
            wallet.get_address(NewOptions::default())?.script_hash(),
            htlc_amount - 1000, // minus fee
        )
        .fee_rate(FeeRate::from_sat_per_vb(10.0));

    let (mut psbt, _) = tx_builder.finish()?;
    wallet.sign(&mut psbt, Default::default())?;

    // Add the preimage to the witness stack for the HTLC input
    let mut tx = psbt.extract_tx();
    tx.input[0].witness = Witness::from_slice(&[
        &[],                  // dummy for OP_0 (CHECKSEQUENCEVERIFY not used)
        preimage,
        &[1u8],              // OP_TRUE — enters the OP_IF branch
        redeem_script.as_bytes(),
    ]);

    let txid = wallet.broadcast(tx)?;
    Ok(txid.to_string())
}
```

---

## 3. Cross-Chain State Machine

### 3.1 States

```
                  ┌───────────────────────────┐
                  │   PENDING                 │
                  │   Creator generated S, H  │
                  │   NOT yet on Kaspa        │
                  └───────────┬───────────────┘
                              │
                  ┌───────────▼───────────────┐
                  │   KAS_LOCKED              │
                  │   Creator locked KAS in   │
                  │   DagLock escrow with     │
                  │   tradeHash = H           │
                  └───────────┬───────────────┘
                              │
                  ┌───────────▼───────────────┐
          ┌──────┤   HTLC_DEPLOYED           │
          │      │   Counterparty locked      │
          │      │   BTC/ETH in HTLC with     │
          │      │   hashlock = H             │
          │      └───────────┬───────────────┘
          │                  │
          │      ┌───────────▼───────────────┐
          │      │   PREIMAGE_DETECTED        │
          │      │   Relayer detected         │
          │      │   preimage on BTC/ETH      │
          │      └───────────┬───────────────┘
          │                  │
          │      ┌───────────▼───────────────┐
          │      │   KAS_SETTLED              │
          │      │   Relayer called            │
          │      │   swap(secret) on Kaspa    │
          │      │   → DONE ✅               │
          │      └───────────────────────────┘
          │
          │      ┌───────────────────────────┐
          └──────┤   HTLC_TIMEOUT            │
                 │   BTC/ETH HTLC expired    │
                 │   → DONE (refund)         │
                 └───────────────────────────┘
```

### 3.2 State Transition Table

| Current State | Event | Next State | Action |
|--------------|-------|-----------|--------|
| PENDING | Kaspa escrow created with tradeHash | KAS_LOCKED | Store escrow_id in cross_chain_swaps |
| KAS_LOCKED | Counterparty creates BTC/ETH HTLC | HTLC_DEPLOYED | Store HTLC address/txid |
| HTLC_DEPLOYED | Claimed event on BTC/ETH | PREIMAGE_DETECTED | Extract preimage from BTC/ETH tx |
| PREIMAGE_DETECTED | Relayer calls `POST /v1/escrows/:id/swap` | KAS_SETTLED | Done |
| HTLC_DEPLOYED | KAS timeout approaching (edge) | EXPIRING_SOON | Notify relayer to watch |
| KAS_LOCKED | KAS timeout exceeded + no HTLC | CANCELLED | Buyer refunds on Kaspa |
| HTLC_DEPLOYED | BTC/ETH timeout exceeded | HTLC_TIMEOUT | Sender refunds on BTC/ETH |

---

## 4. Database Schema

```sql
-- Core cross-chain swap record
CREATE TABLE IF NOT EXISTS cross_chain_swaps (
    id TEXT PRIMARY KEY,
    escrow_id TEXT NOT NULL UNIQUE REFERENCES escrows(id),
    
    -- Which external chain
    chain TEXT NOT NULL CHECK(chain IN ('bitcoin', 'ethereum')),
    
    -- External chain identifiers
    chain_contract_address TEXT,       -- ETH contract address or BTC P2SH address
    chain_funding_txid TEXT,           -- BTC/ETH funding transaction hash
    chain_claim_txid TEXT,             -- BTC/ETH claim transaction hash
    
    -- Parties (Kaspa addresses + external addresses)
    initiator_address TEXT NOT NULL,   -- Kaspa address of the person who started it
    counterparty_address TEXT NOT NULL, -- The other party's Kaspa address
    external_address TEXT,             -- BTC/ETH address of the external party
    
    -- HTLC parameters (shared secret)
    secret_hash_hex TEXT NOT NULL,     -- SHA-256 (32 bytes, 64 hex chars) — SAME on both chains
    preimage_hex TEXT,                 -- The 32-byte secret (stored encrypted, only for relayer)
    
    -- Amounts
    kaspa_amount_sompi INTEGER NOT NULL,
    external_amount_sat_wei TEXT NOT NULL, -- satoshis for BTC, wei for ETH (bigint as string)
    
    -- Timeouts
    kaspa_timeout INTEGER NOT NULL,    -- Unix timestamp for KAS escrow refund
    external_timeout INTEGER NOT NULL, -- Unix timestamp for BTC/ETH refund
    
    -- Lifecycle
    status TEXT NOT NULL DEFAULT 'pending' 
        CHECK(status IN ('pending','kas_locked','htlc_deployed',
                         'preimage_detected','kas_settled','cancelled','htlc_timeout')),
    created_at INTEGER NOT NULL,
    claimed_at INTEGER,
    settled_at INTEGER,
    refunded_at INTEGER
);

CREATE INDEX idx_cross_chain_escrow ON cross_chain_swaps(escrow_id);
CREATE INDEX idx_cross_chain_status ON cross_chain_swaps(status);
CREATE INDEX idx_cross_chain_contract ON cross_chain_swaps(chain_contract_address);
```

---

## 5. REST API

### 5.1 Initiate Cross-Chain Swap

```
POST /v1/cross-chain/initiate

Headers:
  X-Daglock-Address: <kaspa_address>
  X-Daglock-Signature: <schnorr_sig>
  X-Daglock-Message: "cross-chain:initiate:<secret_hash>"

Request Body:
{
  "chain": "ethereum",              // "bitcoin" | "ethereum"
  "counterparty_address": "kaspa:...",
  "external_address": "0x...",      // recipient ETH address
  "kaspa_amount_sompi": 100000000,  // 1 KAS
  "external_amount": "1000000000000000000",  // 1 ETH in wei
  "kaspa_timeout_hours": 48,
  "external_timeout_hours": 24,
  "secret_hash": "abc123..."        // 64 hex chars — SHA-256 of preimage
}

Response 201:
{
  "swap_id": "xc_abc123",
  "escrow_id": "esc_xyz...",
  "status": "pending",
  "kaspa_escrow_instructions": {
    "buyer_address": "kaspa:...",
    "seller_address": "kaspa:...",
    "amount_sompi": 100000000,
    "trade_hash": "abc123..."
  },
  "external_instructions": {
    "chain": "ethereum",
    "contract_address": null,      // not yet deployed
    "expected_contract": {
      "hashlock": "abc123...",
      "recipient": "0x...",
      "timelock": 1800000000
    }
  }
}
```

### 5.2 Report HTLC Deployed

```
POST /v1/cross-chain/:swap_id/htlc-deployed

Request Body (Ethereum):
{
  "contract_address": "0x1234...",
  "deploy_txid": "0xabcd...",
  "timelock": 1800000000
}

Request Body (Bitcoin):
{
  "p2sh_address": "2N...",
  "funding_txid": "abc...",
  "vout": 0,
  "amount_sat": 100000000
}

Response 200:
{
  "status": "htlc_deployed",
  "message": "HTLC detected, relayer monitoring for claim",
  "monitoring": {
    "chain": "ethereum",
    "confirmations_needed": 12,
    "timeout_at": 1800000000
  }
}
```

### 5.3 Get Swap Status

```
GET /v1/cross-chain/:swap_id

Response 200:
{
  "swap_id": "xc_abc123",
  "status": "htlc_deployed",
  "escrow_id": "esc_xyz...",
  "chain": "ethereum",
  "contract_address": "0x1234...",
  "timeline": {
    "created": 1700000000,
    "kaspa_locked": 1700000100,
    "htlc_deployed": 1700000200,
    "claimed": null,
    "settled": null
  },
  "timeouts": {
    "kaspa": 1700086400,       // 48h from creation
    "external": 1700000000 + 86400,  // 24h from creation
    "kaspa_remaining_seconds": 86400,
    "external_remaining_seconds": 43200
  },
  "kaspa_explorer_url": "https://kas.fyi/...",
  "external_explorer_url": "https://sepolia.etherscan.io/address/0x1234..."
}
```

### 5.4 List Cross-Chain Swaps

```
GET /v1/cross-chain?address=kaspa:...

Response 200:
{
  "swaps": [{ ... }],
  "total": 5
}
```

---

## 6. Ethereum Monitor — Rust Implementation

```rust
//! relayer/src/ethereum_monitor.rs
//!
//! Watches for HTLC Claimed events on Ethereum via WebSocket.
//! When a preimage is revealed, relays it to the DagLock Kaspa indexer.

use alloy::providers::{Provider, WsConnect};
use alloy::rpc::types::Filter;
use alloy::sol;
use alloy::primitives::{Address, U256, B256};
use std::sync::Arc;
use tokio::time::{interval, Duration};

// Solidity event signature: Claimed(bytes32 indexed preimageHash)
sol! {
    event Claimed(bytes32 indexed preimageHash);
    event Refunded();
}

pub struct EthereumMonitor {
    provider: Arc<dyn Provider>,
    daglock_api_url: String,
    daglock_api_key: String,
}

impl EthereumMonitor {
    pub async fn new(rpc_url: &str, api_url: &str, api_key: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let ws = WsConnect::new(rpc_url);
        let provider = alloy::providers::ProviderBuilder::default()
            .on_ws(ws)
            .await?;

        Ok(Self {
            provider: Arc::new(provider),
            daglock_api_url: api_url.to_string(),
            daglock_api_key: api_key.to_string(),
        })
    }

    /// Start monitoring a specific HTLC contract for Claim events.
    pub async fn monitor_contract(&self, contract_address: Address, swap_id: &str) {
        let filter = Filter::new()
            .address(contract_address)
            .event("Claimed(bytes32)");

        let sub = self.provider.subscribe_logs(&filter).await.unwrap();
        let mut stream = sub.into_stream();

        tokio::spawn(async move {
            while let Some(log) = stream.next().await {
                // Extract the preimage from the log
                // The Claimed event emits sha256(preimage), not the preimage itself
                // We need the transaction input data to get the actual preimage
                let tx_hash = log.transaction_hash.unwrap();
                let tx = self.provider.get_transaction_by_hash(tx_hash).await.unwrap();
                
                if let Some(tx) = tx {
                    // The preimage is in the `claim(bytes)` function input
                    // Decode it from the transaction calldata
                    if let Some(preimage) = extract_preimage_from_calldata(&tx.input) {
                        // Relay to DagLock
                        let client = reqwest::Client::new();
                        let resp = client
                            .post(format!("{}/v1/escrows/{}/swap", self.daglock_api_url, swap_id))
                            .json(&serde_json::json!({
                                "preimage": hex::encode(&preimage)
                            }))
                            .send()
                            .await;
                    }
                }
            }
        });
    }
}
```

---

## 7. Coordination Engine — Rust State Machine

```rust
//! relayer/src/coordination.rs
//!
//! Manages the state machine for each cross-chain swap.
//! Runs as a background task in the DagLock indexer.

use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
pub enum SwapStatus {
    Pending,
    KasLocked,
    HtlcDeployed,
    PreimageDetected,
    KasSettled,
    Cancelled,
    HtlcTimeout,
}

pub struct CoordinationEngine {
    db: Pool<Sqlite>,
    eth_monitor: Option<EthereumMonitor>,
    btc_monitor: Option<BitcoinMonitor>,
    /// In-memory cache of active cross-chain swaps
    active: Arc<Mutex<HashMap<String, CrossChainSwap>>>,
}

impl CoordinationEngine {
    /// Background loop that runs every 30 seconds.
    /// 1. Check for new escrows with trade_hash (cross-chain intent)
    /// 2. Check for HTLC timeout on external chain
    /// 3. Check for Kaspa timeout (need to refund Kaspa side)
    /// 4. Check for preimage detection
    pub async fn run_loop(&self) {
        let mut timer = interval(Duration::from_secs(30));
        loop {
            timer.tick().await;
            
            // Query cross_chain_swaps for active swaps
            let swaps = sqlx::query_as::<_, CrossChainSwap>(
                "SELECT * FROM cross_chain_swaps WHERE status IN ('htlc_deployed', 'kas_locked')"
            )
            .fetch_all(&self.db)
            .await
            .unwrap_or_default();

            for swap in swaps {
                match swap.status.as_str() {
                    "htlc_deployed" => {
                        // Check if external HTLC has been claimed
                        if self.check_external_claimed(&swap).await {
                            // Preimage detected! Relay to Kaspa
                            self.relay_to_kaspa(&swap).await;
                        }
                        // Check if external HTLC has timed out
                        let now = chrono::Utc::now().timestamp();
                        if now >= swap.external_timeout {
                            self.handle_external_timeout(&swap).await;
                        }
                    }
                    "kas_locked" => {
                        // Check if Kaspa timeout is approaching with no HTLC
                        // Notify user to refund
                        let now = chrono::Utc::now().timestamp();
                        if now >= swap.kaspa_timeout - 3600 {
                            // 1 hour warning
                            self.notify_timeout_warning(&swap).await;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
```

---

## 8. Kaspa Side Escrow Flow

### 8.1 Modified Create Escrow for Cross-Chain

The existing `POST /v1/escrows` already supports `trade_hash`. No changes needed.
Cross-chain swaps just use a longer timeout and the existing `swap(secret)` path.

**Key constraint:** Cross-chain escrows must have `expiration_daa_score` set
far enough in the future to allow BTC/ETH confirmation time.

### 8.2 Kaspa Timeout Safety

```rust
// In indexer/src/services/escrow_service.rs
// When creating a cross-chain escrow:
// 1. Validate: kaspa_timeout >= external_timeout + 24h_margin
// 2. Set timeout in covenant to kaspa_timeout
// 3. The swap(secret) path is available at any time before timeout
// 4. The refund path is available after timeout

fn validate_cross_chain_timeouts(
    kaspa_timeout: i64,
    external_timeout: i64,
) -> Result<(), ServiceError> {
    let min_margin = 24 * 3600; // 24 hours
    if kaspa_timeout < external_timeout + min_margin {
        return Err(ServiceError::InvalidInput(
            format!("Kaspa timeout must be at least {}s after external timeout", min_margin)
        ));
    }
    Ok(())
}
```

---

## 9. User Flow (Web UI)

### Step 1: Initiate (Alice)

```
┌──────────────────────────────────────────┐
│  Cross-Chain Atomic Swap                 │
│                                          │
│  I want to swap:                         │
│  [1 KAS]  ◄──►  [0.05 ETH]              │
│                                          │
│  Chain: [Ethereum ▼]                     │
│  Your ETH address: [0x...]               │
│  Kaspa counterparty: [kaspa:...]         │
│                                          │
│  Timeouts:                               │
│    ETH HTLC: 24 hours                    │
│    KAS escrow: 48 hours                  │
│    ⚠️ 24h margin prevents theft          │
│                                          │
│  [Generate Secret & Create Escrow]       │
└──────────────────────────────────────────┘
```

### Step 2: Share Instructions (Alice)

```
┌──────────────────────────────────────────┐
│  🔑 Secret (save this!):                │
│  f8a2...e9b3                             │
│                                          │
│  📋 Copy instructions for counterparty:  │
│                                          │
│  1. Deploy this HTLC contract:           │
│     Contract code: github.com/.../HTLC.sol│
│     Parameters:                          │
│       hashlock: 7b3c...                  │
│       recipient: 0x... (your ETH addr)   │
│       timelock: 1700086400               │
│                                          │
│  2. Fund the contract with 0.05 ETH      │
│                                          │
│  3. Wait for me to claim on Ethereum     │
│     (I'll reveal the secret)             │
│                                          │
│  4. Use the secret to claim KAS on       │
│     DagLock: https://daglock.com/swap/xyz│
│                                          │
│  [Copy Instructions] [Share Link]        │
└──────────────────────────────────────────┘
```

### Step 3: Waiting (Alice)

```
┌──────────────────────────────────────────┐
│  ⏳ Waiting for counterparty...          │
│                                          │
│  Status: KAS locked ✓                    │
│  Status: Waiting for ETH HTLC...         │
│                                          │
│  ⏰ ETH HTLC timeout: 22h 15m remaining  │
│  ⏰ KAS refund available: 46h 15m        │
│                                          │
│  [Cancel Swap — refund KAS]              │
└──────────────────────────────────────────┘
```

### Step 4: Counterparty View (Bob)

```
┌──────────────────────────────────────────┐
│  🔄 Cross-chain swap offered             │
│                                          │
│  0.05 ETH → 1 KAS                        │
│                                          │
│  Instructions:                           │
│  1. Deploy HTLC with hashlock 7b3c...     │
│  2. Fund with 0.05 ETH                   │
│  3. Alice will claim ETH → you get secret│
│  4. Use secret to claim KAS              │
│                                          │
│  [I've deployed the HTLC — notify relayer]│
│  [I've claimed the KAS — done ✅]        │
└──────────────────────────────────────────┘
```

---

## 10. Cost Analysis

| Item | Bitcoin | Ethereum |
|------|---------|----------|
| **HTLC creation** | ~$0.50 (BTC fee) | ~$3-10 (ETH gas) |
| **HTLC claim** | ~$0.50 (BTC fee) | ~$3-10 (ETH gas) |
| **HTLC refund** | ~$0.50 (BTC fee) | ~$3-10 (ETH gas) |
| **Relayer monitoring** | ESPLORA: free | Infura free tier: 100K req/day |
| **Total per swap** | ~$1.50 | ~$10-30 |

**ETH gas costs dominate.** At current ETH prices, deploying + claiming an HTLC costs ~$10-30.
This makes small swaps uneconomical on Ethereum. Bitcoin fees are lower.

---

## 11. Error Handling Matrix

| Scenario | Behavior |
|----------|----------|
| Alice creates KAS escrow, Bob never deploys HTLC | Alice refunds after KAS timeout (standard escrow refund) |
| Bob deploys HTLC, Alice never claims | Bob refunds after BTC/ETH HTLC timeout |
| Alice claims BTC/ETH, relayer fails to relay preimage to Kaspa | Manual: Alice can use the secret on the web UI to claim KAS herself |
| Preimage detected but Kaspa swap RPC fails | Retry 5 times with exponential backoff. After 5 failures, alert operator |
| Ethereum reorg after claim | Wait 12 confirmations before considering final. If reorg unclaims, monitor again |
| Bitcoin reorg after claim | Wait 6 confirmations. Same approach |
| Gas price spikes, claim tx stuck on ETH | Replace tx with higher gas price (CPFP or speed-up) |
| Relayer goes down mid-swap | Web UI exposes "I have the preimage — claim manually" button |
| Both parties try to claim simultaneously | Covenant only allows one spend. First valid tx wins |
| Secret/private key lost | Recovery via Kaspa refund (48h timeout) or BTC/ETH refund (24h timeout) |

---

## 12. Testing Plan

| Test | What It Validates |
|------|-------------------|
| `test_solidity_compiles` | HTLC.sol compiles with Solidity 0.8.20+ |
| `test_claim_correct_preimage` | ETH claim succeeds with valid preimage |
| `test_claim_wrong_preimage` | ETH claim fails with wrong preimage |
| `test_refund_after_timeout` | ETH refund succeeds after timelock |
| `test_refund_before_timeout` | ETH refund fails before timelock |
| `test_bitcoin_script_parses` | HTLC script builds and serializes correctly |
| `test_bitcoin_p2sh_address` | P2SH address computes correctly from script |
| `test_bitcoin_claim_tx` | Claim tx with preimage passes script verification |
| `test_relayer_detects_claim` | Relayer receives Claimed event within 30s |
| `test_relayer_relays_preimage` | Preimage is submitted to Kaspa API |
| `test_full_kaspa_eth_flow` | End-to-end swap on testnets |
| `test_timeout_safety` | Enforces T2 > T1 + 24h |
| `test_reorg_handling` | Reorg during claim doesn't cause double-spend |
| `test_relayer_downtime_manual_fallback` | Manual claim works via web UI |

---

## Summary: When to Build

| Question | Answer |
|----------|--------|
| **Should you build this now?** | ❌ No. Wait for user demand. |
| **Which chain first?** | Ethereum (same SHA-256, simpler, faster) |
| **How long?** | Ethereum: ~6 weeks. Bitcoin: +4 weeks. Both: ~10 weeks |
| **Biggest risk?** | Asymmetric timeout violation (solved by 24h margin) |
| **Most expensive?** | Ethereum gas ($10-30/swap) |
| **Kaspa covenant changes?** | None. Existing `daglock.sil` works as-is. |
