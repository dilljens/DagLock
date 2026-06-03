# Plan: Reputation v2 — Phases C (Jury) + D (Mediator Rep)

## Status
- ✅ Phase A: Recency weighting + UI breakdown — DONE
- ✅ Phase B: Vouching / Web of Trust — DONE
- ⏳ Phase C: Jury system — TODO
- ⏳ Phase D: Mediator reputation — TODO

---

## Phase C: Jury System (10-12 files)

### DB Migration `007_create_jury.sql`
```sql
CREATE TABLE IF NOT EXISTS juror_registrations (
    address TEXT PRIMARY KEY,
    registered_at INTEGER NOT NULL,
    total_cases_assigned INTEGER DEFAULT 0,
    total_cases_voted INTEGER DEFAULT 0,
    reliability_score REAL DEFAULT 1.0
);

CREATE TABLE IF NOT EXISTS jury_cases (
    id TEXT PRIMARY KEY,
    escrow_id TEXT NOT NULL REFERENCES escrows(id),
    status TEXT NOT NULL,
    juror_count INTEGER NOT NULL,
    threshold INTEGER NOT NULL,
    votes_for_seller INTEGER DEFAULT 0,
    votes_for_buyer INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    decided_at INTEGER,
    outcome TEXT
);

CREATE TABLE IF NOT EXISTS jury_votes (
    case_id TEXT NOT NULL REFERENCES jury_cases(id),
    juror_address TEXT NOT NULL,
    vote TEXT NOT NULL,
    voted_at INTEGER NOT NULL,
    reasoning TEXT,
    PRIMARY KEY (case_id, juror_address)
);
```

### Steps

1. Create `indexer/src/db/migrations/007_create_jury.sql`
2. Wire migration in `indexer/src/db/schema.rs`
3. Add jury types to `indexer/src/types.rs` (JurorRegistration, JuryCase, JuryVote, JuryRegisterRequest, JuryCaseStatus)
4. Add jury queries to `indexer/src/db/queries.rs`:
   - `register_juror()`, `unregister_juror()`, `get_juror()`, `list_eligible_jurors()`
   - `create_jury_case()`, `get_jury_case()`, `cast_vote()`, `check_jury_verdict()`, `list_active_jury_cases()`
5. Create `indexer/src/api/jury.rs`:
   - `POST /v1/jury/register` — opt in (check 10+ trades, 3.0+ score)
   - `POST /v1/jury/unregister` — opt out
   - `GET /v1/jury/cases` — list active cases for the caller (auth required)
   - `POST /v1/jury/cases/:id/vote` — cast vote (auth required, must be assigned juror)
   - `GET /v1/jury/cases/:id` — get case details with evidence
6. Wire routes in `indexer/src/api/mod.rs`
7. Update `indexer/src/api/escrows.rs` — when dispute mode is "jury", trigger jury case creation
8. Update `web/src/api.ts`: jury types + API methods
9. Update `web/src/App.tsx`: juror dashboard + vote panel
10. Update `web/src/styles.css`: jury UI styles
11. Verify: `cargo test --workspace` + `npm run build`

### Jury Selection Algorithm
- When escrow.value < 10K KAS: 3 jurors, threshold 2
- When 10K-100K KAS: 5 jurors, threshold 3
- When 100K+ KAS: 9 jurors, threshold 5
- Selection: score-weighted random from juror_registrations
- Selection filtering: exclude jurors with active cases on same escrow, prefer higher reliability_score

### Voting Rules
- Voting period: 72 hours from case creation
- Once threshold reached: case marked `decided`, outcome stored
- If 72h expires without threshold: defaults to `seller_wins` (prevents buyer fraud)
- Jurors can see evidence via existing GET /v1/escrows/:id/evidence

### Arbitration Key
- Indexer holds a dedicated hot keypair (stored in config/env at startup)
- When verdict reached, jury module signs a transaction with the arbiterKey
- The winning party countersigns (needs their signature too — covenant enforces 2-of-2)

---

## Phase D: Mediator Reputation (2-3 files)

### Steps
1. Add mediator stats to `Reputation` in `indexer/src/types.rs`:
   - `mediator_stats: Option<MediatorStats>` where MediatorStats has `disputes_mediated`, `rulings_accepted`, `acceptance_rate`, `years_active`
   - `mediator_score: Option<f64>`
2. Update `get_reputation()` in `indexer/src/db/queries.rs`:
   - Query escrows where address == mediator_key and status IN ('disputed', 'settled', 'refunded')
   - Count disputes where this address was mediator, count resolved ones
   - Calculate mediator_score: base = min(cases/10, 1)*5 + acceptance_rate*1.0 + min(years/2, 1)*0.5
3. Update `web/src/api.ts` Reputation type with mediator fields
4. Update `web/src/App.tsx` ReputationLookup to show mediator stats

---

## Risks (remaining)
- **Jury apathy** — mitigate with reliability score tracking
- **Jury selection timing** — cases created at dispute time, may stall if no jurors registered
- **Mediator data**: we already have mediator_key on escrows, just need to query it

## Verification
- `cargo test --workspace` — all pass
- `cd web && npm run build` — clean
- Manual: register jury → dispute escrow with mode:jury → vote → verify outcome

## Rollback
- `git revert <commits>` + `DROP TABLE juror_registrations, jury_cases, jury_votes`
