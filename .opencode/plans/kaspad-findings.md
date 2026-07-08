# Findings: Kaspad Deployment

## Current State (June 25, 2026)

### VPS Specs (OVH VPS-2)
| Resource | Value |
|----------|-------|
| Provider | OVHcloud US |
| Plan | VPS-2 (2027 range) |
| vCores | 4 |
| RAM | 8 GB (2.1 GB used, 5.6 GB available) |
| Disk | 72 GB (59 GB available) |
| OS | Ubuntu 26.04 LTS |
| glibc | 2.43 |

**Key observation:** The AGENTS.md incorrectly referenced a "OVHcloud VPS with 2GB RAM" as the VPS. The actual VPS is OVH with 8GB RAM — plenty for kaspad testnet-12.

### Current daglock-indexer service
- Runs as `daglock` user
- `/etc/systemd/system/daglock-indexer.service` with `--no-wrpc`
- Listens on `127.0.0.1:8443`
- Proxy through nginx at `api.daglock.com`

### kaspad not installed
- No binary, no service, no data directory

## kaspad Technical Details

### Binary
- Source: `rusty-kaspa` tag `v2.0.1` (matches indexer dependency)
- Build location: `/tmp/rusty-kaspa/target/release/kaspad` (42 MB, stripped ELF x86-64)
- Build time: ~7:49 min
- glibc compatibility: Both local and VPS have glibc 2.43 — compatible

### Port Map (testnet-12, v2.0.1)
| Protocol | Port | Flag |
|----------|------|------|
| Borsh wRPC | 17210 | `--rpclisten-borsh` |
| JSON gRPC | 16210 | `--rpclisten` (default_rpc_port) |

Source: `consensus/core/src/network.rs`:
- `default_borsh_rpc_port()`: Testnet → 17210
- `default_rpc_port()`: Testnet → 16210

### Required flags
```
--testnet                           # picks testnet-12 in v2.0.1
--rpclisten-borsh=127.0.0.1:17210   # Borsh wRPC for indexer (localhost only)
--utxoindex                         # enables UTXO index for get_utxos_by_addresses
--max-tracked-addresses 1000        # makes UTXO index actually track addresses
```

Note: `--utxoindex` without `--max-tracked-addresses > 0` is a no-op for address tracking (default is 0). The WrpcVerifier uses `get_utxos_by_addresses()` which needs address tracking enabled.

### Sync expectations
- Testnet-12 IBD: ~1-2 hours from scratch
- wRPC endpoint available during sync (just won't have full UTXO data until synced)
- Data directory: default `~/kaspad/data/testnet-12/`
- Database format: RocksDB (not raw flat files as some assume)

### Indexer connection flow (from main.rs)
```
1. --wrpc-url provided → try_connect_wrpc() with Borsh encoding
2. --no-wrpc set → MockVerifier (current state)
3. Neither → try_connect_resolver() with JSON encoding (public resolver network)
4. Any failure → fall back to MockVerifier gracefully
```

## Existing Pre-Announcement Plan

File: `.opencode/plans/pre-announcement.md`
- Written for old OVHcloud VPS (`40.160.241.74`)
- References MockVerifier in demo video script
- Dated June 2026, pre-Toccata
- Will need updating after kaspad deployment

## Decisions
| Decision | Choice | Rationale |
|----------|--------|-----------|
| Deploy binary vs build on VPS | SCP pre-built binary | Faster (already built), same glibc |
| kaspad user | `kaspad` system user | Non-root, matches `daglock` pattern |
| Port exposure | 127.0.0.1 only | Indexer is on same host, no external access needed |
| Max tracked addresses | 1000 | Generous for testnet, minimal memory cost |
| Sync strategy | Background async | Deploy first, wire indexer after sync complete |
