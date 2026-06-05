# @DagLock_bot — Telegram Bot Guide

> Trustless escrow and atomic swaps on Kaspa, from your Telegram chat.

## Getting Started

1. Open [@DagLock_bot](https://t.me/DagLock_bot) on Telegram
2. Send `/start` to begin
3. Use the commands below to create escrows, check reputation, and find trading partners

## Commands

| Command | What it does |
|---------|-------------|
| `/start` | Welcome message and help. Also handles trade links like `t.me/DagLock_bot?start=claim_esc_abc123` |
| `/create` | Opens the web dashboard in Telegram's browser to create an escrow |
| `/claim <id>` | Claim a pending escrow by its ID |
| `/list` | Show all escrows you're involved in |
| `/offers` | Browse open offers on the marketplace |
| `/status <id>` | Check the current status of an escrow |
| `/receipt <id>` | Get a settlement receipt for a completed escrow |
| `/dispute <id> <reason>` | Dispute an escrow with a reason |
| `/cancel <id>` | Cancel an escrow you created |
| `/reputation <address>` | Check a Kaspa address's reputation score |
| `/msg <id> <text>` | Send a message on an escrow thread |
| `/messages <id>` | Read the message thread on an escrow |
| `/help` | Show all commands |

## Tips

- **Trade links:** When you create an escrow, share the trade link with your counterparty. They can claim it with one tap: `t.me/DagLock_bot?start=claim_esc_abc123`
- **Auth required:** Some commands (dispute, cancel, msg) require a wallet signature. Set up your wallet in the web dashboard, then copy the signature to Telegram.
- **No private keys:** The bot never handles private keys. All signing happens in your wallet (KasWare or Kaspium).
- **Testnet:** The bot runs on Kaspa Testnet 12. No real KAS is used.

## Need help?

Contact the community in Kaspa Telegram groups or open an issue on [GitHub](https://github.com/dilljens/DagLock).
