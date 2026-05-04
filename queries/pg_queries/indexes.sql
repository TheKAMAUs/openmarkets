-- migrations/20250405_add_market_id_indexes.sql
USE polyMarket;

-- ============================================
-- Only MARKET_ID indexes (most important for your queries)
-- ============================================

-- Market price data - market_id index
ALTER TABLE market_price_data ADD INDEX idx_market_id market_id TYPE bloom_filter GRANULARITY 3;
ALTER TABLE market_price_data MATERIALIZE INDEX idx_market_id;

-- Order book - market_id is already in ORDER BY, but add secondary index
ALTER TABLE market_order_book ADD INDEX idx_market_id market_id TYPE bloom_filter GRANULARITY 3;
ALTER TABLE market_order_book MATERIALIZE INDEX idx_market_id;

-- Order book analytical - market_id is already in ORDER BY
ALTER TABLE market_order_book_analytical ADD INDEX idx_market_id market_id TYPE bloom_filter GRANULARITY 3;
ALTER TABLE market_order_book_analytical MATERIALIZE INDEX idx_market_id;

-- Volume data - market_id is already in ORDER BY (market_id, ts)
-- No additional index needed since ORDER BY handles it
-- But add for extra performance
ALTER TABLE market_volume_data ADD INDEX idx_market_id market_id TYPE bloom_filter GRANULARITY 3;
ALTER TABLE market_volume_data MATERIALIZE INDEX idx_market_id;