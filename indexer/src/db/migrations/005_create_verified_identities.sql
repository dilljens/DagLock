-- Migration 005: Verified social identities linked to Kaspa addresses
-- Users sign a message proving they control a Telegram handle (or other social)

CREATE TABLE IF NOT EXISTS verified_identities (
    address TEXT NOT NULL,
    platform TEXT NOT NULL,       -- 'telegram', 'twitter', 'discord', etc.
    handle TEXT NOT NULL,
    signed_message TEXT NOT NULL, -- the raw message that was signed
    signature_hex TEXT NOT NULL,  -- the wallet signature proving ownership
    verified_at INTEGER NOT NULL,
    PRIMARY KEY (address, platform)
);

CREATE INDEX IF NOT EXISTS idx_verified_identities_handle ON verified_identities(platform, handle);
