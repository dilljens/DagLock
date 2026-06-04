-- Vaults: time-locked KAS storage with various unlock conditions
CREATE TABLE IF NOT EXISTS vaults (
    id TEXT PRIMARY KEY,
    owner_address TEXT NOT NULL,
    beneficiary_address TEXT,
    vault_type TEXT NOT NULL DEFAULT 'time',
    status TEXT NOT NULL DEFAULT 'locked',
    amount_sompi INTEGER NOT NULL,
    timeout INTEGER NOT NULL,
    lock_tx_id TEXT,
    lock_tx_output_index INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    unlocked_at INTEGER,
    expires_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_vaults_owner ON vaults(owner_address);
CREATE INDEX IF NOT EXISTS idx_vaults_status ON vaults(status);
