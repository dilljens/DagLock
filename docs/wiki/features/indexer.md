# indexer

Rust backend serving the DagLock REST API. Handles escrow lifecycle (create, settle, refund, dispute), offer board, reputation, vaults, jury, encrypted messaging, app registration, webhook dispatch, and WebSocket real-time updates. Uses SQLite or PostgreSQL via SQLx.

## Infrastructure

| Component | Location | Details |
|-----------|----------|---------|
| Indexer | Hetzner VPS CX23 (4GB RAM) | `testnet-12`, `--no-wrpc` (MockVerifier) |
| Bot | Same VPS | `@DagLock_bot` on Telegram |
| Web UI | Cloudflare Pages | `daglock.com` → `api.daglock.com` |
| kaspad | ❌ Not on VPS (too small) | Run locally on dev laptop |

## wRPC Status

Public Kaspa wRPC resolver nodes (`kaspa.stream/red/green/blue`) are down during wRPC v2 migration. Known working alternatives:

- **Mainnet:** `wss://troy.kaspa.stream/kaspa/mainnet/wrpc/borsh` (tested, stable, DAA progressing)
- **Testnet-12:** No public endpoint available. kaspad v2.0.1 doesn't support TN12 natively. Requires a [1-line patch](scripts/setup-laptop-kaspad.sh) to `consensus/core/src/config/params.rs`.

## VPS Hardening

Completed June 23, 2026:
- Service runs as `daglock` user (not root)
- `LimitNOFILE=65536` in systemd
- Toggle script at `/opt/daglock-indexer/toggle-wrpc.sh` for switching between `--no-wrpc` and `--wrpc-url`

---
*Confidence: 0.95 · Last updated: 6/23/2026*