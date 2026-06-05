
## Price-Locked Offers (v0.2.0)

Market-price offers that auto-update via CoinGecko.

### How it works
- Offers can be `fixed` (creator sets exact price) or `market` (price from CoinGecko)
- Market-priced offers store a `price_offset` (optional ±%), `min_price`, and `max_price`
- The offline reconciliation loop fetches KAS/USD from CoinGecko every 15 minutes
- All market-priced offers are updated in the database
- At ~2,880 calls/month, this easily fits within CoinGecko's free tier (10,000/month)

### Backend
- `migrations/010_price_locked_offers.sql` — adds price columns to offers table
- `listener.rs` — `update_market_prices()` fetches price and updates DB
- `offers.rs` — creation handler fetches initial market price

### Frontend
- CreateOfferForm — price type selector, offset, and bounds
- OfferCard — shows current market price for market-priced offers
