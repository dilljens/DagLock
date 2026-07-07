CREATE TABLE IF NOT EXISTS user_reports (
    id TEXT PRIMARY KEY,
    reporter_address TEXT NOT NULL,
    reported_address TEXT NOT NULL,
    escrow_id TEXT,
    reason TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_reports_reported ON user_reports(reported_address);
