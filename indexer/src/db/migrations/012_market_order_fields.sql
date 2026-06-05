-- Market order fields for price-locked escrows
ALTER TABLE escrows ADD COLUMN price_lock_time INTEGER;
ALTER TABLE escrows ADD COLUMN price_at_settlement REAL;
ALTER TABLE escrows ADD COLUMN price_source TEXT;
