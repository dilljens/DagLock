# DagLock Architecture

> System design, component relationships, and data flow for the DagLock trustless escrow protocol. Updated with market-research-driven components.

---

## 1. High-Level System Diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│                          Users                                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │
│  │  Telegram    │  │  Web         │  │  CLI         │               │
│  │  @DagLockBot │  │  daglock.io  │  │  daglock-cli │               │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘               │
│         │                 │                 │                        │
│         └─────────────────┼─────────────────┘                        │
│                           │ KasWare / Kaspium signing               │
└───────────────────────────┼──────────────────────────────────────────┘
                            │ HTTPS + WebSocket
                            ▼
┌──────────────────────────────────────────────────────────────────────┐
│                      DagLock Indexer (Rust)                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐ │
│  │ wRPC     │ │ Template │ │ Database │ │ REST     │ │ Reputation│ │
│  │ Listener │▶│ Matcher  │▶│ (SQLite  │▶│ API      │▶│ Engine    │ │
│  │          │ │(KAS+KRC) │ │ /PG)     │ │          │ │           │ │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └───────────┘ │
│                                                  │                   │
│  ┌──────────┐ ┌──────────┐                      │                   │
│  │ Offer    │ │ Receipt  │◀─────────────────────┘                   │
│  │ Board    │ │ Generator│                                          │
│  └──────────┘ └──────────┘                                          │
└───────────────────────────┼──────────────────────────────────────────┘
                            │ wRPC (Borsh binary over WebSocket)
                            ▼
┌──────────────────────────────────────────────────────────────────────┐
│               Kaspa Node (rusty-kaspa)                               │
│  BlockDAG Consensus Engine (10 BPS → 100 BPS)                        │
│  ┌─────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐            │
│  │ Mempool │ │ UTXO Set │ │  Script  │ │ wRPC Server  │            │
│  │         │ │ (pruned) │ │  Engine  │ │              │            │
│  └─────────┘ └──────────┘ └──────────┘ └──────────────┘            │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 2. Component Descriptions

### 2.1 SilverScript Covenants

Two covenant files, both compiled against `silverscript-lang`:

| Covenant | Asset | Template Hash | Entrypoints |
|---|---|---|---|
| `daglock.sil` | Native KAS | `DAGLOCK_KAS_TEMPLATE` | `release`, `swap`, `refund` |
| `daglock_krc20.sil` | KRC-20 tokens | `DAGLOCK_KRC20_TEMPLATE` | `release`, `swap`, `refund` |

Both share the same three-path architecture but differ in output validation: KAS covenants validate `tx.outputs[0].value`, KRC-20 covenants validate token state transitions.

### 2.2 Rust Indexer

The central backend. Runs as a single binary.

**Sub-components:**

| Component | Responsibility |
|---|---|
| **wRPC Listener** | Subscribe to BlockAdded notifications. Parse blocks for DagLock UTXOs. |
| **Template Matcher** | Compare `tx.output[].scriptPubKey` against both template hashes (KAS + KRC-20). |
| **Database** | Store escrow lifecycle. SQLite for dev/alpha, PostgreSQL for production. |
| **REST API** | Endpoints for escrow CRUD, offer discovery, reputation queries, receipts. |
| **Offer Board** | Public listing of proposed (unfunded) escrow offers. Filter by asset, amount, side. |
| **Reputation Engine** | Aggregates on-chain data per address: trade count, total volume, dispute rate, account age. |
| **Receipt Generator** | Produces signed JSON receipts after settlement with all on-chain verification data. |

### 2.3 Telegram Bot

Node.js application using `grammY` or `telegraf`. Communicates with the indexer REST API.

**Commands:**
- `/create` — guided escrow creation wizard
- `/offers` — browse open offers with inline keyboard
- `/claim <id>` — claim a pending escrow
- `/reputation <address>` — check counterparty stats
- `/receipt <id>` — export settlement receipt
- `/status <id>` — check escrow lifecycle state

**Security:** Bot never sees private keys. Unsigned transactions are passed to KasWare/Kaspium for signing via deep links.

### 2.4 CLI Tool

Rust binary (`daglock-cli`) for power users and scripting.

**Commands:**
- `daglock-cli create --amount 5000 --counterparty kaspa:qz...`
- `daglock-cli claim <escrow-id>`
- `daglock-cli offers --token KRC20:NACHO`
- `daglock-cli receipt <escrow-id> --format json`

### 2.5 Web UI

React + Vite dashboard for browser-based users. Communicates with the indexer REST API.

**Pages:**
- **Create Escrow** — KAS or KRC-20 form, proposal-or-commit toggle
- **Claim** — Trade link detection, terms display, one-click sign
- **Dashboard** — Active escrows for connected wallet
- **Offer Board** — Browse/accept open offers
- **Atomic Swap Wizard** — Guided multi-step swap flow

