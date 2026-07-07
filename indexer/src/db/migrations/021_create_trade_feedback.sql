CREATE TABLE IF NOT EXISTS trade_feedback (
    id TEXT PRIMARY KEY,
    escrow_id TEXT NOT NULL REFERENCES escrows(id),
    reviewer_address TEXT NOT NULL,
    rating INTEGER NOT NULL CHECK(rating >= 1 AND rating <= 5),
    comment TEXT,
    created_at INTEGER NOT NULL,
    UNIQUE(escrow_id, reviewer_address)
);
CREATE INDEX IF NOT EXISTS idx_feedback_escrow ON trade_feedback(escrow_id);
CREATE INDEX IF NOT EXISTS idx_feedback_reviewer ON trade_feedback(reviewer_address);
