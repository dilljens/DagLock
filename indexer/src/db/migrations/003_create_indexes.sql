CREATE INDEX IF NOT EXISTS idx_escrows_buyer ON escrows(buyer_address);
CREATE INDEX IF NOT EXISTS idx_escrows_seller ON escrows(seller_address);
CREATE INDEX IF NOT EXISTS idx_escrows_status ON escrows(status);
CREATE INDEX IF NOT EXISTS idx_offers_creator ON offers(creator_address);
CREATE INDEX IF NOT EXISTS idx_offers_status ON offers(status);