---

## 3. Data Flow: Counterparty Discovery

```
Alice (Buyer)               DagLock Indexer               Bob (Seller)
     │                            │                            │
     │ 1. POST /v1/offers         │                            │
     │    {side:buy,asset:KAS,    │                            │
     │     amount:5000}           │                            │
     │───────────────────────────▶│                            │
     │                            │                            │
     │                  2. Offer visible on board              │
     │                            │                            │
     │                            │  3. GET /v1/offers?asset=KAS
     │                            │◀───────────────────────────│
     │                            │                            │
     │                            │  4. Bob sees Alice's offer │
     │                            │───────────────────────────▶│
     │                            │                            │
     │                            │  5. POST /v1/offers/:id/accept
     │                            │◀───────────────────────────│
     │                            │                            │
     │  6. Alice notified:        │                            │
     │     "Bob accepted.         │                            │
     │      Fund the escrow now." │                            │
     │◀───────────────────────────│                            │
     │                            │                            │
     │  7. Alice deploys DagLock  │                            │
     │     on-chain (KasWare)     │                            │
     │────────────────────────────│────────────────────────────│
     │                            │                            │
     │                  8. Indexer detects lock tx             │
     │                     Offer status → LOCKED               │
     │                            │                            │
     │  9. Bob claims (signs      │                            │
     │     release tx)            │                            │
     │                            │◀───────────────────────────│
     │                            │                            │
     │                 10. Settlement detected                 │
     │                     Receipt generated                   │
```

---

## 4. Reputation Model

Reputation is derived from on-chain data. No subjective ratings. No centralized scoring.

| Metric | Source | Weight |
|---|---|---|
| **Trade count** | Number of settled escrows (as buyer + seller) | Positive |
| **Total volume** | Sum of all escrow amounts | Positive |
| **Account age** | Time since first Kaspa transaction | Positive |
| **Dispute rate** | Escrows that expired without settlement / total created | Negative |
| **Refund rate** | Refunded escrows / total created | Neutral (legitimate use) |

**Formula:**
```
trade_component = ln(trade_count + 1)
volume_component = ln(volume_kas + 1)
age_factor = clamp(age_days / 30, 0.25, 1.75)
quality_factor = (1 - dispute_rate)^2 * (1 - refund_rate * 0.25)
raw = (trade_component + volume_component) * age_factor * quality_factor
reputation_score = clamp(1 + (raw / 3), 1.0, 5.0)
```

Displayed as a 1-5 shield rating in the UI. Raw metrics always available for verification.

---

## 5. UTXO Contention Strategy (Unchanged)

Each escrow = one distinct UTXO. No shared contract wallets. Zero UTXO contention. This is the core architectural advantage.

---

## 6. Template Hash Detection (Dual-Hash)

The indexer maintains two template hashes:

```
DAGLOCK_KAS_TEMPLATE   = blake2b-160(prefix_kas || suffix_kas)
DAGLOCK_KRC20_TEMPLATE = blake2b-160(prefix_krc20 || suffix_krc20)
```

Each new block is scanned for outputs matching either hash. The indexer routes detected UTXOs to the correct escrow type based on which template hash matched.

---

## 7. Network Topology (Simplified)

```
         ┌──────────────────┐
         │   DagLock DNS    │
         └────────┬─────────┘
                  │
         ┌────────▼─────────┐
         │   Indexer        │  (Single binary, single instance for alpha)
         │   + REST API     │
         └────────┬─────────┘
                  │
         ┌────────▼─────────┐
         │   PostgreSQL     │  (Or SQLite for dev)
         └──────────────────┘
                  │
         ┌────────▼─────────┐
         │   Kaspa Node     │  (wRPC — local or public resolver)
         └──────────────────┘
```

Horizontal scaling (multiple indexers behind load balancer) is deferred until user volume demands it (> 100 concurrent users).

---

## 8. Technology Choices

| Decision | Choice | Why |
|---|---|---|
| Smart contracts | SilverScript | Native L1 covenant compilation; no EVM overhead |
| Indexer | Rust + Axum + SQLx | Matches rusty-kaspa ecosystem; compile-time query checking |
| Telegram bot | Node.js (grammY) | Fastest path to Telegram integration; large ecosystem |
| CLI | Rust (clap) | Shares tx assembly code with indexer |
| Web UI | React + Vite + Tailwind | Fast iteration; KasWare browser detection |
| Wallet integration | KasWare + Kaspium | Dominant Kaspa web + mobile wallets |
| Database (alpha) | SQLite | Zero setup; single file; swap to PG when needed |
| Database (prod) | PostgreSQL | When > 50 concurrent users or HA needed |
