# Linked Atomic Swaps — Implementation Plan

## Problem

Current flow requires manual secret sharing off-chain:
1. Alice generates secret → shares with Bob via Signal/Telegram
2. Bob submits secret to settle

This is fragile — if Alice doesn't share the secret, Bob can't complete the swap.

## Solution: Linked Swap Pairs

Two escrows share the same `trade_hash`. Either party can reveal the secret to settle their escrow. Once one settles, the other party can use the same secret to settle theirs.

## User Flow

### Setup (one-time)
1. Alice and Bob agree to trade KAS ↔ KRC-20
2. Alice clicks "Create Atomic Swap" in the UI
3. System generates a secret + hash
4. Alice creates escrow1 (locks KAS, hash stored)
5. Bob sees the escrow and creates escrow2 (locks KRC-20, same hash)
6. Both escrows are now linked by the shared hash

### Settlement
1. Alice reveals the secret to settle escrow1 (sends KRC-20 to Bob)
2. Bob uses the same secret to settle escrow2 (sends KAS to Alice)
3. Both escrows settle atomically

### If one party backs out
- If Alice never reveals the secret, Bob can't settle → timeout refund
- If Bob never reveals the secret, Alice can't settle → timeout refund
- Both parties get their funds back after timeout

## Database Changes

### New table: swap_pairs
```sql
CREATE TABLE swap_pairs (
    id TEXT PRIMARY KEY,
    escrow_id_1 TEXT NOT NULL,
    escrow_id_2 TEXT NOT NULL,
    trade_hash TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, partial, completed, expired
    created_at INTEGER NOT NULL,
    completed_at INTEGER
);
```

### Add to escrows table
```sql
ALTER TABLE escrows ADD COLUMN swap_pair_id TEXT;
```

## API Changes

### New endpoints
```
POST /v1/swaps/create       — Create a swap pair with two escrows
GET  /v1/swaps/:id          — Get swap pair status
POST /v1/swaps/:id/reveal   — Reveal secret to settle an escrow
```

### Modified endpoints
- `POST /v1/escrows` — Accept optional `swap_pair_id` field
- `GET /v1/escrows/:id` — Include swap pair info

## Frontend Changes

### New component: AtomicSwapForm
1. Input: "I want to sell X for Y"
2. System generates secret + hash
3. Shows both escrows (one locked, one pending)
4. Share link: `daglock.com/swap/<pair_id>`
5. Counterparty clicks link → sees both escrows → creates their escrow
6. Either party can reveal secret to settle

### Modified: SwapForm
- Show swap pair status
- Show which escrow is settled
- Show time remaining for timeout

## Security Considerations

1. **Secret storage** — Secret is stored in memory only, never in DB
2. **Hash verification** — SHA-256 of secret must match trade_hash
3. **Atomicity** — Both escrows must be settled or both timeout
4. **Front-running** — Secret is revealed only when settling, not before

## Implementation Order

| Step | Task | Effort |
|------|------|--------|
| 1 | Add `swap_pairs` table + `swap_pair_id` to escrows | 2h |
| 2 | Add swap pair API endpoints | 3h |
| 3 | Modify escrow creation to support swap pairs | 2h |
| 4 | Add AtomicSwapForm to web UI | 4h |
| 5 | Modify SwapForm for linked swaps | 2h |
| 6 | Add share link generation | 1h |
| 7 | Testing + edge cases | 3h |

**Total: ~2 days**

## Alternative: Simpler Approach

Instead of a full swap pair system, we could:
1. Just let users create two separate escrows with the same trade_hash
2. Add a "Link Swap" button that shows both escrows
3. Keep the existing swap endpoint

This is simpler but requires users to manually coordinate. The full swap pair system automates this.
