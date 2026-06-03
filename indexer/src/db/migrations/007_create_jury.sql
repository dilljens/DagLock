-- Migration 007: Jury system for dispute resolution

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
    status TEXT NOT NULL DEFAULT 'selecting',
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

CREATE INDEX IF NOT EXISTS idx_jury_cases_status ON jury_cases(status);
CREATE INDEX IF NOT EXISTS idx_jury_cases_escrow ON jury_cases(escrow_id);
CREATE INDEX IF NOT EXISTS idx_jury_cases_created ON jury_cases(created_at);
