# Findings: RestVerifier Testnet Test

## VPS Info (from /home/dillon/_code/VPS.md)

| Field | Value |
|-------|-------|
| **IP** | `40.160.241.74` |
| **SSH user** | `ubuntu` |
| **Password** | `raspi9000` |
| **Indexer service** | `daglock-indexer.service` → `:8443` |
| **Bot service** | `daglock-bot.service` |
| **Indexer path** | `/opt/daglock-indexer/` |
| **Bot path** | `/opt/daglock-bot/` |
| **SQLite DB** | `/opt/daglock-indexer/daglock.db` |
| **SSL** | Let's Encrypt, expires Sep 22 |
| **DNS** | `api.daglock.com` → Cloudflare proxied → VPS :443 → nginx → :8443 |

## Kaspa Testnet-11 REST API

- **URL:** `https://api-tn11.kaspa.org`
- **UTXO endpoint:** `GET /addresses/{address}/utxos` — returns array of UTXOs with outpoint info
- **Note:** Previously returned 503 on one check. May be intermittent.
- **Alternative:** Mainnet API at `https://api.kaspa.org` (more reliable)
- **Faucet:** `https://faucet-tn11.kaspanet.io/`

## RestVerifier Implementation (already built)

- **File:** `indexer/src/verification.rs` — `RestVerifier` struct
- **Config flag:** `--kaspa-api-url https://api.kaspa.org` (default)
- **Logic:** Queries `{api_url}/addresses/{address}/utxos`, matches outpoint by `transactionId` + `index`
- **Fallback:** If request fails, returns `Err(VerificationError)` → API returns 500
- **Address:** Queries `buyer_address` first, falls back to `seller_address`

## Known Risks

1. **api-tn11.kaspa.org stability** — It was returning 503 previously. The mainnet API at `api.kaspa.org` is more reliable but won't have testnet data.
2. **Address format** — Testnet addresses use `kaspatest:` prefix (not `kaspa:`). The REST API expects `kaspa:` addresses. This may be an issue — testnet addresses might not work with the mainnet endpoint.
3. **UTXO response format** — Need to verify the actual JSON structure matches what RestVerifier expects (`outpoint.transactionId`, `outpoint.index`, `utxoEntry.amount`).

## What to Watch For During Testing

- Startup log: `Using Kaspa REST API verifier at https://api-tn11.kaspa.org`
- UTXO check: `RestVerifier: checking UTXO for escrow esc_xxx (tx: xxx, output: 0)`
- Success: `RestVerifier: UTXO found for escrow esc_xxx — amount: 100000000`
- Not found: `RestVerifier: UTXO NOT found for escrow esc_xxx`
- Error: `RestVerifier: request failed for escrow esc_xxx: ...`
