-- DagLock PostgreSQL schema: additional indexes

CREATE INDEX idx_escrows_lock_tx ON escrows(lock_tx_id);
CREATE INDEX idx_escrows_amount ON escrows(amount_sompi);
