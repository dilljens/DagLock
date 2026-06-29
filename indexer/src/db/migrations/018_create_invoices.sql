CREATE TABLE IF NOT EXISTS invoices (
    id TEXT PRIMARY KEY,
    freelancer_address TEXT NOT NULL,
    client_address TEXT,
    escrow_id TEXT REFERENCES escrows(id),
    description TEXT NOT NULL,
    amount_sompi INTEGER NOT NULL,
    due_date INTEGER,
    status TEXT NOT NULL DEFAULT 'draft',
    created_at INTEGER NOT NULL,
    paid_at INTEGER,
    settled_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_invoices_freelancer ON invoices(freelancer_address);
CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);
