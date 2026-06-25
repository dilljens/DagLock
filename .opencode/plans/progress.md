# Progress: Mainnet Readiness

## Session 2026-06-23 — Complete

### VPS Status
```
Network:  testnet-12
wRPC:     --no-wrpc (MockVerifier, offline mode)
User:     daglock (not root)
FD limit: 65536
Binary:   v0.1.0 release (with ICC S3 fix)
Hash:     KRC-20 = ae0946e4a9bd4a7585e6bf9135de38083cb11c85
```

### Services Running
- ✅ daglock-indexer (testnet-12, --no-wrpc)
- ✅ daglock-bot (Telegram)
- ✅ daglock-trade-bot (timer)

### Test Results
- ✅ Rust: 241 tests pass
- ✅ Web: 40/40 tests, build succeeds
- ✅ Bot: 22/22 tests
- ✅ Sentrux: no degradation

### What Was Done This Session
1. **A1/A2**: S3 fix — ICC pattern in `daglock_krc20.sil` (KCC-20 input ownership validation)
2. **B1-B3**: Code quality — no `.unwrap()`, shared fee constant, flaky tests fixed
3. **D1**: VPS hardened — daglock user, LimitNOFILE=65536
4. **D2**: Release binary built and deployed
5. **D3**: `deploy-mainnet.sh` updated (dual testnet/mainnet mode)
6. **D5**: Mainnet wRPC found (`troy.kaspa.stream`, borsh) — documented for future use

### Remaining for Mainnet (June 30)
- Generate treasury pubkey
- Find a working testnet-12 wRPC endpoint (or run kaspad on a bigger machine)
- Web onboarding modal (U7) — low priority
- TradeHash newtype (B4/Q4) — low priority
- Launch prep: demo video, announcements, monitoring
