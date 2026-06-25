# Bot & Exchange Labeling for Reputation

## Goal
Allow addresses to self-label as `bot` or `exchange`. Apply different reputation score formulas based on label. Display the label publicly in API + web UI.

## Design Decisions (Pre-Resolved)

| Question | Decision |
|----------|----------|
| Who can label? | Self-label only — sign with the address's key |
| On-chain label? | DB-only for now (cheap, fast, reversible). On-chain later if demanded. |
| Bot-specific UI? | Simple filter toggle on offer board: "Show bot offers" (default on) |
| Exchange label? | Yes — same mechanism, different label value |

## Label Set

| Label | Score algo | Display |
|-------|-----------|---------|
| `human` (unlabeled default) | Beta reputation (current) | No badge |
| `bot` | Bot-adjusted formula | 🤖 Bot badge |
| `exchange` | Human formula | 🏦 Exchange badge |

---

## Phase 1: Database (timebox: 30min)

### New migration: `011_address_labels.sql`

```sql
CREATE TABLE address_labels (
    address TEXT NOT NULL PRIMARY KEY,
    label TEXT NOT NULL CHECK(label IN ('bot', 'exchange')),
    memo TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);
```

No `human` label stored — absence of a label implies human.

### New query functions in `indexer/src/db/queries/reputation.rs`:

```rust
pub async fn set_address_label(
    pool: &Pool<Sqlite>,
    address: &str,
    label: &str,
    memo: Option<&str>,
) -> Result<(), DbError> { ... }

pub async fn get_address_label(
    pool: &Pool<Sqlite>,
    address: &str,
) -> Result<Option<String>, DbError> { ... }
```

---

## Phase 2: Reputation Algorithm (timebox: 1h)

### Current formula in `calculate_reputation_score()`:
```
score = beta_core + recency_bonus + volume_bonus + age_bonus
clamped to [1.0, 5.0]
```

### Bot-adjusted formula:

| Component | Human | Bot | Rationale |
|-----------|-------|-----|----------|
| Beta core (success rate) | `(α+1)/(α+β+2)` | Same | Both need trustworthiness |
| Recency bonus | Last 90d ×2 | **Remove** | Bots do 1000 trades/week — recency distorts |
| Volume bonus | `ln(vol/1000+1)×0.12` | **`ln(vol/10000+1)×0.06`** | Bots inflate volume naturally |
| Age bonus | `min(days/365,2)×0.05` | **Remove** | Bot could be 1 day old and legitimate |
| Operating time bonus | None | **`min(hours_since_first/1000,1)×0.3`** | Sustained uptime proves reliability |
| Clamp | [1.0, 5.0] | [1.0, 5.0] | Same range |

### Implementation:

```rust
pub async fn calculate_reputation_score(
    pool: &Pool<Sqlite>,
    address: &str,
    label: &Option<String>,
    stats: &ReputationStats,
) -> f64 {
    let beta = (stats.settled as f64 + 1.0) / ((stats.settled + stats.refunded) as f64 + 2.0);
    
    match label.as_deref() {
        Some("bot") => {
            // Bot formula
            let volume_bonus = (stats.total_volume_sompi as f64 / 1e8 / 10_000.0 + 1.0).ln() * 0.06;
            let operating_time = stats.hours_since_first_trade.min(1000) as f64 / 1000.0 * 0.3;
            (beta + volume_bonus + operating_time).clamp(1.0, 5.0)
        }
        _ => {
            // Human formula (current)
            let recency_bonus = ...existing...;
            let volume_bonus = (stats.total_volume_sompi as f64 / 1e8 / 1000.0 + 1.0).ln() * 0.12;
            let age_bonus = (stats.age_days as f64 / 365.0).min(2.0) * 0.05;
            (beta + recency_bonus + volume_bonus + age_bonus).clamp(1.0, 5.0)
        }
    }
}
```

---

## Phase 3: API Changes (timebox: 1h)

### New endpoint: `PUT /v1/reputation/:address/label`

```rust
#[derive(Deserialize)]
pub struct LabelRequest {
    pub label: String,       // "bot" | "exchange"
    pub memo: Option<String>,
    pub signature: String,   // Schnorr signature of the message
    pub message: String,     // "label:bot:kaspa:..."
}

// Handler:
pub async fn set_label(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Json(req): Json<LabelRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 1. Verify signature matches address
    let sig_verifier = &state.sig_verifier;
    let auth = AuthHeaders {
        address: address.clone(),
        signature: req.signature,
        message: req.message,
    };
    sig_verifier.verify(&auth).await.map_err(|_| unauthorized!("Invalid signature"))?;
    
    // 2. Validate label value
    if req.label != "bot" && req.label != "exchange" {
        return Err(bad_request!("Label must be 'bot' or 'exchange'"));
    }
    
    // 3. Store label
    queries::set_address_label(&state.db, &address, &req.label, req.memo.as_deref()).await
        .map_err(|_| internal_error!("Failed to set label"))?;
    
    // 4. Return updated reputation
    let rep = queries::get_reputation(&state.db, &address).await
        .map_err(|_| internal_error!("Failed to fetch reputation"))?;
    Ok(Json(json!(rep)))
}
```

### Modified `Reputation` struct (`types.rs`):

```rust
#[derive(Serialize)]
pub struct Reputation {
    // ...existing fields...
    pub label: Option<String>,        // NEW: "bot", "exchange"
    pub label_memo: Option<String>,   // NEW: free-text
}
```

---

## Phase 4: Frontend Changes (timebox: 2h)

### ReputationPage.tsx — Add badge display

```tsx
// Next to the address:
{rep.label === "bot" && <span className="badge badge--bot">🤖 Bot</span>}
{rep.label === "exchange" && <span className="badge badge--exchange">🏦 Exchange</span>}

// Add a "Label this address" section:
<Panel title="Label this address">
  <p>Set an identity label to help others know who they're trading with.</p>
  <button onClick={}>Label as Bot</button>
  <button onClick={}>Label as Exchange</button>
</Panel>
```

### Offer board — filter toggle

```tsx
<label>
  <input type="checkbox" checked={showBots} onChange={...} />
  Show offers from bots
</label>
```

---

## Phase 5: Verification (timebox: 30min)

### Playwright tests to add:

- `reputation-label.spec.ts`
  - Lookup own reputation → no label
  - Label self as bot with valid signature → label appears
  - Label self with wrong signature → 401
  - Label another address (can't sign) → 401
  - Bot score differs from human with same trade data

---

## Implementation Priority

**Do this before mainnet announcements?** Low priority. The reputation system works without labels. This is a **Phase 2 (after traction)** item — build it when users ask "how do I know if I'm trading with a human?"

**Total effort:** ~4-5 hours once prioritized
