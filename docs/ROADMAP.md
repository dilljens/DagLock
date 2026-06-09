# DagLock Development Roadmap

> Updated June 9, 2026. Post-audit fixes complete. Pre-mainnet sprint in progress.

**Toccata Mainnet Hard Fork:** Activates at DAA score 474,165,565 (~June 30, 16:15 UTC).
**DagLock mainnet launch target:** June 30, 2026 (same day as Toccata).

---

## Current State

**168 tests pass across 6 crateworks (Rust + web).**
**CI: green** — zero warnings, all checks pass.
**Production: live** at `daglock-production.up.railway.app` (template hashes visible, offline/MockVerifier mode).
**Security audit:** All 7 critical/high findings fixed.

---

## Phase 0: Covenants  (Done — 6 contracts)

| Covenant | File | Entrypoints | Fee | Status |
|----------|------|-------------|-----|--------|
| KAS escrow | `daglock.sil` | release, swap, refund | 0.5% treasury | Done |
| KRC-20 escrow | `daglock_krc20.sil` | release, swap, refund | 0.5% treasury | Done (S2 fixed) |
| Arbiter (mediator/jury) | `daglock_arbiter.sil` | release, swap, refundAfterTimeout, dispute paths | 0.5% treasury | Done |
| Vault (standard) | `daglock_vault.sil` | withdraw | None | Done |
| Vault Softlock | `daglock_vault_softlock.sil` | withdrawPassword, withdrawTimeout | None | Done |
| Vault Multisig | `daglock_vault_multisig.sil` | withdraw (up to 3-of-3) | None | Done |

### Template Hashes
| Covenant | Hash |
|----------|------|
| KAS | `30876e3ea42d0e23bb0980f3fd97ae8807e9c70f` |
| Arbiter | `c6d10350b51d5fedcc05382d02d8334a783be220` |
| KRC-20 | `8a43a8438d183a92bc7b94337c031196ff16725b` |
| Vault | `d773d10a9a2626986226e4eca528e0cb071b79be` |
| Softlock | `dd2d3699db1332bb21fcc31ef3971963e8735b16` |
| Multisig | (run `cargo test -p daglock-contracts print_template_hashes`) |

---

## Phase 1: Security & Audit Fixes  (Done)

| ID | Issue | Fix | Status |
|----|-------|-----|--------|
| S1 | MockVerifier in production | Async WrpcVerifier with real get_utxos_by_addresses() | Done |
| S2 | KRC-20 fee boolean-only | Exact check: outputs[1].value == inputValue | Done |
| S3 | KCC-20 ownership validation | Closed — multi-sig design prevents | Closed |
| S4 | trade_hash unvalidated | daglock_shared::validate_trade_hash() on create | Done |
| S5 | No replay protection | action:id:ts:nonce format, DB-backed nonce store | Done |
| S6 | Bot plaintext addresses | AES-256-GCM encryption | Done |
| S7 | Docker root | Non-root daglock user | Done |

---

## Phase 2: Indexer API  (Done — 30+ endpoints)

### Integrator API (New June 8-9)
| Feature | Status |
|---------|--------|
| `POST /v1/apps/register` — API key registration | Done |
| `/v1/apps/:id/keys` — key management CRUD | Done |
| `/v1/apps/:id/webhooks` — 8 event types, 3x retry | Done |
| `GET /v1/openapi.json` — OpenAPI 3.1 spec | Done |
| `GET /v1/status` — public status page | Done |

### Outgoing Integrations
| Package | Status |
|---------|--------|
| `@daglock/sdk` — TypeScript client (npm) | Done |
| `@daglock/widget` — `<daglock-escrow>` custom element | Done |

---

## Phase 3: Web UI  (Rewritten June 6-9)

- Hash-based router, sidebar nav (240px, mobile drawer)
- WalletContext — connected address auto-fills everywhere
- Toast notifications, emoji-free design
- 6 pages: Dashboard, Offers, Escrows, Vaults, Reputation, Jury

---

## Phase 4: Telegram Bot  (Running)

16 commands, trade link deep links, user address encryption, API retry/backoff.

**Pending:** Native `/create` wizard (redirects to web).

---

## Phase 5: CLI  (Functional)

7 command modules, unsigned tx assembly.
**Pending:** Real wallet integration with `kaspawallet sign`.

---

## Phase 6: Remaining Before Mainnet (June 30)

### Important
| Task | Effort |
|------|--------|
| Testnet deploy with real wRPC node | 2-3h |
| CLI wallet integration (U1, U3) | 1 day |
| Bot /create wizard (U4) | 1 day |

### Nice-to-Have
| Task | Effort |
|------|--------|
| Split queries.rs | 1-2 days |
| Lifecycle integration tests | 1 day |
| Structured API errors | 2h |
| CoinGecko caching | 2h |

### Deferable
Atomic swap wizard UI, price oracle, cross-chain BTC.

---

## Phase 7: Post-Launch (Q3 2026+)

Analytics dashboard, volume-based fee rebates, hardware wallet support (see `docs/WALLET.md`).

---

## Audit Reference

Full audit findings: `docs/wiki/_index.md#audit-log`
Wallet build plan: `docs/WALLET.md`
