-- Migration 004: Dispute evidence table and mediator/dispute outcome columns

CREATE TABLE IF NOT EXISTS dispute_evidence (
    id TEXT PRIMARY KEY,
    escrow_id TEXT NOT NULL REFERENCES escrows(id),
    submitted_by TEXT NOT NULL,
    content TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    signed_message TEXT,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_dispute_evidence_escrow_id ON dispute_evidence(escrow_id);
