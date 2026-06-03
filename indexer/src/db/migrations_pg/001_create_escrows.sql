-- DagLock PostgreSQL schema: escrows table

CREATE TABLE IF NOT EXISTS escrows (
    id TEXT PRIMARY KEY,
    lock_tx_id TEXT NOT NULL,
    lock_tx_output_index INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending_confirmation',
    asset_type TEXT NOT NULL DEFAULT 'KAS',
    buyer_address TEXT NOT NULL,
    seller_address TEXT,
    amount_sompi BIGINT NOT NULL,
    fee_sompi BIGINT NOT NULL,
    template_hash BYTEA,
    expiration_daa_score BIGINT,
    disputed_at BIGINT,
    dispute_reason TEXT,
    cancelled_at BIGINT,
    expired_at BIGINT,
    created_at BIGINT NOT NULL,
    settled_at BIGINT,
    refunded_at BIGINT
);

CREATE INDEX idx_escrows_status ON escrows(status);
CREATE INDEX idx_escrows_buyer ON escrows(buyer_address);
CREATE INDEX idx_escrows_seller ON escrows(seller_address);
CREATE INDEX idx_escrows_created ON escrows(created_at);
