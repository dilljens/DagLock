# Testing & Debugging Improvement Plan

**Goal:** Make DagLock's testing suite production-grade — catch regressions before they ship, automate what's manual, and make debugging fast.

**Current:** 376 tests (293 Rust + 44 Web + 39 Bot). Good foundation, major gaps in integration and E2E.

---

## Priority Order

| Pri | Area | What | Why Now | Effort |
|-----|------|------|---------|--------|
| **P0** | Rust API tests | axum HTTP integration tests (currently only SQL-level) | No handler tests exist | 2-3d |
| **P0** | Time-paused tests | Start paused for expiry/auto-settle tests | Blockchain time bugs are common | 1d |
| **P0** | Insta snapshots | Snapshot API responses for regression | Cheap safety net for refactors | 1d |
| **P1** | State machine tests | proptest-state-machine for escrow lifecycle | Catches state transition bugs | 2-3d |
| **P1** | MSW frontend | Replace mockApi() with MSW handlers | Tests would exercise real fetch | 2-3d |
| **P1** | Tracing/OTLP | Structured logging + span hygiene | Debugging production issues | 1-2d |
| **P1** | Covenant fuzz | proptest-based covenant parameter fuzzing | Finds edge cases in contracts | 2-3d |
| **P2** | E2E wallet flow | Playwright + wallet harness | Manual KasWare test is slow | 2-3d |
| **P2** | Prometheus metrics | Business counters + histograms | Capacity planning | 1d |
| **P3** | cargo-nextest | Parallel test runner | Faster CI | 0.5d |
| **P3** | Benchmarks | Criterion for auth/verify/encrypt | Performance regression catch | 1d |

---

## Track A: Rust Backend Testing `[in-progress]`

### Phase A1: HTTP API Integration Tests `[ ]`
- [ ] Add `axum-test` based tests for key endpoints: escrows CRUD, offers CRUD, health
- [ ] Test error responses: 400, 401, 403, 404, 429, 500
- [ ] Test auth flow: valid sig, invalid sig, missing headers
- ✅ Checkpoint: `cargo test -p daglock-indexer -- api_tests` covers HTTP layer
- Depends on: nothing

### Phase A2: Insta Snapshot Tests `[ ]`
- [ ] Add `insta` dev-dependency
- [ ] Snapshot happy-path responses for 5 key endpoints
- [ ] Add redactions for timestamps/IDs
- ✅ Checkpoint: `cargo insta review` shows clean diffs
- Depends on: A1

### Phase A3: Time-Paused Tests `[ ]`
- [ ] Add time-paused tests for escrow expiry, mediation timeout, auto-settle
- [ ] Add concurrent-access test (two simultaneous settle attempts)
- ✅ Checkpoint: Expiry tests complete in milliseconds, not seconds
- Depends on: nothing

### Phase A4: State Machine Tests `[ ]`
- [ ] Add `proptest` + `proptest-state-machine` dev-deps
- [ ] Model escrow lifecycle as state machine (7 states, legal transitions)
- [ ] Generate random transition sequences and verify invariants
- ✅ Checkpoint: 1000+ random state sequences tested
- Depends on: nothing

## Track B: Frontend Testing `[]`

### Phase B1: MSW Migration `[ ]`
- [ ] Create `src/mocks/handlers.ts` covering all 20+ endpoints
- [ ] Set up MSW server in test setup
- [ ] Migrate existing tests from `mockApi()` to MSW
- 🚩 Checkpoint: All existing tests pass with MSW
- Depends on: nothing

## Track C: Monitoring `[]`

### Phase C1: Structured Logging `[ ]`
- [ ] Add `tracing` spans to all axum handlers
- [ ] Add JSON log output with `--log-format json` flag
- [ ] Add `TraceLayer` for auto-request instrumentation
- ✅ Checkpoint: `journalctl -u daglock-indexer --output=json` shows structured logs
- Depends on: nothing

### Phase C2: Prometheus Metrics `[ ]`
- [ ] Add business metric counters (escrows_created, settled, etc.)
- [ ] Add API latency histogram
- [ ] Expose `/metrics` endpoint
- ✅ Checkpoint: `curl /metrics` returns prometheus-formatted output
- Depends on: nothing

## Track D: Contract Testing `[]`

### Phase D1: Covenant Fuzz `[ ]`
- [ ] Add `proptest` to contract tests
- [ ] Add property-based tests for covenant invariants
- [ ] Fuzz constructor parameters (amounts, keys, timeouts)
- ✅ Checkpoint: Property tests pass with 10K+ random inputs
- Depends on: nothing
