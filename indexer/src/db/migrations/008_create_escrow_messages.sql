-- Migration 008: Escrow-threaded messaging with AES-256-GCM encrypted content

CREATE TABLE IF NOT EXISTS escrow_messages (
    id TEXT PRIMARY KEY,
    escrow_id TEXT NOT NULL REFERENCES escrows(id),
    sender_address TEXT NOT NULL,
    content_enc TEXT NOT NULL,       -- AES-256-GCM ciphertext, hex-encoded
    nonce TEXT NOT NULL,              -- AES-256-GCM nonce, hex-encoded
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_escrow_messages_escrow_id ON escrow_messages(escrow_id);
CREATE INDEX IF NOT EXISTS idx_escrow_messages_created ON escrow_messages(created_at);
