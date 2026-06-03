-- Migration 006: Vouching / Web of Trust
-- Users can vouch for each other after completed trades.

CREATE TABLE IF NOT EXISTS vouches (
    id TEXT PRIMARY KEY,
    voucher_address TEXT NOT NULL,
    subject_address TEXT NOT NULL,
    escrow_id TEXT,
    note TEXT,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_vouches_subject ON vouches(subject_address);
CREATE INDEX IF NOT EXISTS idx_vouches_voucher ON vouches(voucher_address);
CREATE INDEX IF NOT EXISTS idx_vouches_expires ON vouches(expires_at);
