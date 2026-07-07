CREATE TABLE IF NOT EXISTS token_registry (
    id TEXT PRIMARY KEY,
    ticker TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    total_supply INTEGER NOT NULL,
    decimals INTEGER NOT NULL DEFAULT 8,
    mint_mode TEXT NOT NULL DEFAULT 'fixed',
    owner_address TEXT,
    covenant_address TEXT,
    template_hash BLOB,
    metadata_json TEXT,
    deploy_tx_id TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at INTEGER NOT NULL,
    deployed_at INTEGER
);
CREATE INDEX IF NOT EXISTS idx_token_registry_ticker ON token_registry(ticker);
CREATE INDEX IF NOT EXISTS idx_token_registry_owner ON token_registry(owner_address);
