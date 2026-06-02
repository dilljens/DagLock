CREATE TABLE IF NOT EXISTS offers (
    id TEXT PRIMARY KEY NOT NULL,
    creator_address TEXT NOT NULL,
    side TEXT NOT NULL,
    base_asset TEXT NOT NULL,
    quote_asset TEXT NOT NULL,
    amount_sompi INTEGER NOT NULL,
    counterparty_address TEXT,
    status TEXT NOT NULL DEFAULT 'proposed',
    expires_at INTEGER,
    created_at INTEGER NOT NULL
);
