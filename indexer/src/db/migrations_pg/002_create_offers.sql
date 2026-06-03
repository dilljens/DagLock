-- DagLock PostgreSQL schema: offers table

CREATE TABLE IF NOT EXISTS offers (
    id TEXT PRIMARY KEY,
    creator_address TEXT NOT NULL,
    side TEXT NOT NULL,
    base_asset TEXT NOT NULL,
    quote_asset TEXT NOT NULL,
    amount_sompi BIGINT NOT NULL,
    counterparty_address TEXT,
    status TEXT NOT NULL DEFAULT 'proposed',
    expires_at BIGINT,
    created_at BIGINT NOT NULL
);

CREATE INDEX idx_offers_status ON offers(status);
CREATE INDEX idx_offers_creator ON offers(creator_address);
CREATE INDEX idx_offers_asset ON offers(base_asset, quote_asset);
