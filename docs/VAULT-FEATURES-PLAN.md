# Vault Features Implementation Plan

## Overview

Expand DagLock's vault functionality from basic time-locked vaults to a full suite of programmable vaults. All vaults are implemented as SilverScript covenants on Kaspa L1.

---

## Current Status

### Phase 1: Vault Status & Withdrawal ✅ COMPLETE
- [x] Added 009_create_vaults.sql migration
- [x] Added Vault, VaultType, VaultStatus types
- [x] Added vault queries (insert, get, list, update)
- [x] Added vault API endpoints (GET /v1/vaults, POST /v1/vaults, POST /v1/vaults/:id/withdraw)
- [x] Added VaultListPanel component

### Phase 2: Vault UI Improvements ✅ COMPLETE
- [x] Added VaultStatusPanel component (shows lock/expiry state, withdraw button)
- [x] Added VaultLookup component (look up vault by ID)
- [x] Updated CreateVaultForm to use Kaspa address and amount
- [x] Create vault entry in database via API

---

## Future Phases (Planned)

### Phase 3: Beneficiary Vault
**Status**: 📋 Planned  
**Complexity**: Medium

**What it does:**
- Lock KAS with a beneficiary address
- After timeout, funds auto-send to beneficiary
- Owner can withdraw before timeout (cancels the transfer)

**Use cases:**
- Conditional gifts ("if I don't claim, send to charity")
- Inheritance planning
- Escrow fallback

**Covenant changes:**
- Add `beneficiary_key` constructor param
- Add `transfer` entrypoint (auto-sends after timeout)
- Owner can still `withdraw` before timeout

**Web UI additions:**
- Beneficiary address field in CreateVaultForm
- Vault status panel showing lock/expiry info
- Withdraw button (owner only, before timeout)
- Cancel transfer button (owner only, before timeout)

---

### 3. Deadman Switch Vault
**Status**: 📋 Planned  
**Complexity**: High

**What it does:**
- Lock KAS with a check-in requirement
- Owner must sign a message every X days
- If owner fails to check in, funds return to beneficiary

**Use cases:**
- Emergency fund release
- Long-term storage with safety net
- Inheritance with proof-of-life

**Covenant changes:**
- Add `beneficiary_key` and `check_interval` params
- Add `check_in` entrypoint (resets timer)
- Auto-refund to beneficiary if timer expires

**Web UI additions:**
- Check-in interval selector (7d, 30d, 90d, 365d)
- Last check-in timestamp display
- Check-in button (owner only)
- Time remaining until auto-refund

**Backend additions:**
- Check-in tracking (API endpoint)
- Timer state stored in covenant

---

### 4. Inheritance Vault
**Status**: 📋 Planned  
**Complexity**: High

**What it does:**
- Lock KAS for beneficiary
- Owner sets a "dead man's switch" timeout
- If owner doesn't check in, beneficiary can claim after grace period
- Owner can always withdraw before timeout

**Use cases:**
- Estate planning
- Long-term savings with fallback
- Trust fund mechanics

**Covenant changes:**
- Add `beneficiary_key`, `timeout`, `grace_period` params
- Add `claim` entrypoint (beneficiary after timeout + grace)
- Owner can `withdraw` anytime before timeout

**Web UI additions:**
- Beneficiary address field
- Timeout + grace period selectors
- Claim button (beneficiary only, after conditions met)
- Withdraw button (owner only, before timeout)

---

### 5. Multi-Sig Vault
**Status**: 📋 Planned  
**Complexity**: Very High

**What it does:**
- Lock KAS requiring N-of-M signatures to unlock
- Configurable quorum (e.g., 2-of-3, 3-of-5)
- Time-locked fallback (all signers can withdraw after timeout)

**Use cases:**
- Joint accounts
- Corporate treasury
- Escrow with multiple parties

**Covenant changes:**
- Add `signers` array and `threshold` param
- Add `spend` entrypoint (requires N signatures)
- Add timeout fallback for all signers

**Web UI additions:**
- Multi-signer address input (add/remove)
- Threshold selector (N of M)
- Signature collection interface
- Time remaining until timeout fallback

---

## Implementation Phases

### Phase 1: Vault Status & Withdrawal (1-2 days)
**Goal**: Make existing vaults usable

- [ ] Add `GET /v1/vaults` endpoint (list vaults by owner)
- [ ] Add `GET /v1/vaults/:id` endpoint (vault details)
- [ ] Add `POST /v1/vaults/:id/withdraw` endpoint
- [ ] Add VaultListPanel component to web UI
- [ ] Add VaultStatusPanel component (shows lock/expiry state)
- [ ] Add WithdrawButton component (owner only, before timeout)

### Phase 2: Beneficiary Vault (2-3 days)
**Goal**: Add beneficiary support

- [ ] Extend `daglock_vault.sil` with beneficiary param
- [ ] Add `transfer` entrypoint to covenant
- [ ] Update CreateVaultForm with beneficiary field
- [ ] Add beneficiary claim UI
- [ ] Add cancel transfer UI (owner only)

### Phase 3: Deadman Switch (3-4 days)
**Goal**: Add check-in mechanism

