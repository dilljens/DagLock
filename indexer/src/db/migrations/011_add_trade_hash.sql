-- Add trade_hash to escrows table for atomic swaps
ALTER TABLE escrows ADD COLUMN trade_hash TEXT;
