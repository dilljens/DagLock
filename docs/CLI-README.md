# daglock-cli — Command Line Tool

> Power-user terminal tool for DagLock escrow operations.

## Installation

```bash
cargo install --git https://github.com/dilljens/DagLock daglock-cli
```

Or build from source:
```bash
cd daglock/cli && cargo build --release
./target/release/daglock-cli --help
```

## Usage

All commands take `--api-url` to point at a DagLock indexer:

```bash
daglock-cli --api-url https://api.daglock.io <command>
```

Set a default in your config:
```bash
daglock-cli config --api-url https://api.daglock.io
```

## Commands

### Escrows

| Command | What it does |
|---------|-------------|
| `create --amount 500 --counterparty kaspa:...` | Create a new escrow proposal |
| `claim <escrow_id>` | Claim/release an escrow as the seller |
| `refund <escrow_id>` | Refund an escrow as the buyer (after timeout) |
| `dispute <escrow_id> --reason "explanation"` | Dispute an escrow |
| `cancel <escrow_id>` | Cancel an escrow before settlement |
| `status <escrow_id>` | Check escrow status and details |

### Offers

| Command | What it does |
|---------|-------------|
| `offer list` | Browse open offers |
| `offer create --side sell --base KAS --quote KRC20:NACHO --amount 500` | Create an offer |
| `offer accept <offer_id> --address kaspa:...` | Accept an offer as counterparty |
| `offer cancel <offer_id>` | Cancel your own offer |

### Lookups

| Command | What it does |
|---------|-------------|
| `reputation <kaspa_address>` | Check reputation score and trade history |
| `receipt <escrow_id>` | Get a settlement receipt |

### Messaging

| Command | What it does |
|---------|-------------|
| `msg <escrow_id> --text "hello" --address kaspa:... --signature hex` | Send a message on an escrow |
| `messages <escrow_id> --address kaspa:... --signature hex` | Read the escrow message thread |

## Auth

State-changing commands (dispute, cancel, msg) require a wallet signature.
The mock verifier accepts any hex string during development.
In production, sign the message `{action}:{escrow_id}` with your Kaspa wallet.

```bash
# Get a signature from KasWare:
# 1. Open KasWare extension
# 2. Click "Sign Message"
# 3. Enter: settle:esc_abc123
# 4. Copy the hex signature
```
