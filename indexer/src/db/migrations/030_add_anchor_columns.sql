ALTER TABLE escrow_messages ADD COLUMN anchor_tx_id TEXT;
ALTER TABLE escrow_messages ADD COLUMN anchor_daa_score INTEGER;
ALTER TABLE escrow_messages ADD COLUMN anchor_batch_hash TEXT;