- [ ] Extend covenant with check-in tracking
- [ ] Add `POST /v1/vaults/:id/check-in` endpoint
- [ ] Add check-in UI with timer display
- [ ] Add auto-refund monitoring
- [ ] Add beneficiary notification system

### Phase 4: Inheritance Vault (3-4 days)
**Goal**: Add inheritance mechanics

- [ ] Extend covenant with grace period
- [ ] Add `claim` entrypoint (beneficiary)
- [ ] Add claim UI with verification
- [ ] Add owner withdrawal UI
- [ ] Add grace period tracking

### Phase 5: Multi-Sig Vault (5-7 days)
**Goal**: Add multi-signature support

- [ ] Design multi-sig covenant architecture
- [ ] Implement signature verification in covenant
- [ ] Add signature collection API
- [ ] Add multi-sig UI with signature status
- [ ] Add timeout fallback mechanism

---

## Technical Details

### Covenant Architecture

All vaults extend the base `daglock_vault.sil` covenant with additional constructor parameters and entrypoints.

**Constructor parameters:**
- `owner_key`: 32-byte public key (owner)
- `timeout`: Unix timestamp (lock expiry)
- `beneficiary_key`: 32-byte public key (optional, for beneficiary vaults)
- `check_interval`: Seconds between check-ins (deadman switch)
- `signers`: Array of 32-byte public keys (multi-sig)
- `threshold`: Number of required signatures (multi-sig)

**Entrypoints:**
- `withdraw`: Owner withdraws after timeout
- `transfer`: Auto-send to beneficiary (beneficiary vault)
- `check_in`: Reset deadman timer (deadman switch)
- `claim`: Beneficiary claims after timeout + grace (inheritance)
- `spend`: Multi-sig unlock (multi-sig)

### Database Schema

```sql
CREATE TABLE vaults (
    id TEXT PRIMARY KEY,
    owner_address TEXT NOT NULL,
    beneficiary_address TEXT,
    vault_type TEXT NOT NULL,  -- 'time', 'beneficiary', 'deadman', 'inheritance', 'multisig'
    status TEXT NOT NULL,      -- 'locked', 'unlocked', 'expired', 'transferred'
    amount_sompi INTEGER NOT NULL,
    timeout INTEGER NOT NULL,
    check_interval INTEGER,
    last_check_in INTEGER,
    grace_period INTEGER,
    signers TEXT,  -- JSON array of addresses
    threshold INTEGER,
    lock_tx_id TEXT,
    created_at INTEGER NOT NULL,
    unlocked_at INTEGER,
    expires_at INTEGER
);
```

### API Endpoints

```
GET    /v1/vaults                    -- List vaults by owner
GET    /v1/vaults/:id                -- Get vault details
POST   /v1/vaults                    -- Create vault
POST   /v1/vaults/:id/withdraw       -- Withdraw from vault
POST   /v1/vaults/:id/transfer       -- Transfer to beneficiary
POST   /v1/vaults/:id/check-in       -- Check in (deadman switch)
POST   /v1/vaults/:id/claim          -- Beneficiary claim
POST   /v1/vaults/:id/sign           -- Add signature (multi-sig)
```

---

## Web UI Components

### VaultListPanel
- List all vaults for an address
- Show vault type, status, amount, expiry
- Filter by status (locked/unlocked/expired)

### VaultStatusPanel
- Detailed vault info
- Time remaining display
- Owner/beneficiary addresses
- Current status with visual indicator

### CreateVaultForm (Enhanced)
- Vault type selector
- Amount input
- Duration/timeout selector
- Beneficiary address (for beneficiary/inheritance)
- Check-in interval (for deadman switch)
- Multi-sig signer list (for multi-sig)

### WithdrawButton
- Owner only
- Disabled if not unlocked
- Confirmation dialog

### ClaimButton
- Beneficiary only
- Disabled if conditions not met
- Verification required

### CheckInButton
- Owner only
- Shows time since last check-in
- Disabled if already checked in recently

---

## Testing Strategy

### Unit Tests
- Covenant compilation for each vault type
- Entrypoint verification
- Timeout logic
- Multi-sig threshold validation

### Integration Tests
- Full vault lifecycle (create → lock → withdraw/transfer)
- Beneficiary claim flow
- Deadman switch check-in and auto-refund
- Multi-sig signature collection

### E2E Tests
- Web UI vault creation
- Vault status display
- Withdrawal flow
- Beneficiary claim flow

---

## Security Considerations

1. **Covenant enforcement**: All rules enforced on-chain, no trusted intermediaries
2. **Timeout validation**: Ensure timeouts are reasonable (min 1 hour, max 10 years)
3. **Multi-sig thresholds**: Validate N <= M, M >= 2
4. **Check-in windows**: Prevent gaming by requiring minimum interval
5. **Beneficiary verification**: Ensure beneficiary addresses are valid Kaspa addresses

---

## Success Metrics

- [ ] All vault types compile and execute correctly
- [ ] Web UI shows vault status in real-time
- [ ] Withdrawal/claim flows work without errors
- [ ] Deadman switch auto-refunds correctly
- [ ] Multi-sig threshold enforcement works
- [ ] Gas fees are reasonable for all operations
