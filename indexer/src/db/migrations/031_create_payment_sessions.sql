-- Payment sessions for merchant checkout flow (Escrow-as-a-Service)
-- Each session represents a pending payment created via the merchant API.

CREATE TABLE IF NOT EXISTS payment_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    app_id TEXT NOT NULL REFERENCES apps(id),
    escrow_id TEXT REFERENCES escrows(id),
    amount_sompi INTEGER NOT NULL,
    asset_type TEXT NOT NULL DEFAULT 'KAS',
    seller_address TEXT NOT NULL,
    memo TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    buyer_address TEXT,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    webhook_url TEXT,
    redirect_url TEXT
);

CREATE INDEX IF NOT EXISTS idx_payment_sessions_app ON payment_sessions(app_id);
CREATE INDEX IF NOT EXISTS idx_payment_sessions_status ON payment_sessions(status);
CREATE INDEX IF NOT EXISTS idx_payment_sessions_escrow ON payment_sessions(escrow_id);
