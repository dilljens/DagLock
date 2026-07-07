CREATE TABLE IF NOT EXISTS blocked_users (
    id TEXT PRIMARY KEY,
    blocker_address TEXT NOT NULL,
    blocked_address TEXT NOT NULL,
    reason TEXT,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_blocked_users_blocker ON blocked_users(blocker_address);
CREATE INDEX IF NOT EXISTS idx_blocked_users_blocked ON blocked_users(blocked_address);
