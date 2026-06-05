-- Add price_type column to escrows table (for market vs fixed price tracking)
ALTER TABLE escrows ADD COLUMN price_type TEXT;
