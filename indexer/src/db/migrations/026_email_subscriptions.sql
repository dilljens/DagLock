CREATE TABLE IF NOT EXISTS email_subscriptions (
    address TEXT NOT NULL PRIMARY KEY,
    email TEXT NOT NULL,
    email_verified INTEGER NOT NULL DEFAULT 0,
    verification_code TEXT,
    notify_created INTEGER NOT NULL DEFAULT 1,
    notify_settled INTEGER NOT NULL DEFAULT 1,
    notify_disputed INTEGER NOT NULL DEFAULT 1,
    notify_refunded INTEGER NOT NULL DEFAULT 1,
    notify_expired INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    verified_at INTEGER,
    updated_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_email_subs_address ON email_subscriptions(address);
