CREATE TABLE IF NOT EXISTS offer_counteroffers (
    id TEXT PRIMARY KEY,
    offer_id TEXT NOT NULL REFERENCES offers(id),
    proposer_address TEXT NOT NULL,
    amount_sompi INTEGER,
    price_offset REAL,
    timeout INTEGER,
    dispute_mode TEXT,
    message TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_counteroffers_offer ON offer_counteroffers(offer_id);
CREATE INDEX IF NOT EXISTS idx_counteroffers_proposer ON offer_counteroffers(proposer_address);
