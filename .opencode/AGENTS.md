# DagLock

Trustless escrow and atomic swap platform on the Kaspa blockchain.

## Project Rules

### Architecture
- Rust indexer (Axum HTTP + WebSocket, :8443) + React 18 frontend (Vite, TanStack Query, Radix)
- SQLite databases: `daglock.db`, `daglock_sim.db`
- WASM SDK compiles to WebAssembly for browser use

### SQLite
- All connections: `PRAGMA busy_timeout=5000`, `PRAGMA journal_mode=WAL`
- Write-heavy paths (block processing) should batch transactions

### Frontend API Calls
- Frontend talks to indexer API via Axum routes
- CORS config must match deployment target
- WebSocket reconnection logic required

### Pre-Deploy Checklist
1. `cargo build --release` — indexer compiles
2. SQLite PRAGMAs validated on target database
3. `npm run build` in `web/` — frontend builds
4. WASM SDK: `wasm-pack build --target web` in `wasm-sdk/`
5. Dockerfile + CORS config match deployment target
