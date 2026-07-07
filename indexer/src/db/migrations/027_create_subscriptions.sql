CREATE TABLE IF NOT EXISTS subscriptions (
    id TEXT PRIMARY KEY,
    payer_address TEXT NOT NULL,
    recipient_address TEXT NOT NULL,
    total_amount INTEGER NOT NULL,
    installment_amount INTEGER NOT NULL,
    interval_seconds INTEGER NOT NULL,
    start_time INTEGER NOT NULL,
    current_period INTEGER NOT NULL DEFAULT 0,
    max_periods INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at INTEGER NOT NULL,
    cancelled_at INTEGER,
    completed_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_subscriptions_payer ON subscriptions(payer_address);
CREATE INDEX IF NOT EXISTS idx_subscriptions_recipient ON subscriptions(recipient_address);
CREATE INDEX IF NOT EXISTS idx_subscriptions_status ON subscriptions(status);
