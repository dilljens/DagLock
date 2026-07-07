# Plan: KRC-20 Token Launchpad (P4)

**Goal:** Let anyone create a KRC-20 token in a few clicks via the DagLock web UI. Like pump.fun for Kaspa. Every new token is a potential DagLock escrow user.

**Effort:** 3-4 days

**Why this matters:** KRC-20 tokens launched with Toccata (today). There is NO easy way to create a token — users need to write a SilverScript covenant manually. DagLock has the compile API, the indexer infrastructure, and a web UI. If DagLock is the easiest place to create a token, it gets first access to every project launching on Kaspa.

---

## Architecture

```
User fills form (name, ticker, supply, etc.)
  │
  ├── POST /v1/compile → returns covenant script + template hash
  ├── POST /v1/tokens/deploy → creates token record in indexer
  └── User broadcasts the covenant TX → token is live
       └── Token appears on /tokens dashboard automatically
```

---

## Phase A: Token deploy API `[ ]`
**⏱ Timebox:** 1 day

- [ ] The `/v1/compile` endpoint already exists — wire it to a "deploy token" flow
- [ ] Create `POST /v1/tokens/deploy`:
  - Accepts: `name`, `ticker`, `total_supply`, `decimals`, `owner_address`, `mint_mode` (fixed | mintable | burnable)
  - Compiles a KRC-20 minter covenant with the given parameters
  - Returns: `script_hex`, `template_hash`, `covenant_address`, `token_id`
- [ ] Create `GET /v1/tokens/:ticker/deploy-status` — returns whether the token TX has been broadcast
- [ ] Register the token in the indexer's internal token registry so it appears on `/v1/tokens` immediately after creation (not just after an escrow trade)

**Token types to support:**
| Type | Description | Compile params |
|------|-------------|----------------|
| Fixed supply | All tokens minted at deploy | `total_supply` |
| Mintable | Owner can mint more later | `total_supply`, `owner_key` |
| Burnable | Holders can burn tokens | `total_supply` |

**✅ Checkpoint:** `curl POST /v1/tokens/deploy -d '{"ticker":"TEST","total_supply":"1000000"}'` returns a covenant address.

---

## Phase B: Web UI token creator `[ ]`
**⏱ Timebox:** 1 day

- [ ] New page at `/tokens/create`:
  - Step 1: Token details — name, ticker, supply, decimals (default 8)
  - Step 2: Tokenomics — mint mode, optional premine %, optional team allocation
  - Step 3: Review — show summary, network fee estimate, covenant address
  - Step 4: Deploy — "Broadcast with KasWare" button → opens wallet → confirm
  - Step 5: Success — link to token page on /tokens/:ticker, shareable link
- [ ] Form validation:
  - Ticker: 3-8 uppercase alphanumeric
  - Supply: max 1 trillion (10^12)
  - Name: 2-32 characters
  - Fee display: show network cost to deploy
- [ ] Pre-filled templates for popular use cases:
  - "Standard KRC-20" (fixed, 1B supply, 8 decimals)
  - "Community token" (mintable, 100M supply, 8 decimals)
  - "Memecoin" (fixed, 1T supply, 8 decimals, max hype)

**✅ Checkpoint:** Navigate to `/tokens/create`, fill form, deploy → token appears on `/tokens`.

---

## Phase C: Token management dashboard `[ ]`
**⏱ Timebox:** 1 day

- [ ] Token deployer dashboard at `/tokens/manage`:
  - List of tokens you've deployed
  - For each: ticker, supply minted, holders count, escrow volume
  - Actions: "Create escrow for this token", "View on explorer", "Edit metadata"
- [ ] Token metadata:
  - Logo/icon upload (stored in IPFS or R2)
  - Description
  - Website URL
  - Social links (Telegram, Twitter, Discord)
- [ ] Metadata API: `PATCH /v1/tokens/:ticker/metadata` (with deployer auth)

**✅ Checkpoint:** Deployer sees their token in a management dashboard with metadata editing.

---

## Phase D: Token + Escrow integration `[ ]`
**⏱ Timebox:** 1 day

- [ ] After deployment, show "Bootstrap Liquidity" CTA:
  - "Create a buy offer for your token" → pre-filled offer form
  - "Create a sell offer" → pre-filled offer form
  - "Share your token" → Telegram card image + invite link
- [ ] Token launch event: when a new token is created, WebSocket broadcast → dashboard updates
- [ ] Token badges on escrow and offer cards (already partially done)

**✅ Checkpoint:** Deploy a token → immediately create a buy offer for it → token appears on offer board.

---

## Phase E: Tests `[ ]`
**⏱ Timebox:** 4h

- [ ] API tests: deploy token → token appears in /v1/tokens list
- [ ] API tests: invalid ticker (too short/long) → 400
- [ ] API tests: duplicate ticker → 409
- [ ] Web tests: form renders all 4 steps
- [ ] Web tests: validation errors show correctly

**✅ Checkpoint:** All tests pass.

---

## Files Changed / Created

| File | Change |
|------|--------|
| `indexer/src/api/tokens.rs` | Add `POST /v1/tokens/deploy`, `PATCH /v1/tokens/:ticker/metadata` |
| `indexer/src/db/migrations/025_token_registry.sql` | **New** — KRC-20 token registry table |
| `indexer/src/db/queries/tokens.rs` | Add deploy + metadata queries |
| `indexer/src/db/schema.rs` | Add migration |
| `indexer/src/types.rs` | Add token deploy types |
| `web/src/pages/CreateTokenPage.tsx` | **New** — 5-step token creation wizard |
| `web/src/pages/ManageTokensPage.tsx` | **New** — deployer dashboard |
| `web/src/pages/TokenDetailPage.tsx` | Add metadata display |
| `web/src/App.tsx` | Add `/tokens/create`, `/tokens/manage` routes |
| `web/src/router.tsx` | Add new route types |
| `web/src/layout/Sidebar.tsx` | Add "Create Token" link |

## Token Registry Schema

```sql
CREATE TABLE token_registry (
    id TEXT PRIMARY KEY,
    ticker TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    total_supply INTEGER NOT NULL,
    decimals INTEGER NOT NULL DEFAULT 8,
    mint_mode TEXT NOT NULL DEFAULT 'fixed',
    owner_address TEXT,
    covenant_address TEXT,
    template_hash BLOB,
    metadata_json TEXT,          -- JSON: logo, description, website, socials
    deploy_tx_id TEXT,
    status TEXT NOT NULL DEFAULT 'pending',  -- pending | active | failed
    created_at INTEGER NOT NULL,
    deployed_at INTEGER
);
```

## Edge Cases

| Case | Handling |
|------|----------|
| Ticker already exists | 409 Conflict — "NACHO is already taken" |
| Broadcast fails (network error) | Token stays `pending`. User can retry broadcast |
| User navigates away before broadcasting | Token stays `pending`. Resume from dashboard |
| Supply overflow (> u64 max) | Validate: max 10^12 tokens |
| Invalid decimals (> 18) | Cap at 18 (Kaspa KIP standard) |
| User creates duplicate token manually | Indexer detects via template hash + owner |
