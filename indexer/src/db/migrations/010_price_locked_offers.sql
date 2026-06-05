-- Price-locked offers: track market price vs fixed price
ALTER TABLE offers ADD COLUMN price_type TEXT DEFAULT 'fixed';
ALTER TABLE offers ADD COLUMN price_offset REAL DEFAULT 0.0;
ALTER TABLE offers ADD COLUMN min_price REAL;
ALTER TABLE offers ADD COLUMN max_price REAL;
ALTER TABLE offers ADD COLUMN current_price REAL;
ALTER TABLE offers ADD COLUMN price_currency TEXT DEFAULT 'USD';
ALTER TABLE offers ADD COLUMN price_updated_at INTEGER;
