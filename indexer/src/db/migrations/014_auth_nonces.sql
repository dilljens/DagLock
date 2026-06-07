-- Auth nonce store for replay protection
--
-- Stores used nonces with a timestamp so they can be cleaned up
-- after the 5-minute expiry window. Messages must include:
--   {action}:{escrow_id}:{timestamp}:{nonce}
-- where nonce = BLAKE2b-160(timestamp + random_bytes) truncated to 20 bytes

CREATE TABLE IF NOT EXISTS auth_nonces (
    nonce BLOB PRIMARY KEY,         -- 20-byte nonce (BLAKE2b-160)
    action TEXT NOT NULL,            -- settle, refund, dispute, cancel
    escrow_id TEXT NOT NULL,         -- escrow identifier
    address TEXT NOT NULL,           -- signer's Kaspa address
    created_at INTEGER NOT NULL      -- unix timestamp when stored
);

-- Index for garbage collection (cleanup expired nonces)
CREATE INDEX IF NOT EXISTS idx_auth_nonces_created_at
    ON auth_nonces(created_at);
