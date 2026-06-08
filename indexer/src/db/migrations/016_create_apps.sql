-- API apps + keys for integrator access
-- Keys are stored as SHA-256 hashes (never plaintext)

CREATE TABLE IF NOT EXISTS apps (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    callback_url TEXT,
    webhook_secret TEXT,
    created_at INTEGER NOT NULL,
    owner_address TEXT NOT NULL,
    is_active INTEGER DEFAULT 1
);

CREATE TABLE IF NOT EXISTS api_keys (
    id TEXT PRIMARY KEY NOT NULL,
    app_id TEXT NOT NULL REFERENCES apps(id),
    key_hash BLOB NOT NULL,
    label TEXT DEFAULT 'default',
    created_at INTEGER NOT NULL,
    last_used_at INTEGER,
    is_active INTEGER DEFAULT 1
);

-- Index for looking up keys by hash
CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);
-- Index for listing keys by app
CREATE INDEX IF NOT EXISTS idx_api_keys_app_id ON api_keys(app_id);
