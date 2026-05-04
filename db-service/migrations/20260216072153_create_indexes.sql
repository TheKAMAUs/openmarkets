
CREATE INDEX IF NOT EXISTS idx_orders_market_id ON polymarket.orders (market_id);
CREATE INDEX IF NOT EXISTS idx_orders_user_id ON polymarket.orders (user_id);

CREATE INDEX IF NOT EXISTS idx_orders_status ON polymarket.orders(status);

-- 4. OPEN ORDERS QUERIES (matching engine)
CREATE INDEX  IF NOT EXISTS idx_orders_open_lookup
ON polymarket.orders (status, market_id, created_at DESC)
INCLUDE (price, quantity, filled_quantity, side, order_type, created_at, updated_at, user_id)
WHERE status IN ('open', 'unspecified'::polymarket.order_status);

-- covers both the status and non-status path reasonably
CREATE INDEX  IF NOT EXISTS idx_orders_user_market_status_created_desc
  ON polymarket.orders (user_id, market_id, status, created_at DESC)
  INCLUDE (id, price, quantity, filled_quantity, side, order_type, updated_at);

CREATE INDEX  IF NOT EXISTS idx_markets_status_created_desc
    ON polymarket.markets (status, created_at DESC)
    INCLUDE (name, description, logo, final_outcome, liquidity_b, market_expiry, updated_at);

-- 4. EXPIRY-BASED QUERIES (for cron jobs)
CREATE INDEX  IF NOT EXISTS idx_markets_expiry_status 
ON polymarket.markets (market_expiry, status) 
WHERE status = 'open';

CREATE UNIQUE INDEX  IF NOT EXISTS uq_users_google_id
  ON polymarket.users (google_id);

-- 3. Orders table: used heavily in get_user_metadata (counts + avg filled ratio)
CREATE INDEX  IF NOT EXISTS idx_orders_user_covering
  ON polymarket.orders (user_id)
  INCLUDE (status, quantity, filled_quantity, created_at, market_id);

-- 2. Covering index for metadata query (CRITICAL!)
CREATE INDEX  IF NOT EXISTS idx_transactions_metadata_covering 
ON polymarket.user_transactions (user_id, transaction_type, created_at DESC) 
INCLUDE (amount, transaction_status, confirmed_at);

CREATE UNIQUE INDEX  IF NOT EXISTS idx_user_holdings_unique 
ON polymarket.user_holdings (user_id, market_id, outcome);

CREATE INDEX  IF NOT EXISTS idx_user_holdings_user_created 
ON polymarket.user_holdings (user_id, created_at DESC) 
INCLUDE (market_id, outcome, shares);

CREATE INDEX  IF NOT EXISTS idx_user_holdings_top_holders 
ON polymarket.user_holdings (market_id, shares DESC) 
INCLUDE (user_id, outcome);

CREATE INDEX  IF NOT EXISTS idx_user_trades_user_timestamp 
ON polymarket.user_trades (user_id, timestamp DESC) 
INCLUDE (market_id, trade_type, outcome, price, quantity);

  -- 5. MARKET + USER COMBOS (for analytics)
CREATE INDEX  IF NOT EXISTS idx_user_trades_market_user 
ON polymarket.user_trades (market_id, user_id, timestamp DESC) 
INCLUDE (trade_type, outcome, price, quantity);
-- Used by: Checking if user traded in specific market












CREATE INDEX  IF NOT EXISTS idx_orders_user_status_created_desc
  ON polymarket.orders (user_id, status, created_at DESC)
  INCLUDE (id, market_id, price, quantity, filled_quantity, side, order_type, updated_at);

  CREATE INDEX  IF NOT EXISTS idx_orders_locked_funds 
ON polymarket.orders (user_id, side, status) 
  WHERE side = 'buy'::polymarket.order_side AND status = 'open'::polymarket.order_status;


CREATE INDEX  IF NOT EXISTS idx_orders_locked_stokes 
ON polymarket.orders (user_id, outcome, side, status) 
 WHERE side = 'sell'::polymarket.order_side AND status = 'open'::polymarket.order_status;

CREATE INDEX  IF NOT EXISTS idx_markets_created_desc 
ON polymarket.markets (created_at DESC);

-- 4. User holdings: used to aggregate shares by (user_id, market_id, outcome)
CREATE INDEX  IF NOT EXISTS idx_user_holdings_user_market_outcome
  ON polymarket.user_holdings (user_id, market_id, outcome)
  INCLUDE (shares);

CREATE INDEX  IF NOT EXISTS idx_user_trades_user_created_desc
  ON polymarket.user_trades (user_id, created_at DESC)
  INCLUDE (quantity, price, market_id);

-- 6. Additional index to speed COUNT(DISTINCT market_id) for trades
CREATE INDEX  IF NOT EXISTS idx_user_trades_user_market
  ON polymarket.user_trades ( market_id)


-- 7. User transactions: used to SUM(amount) FILTER(transaction_type) and MAX(created_at) FILTER(transaction_type)
--    A composite index plus partial indexes for the two common types helps a lot.
CREATE INDEX  IF NOT EXISTS idx_user_transactions_user_created_desc
  ON polymarket.user_transactions (user_id, created_at DESC)
INCLUDE (transaction_type);

-- Partial covering indexes for deposit and withdrawal aggregates
CREATE INDEX  IF NOT EXISTS idx_user_txns_deposit_covering
  ON polymarket.user_transactions (user_id)
  INCLUDE (amount, created_at)
  WHERE transaction_type = 'deposit'::polymarket.user_transaction_type;

CREATE INDEX  IF NOT EXISTS idx_user_txns_withdrawal_covering
  ON polymarket.user_transactions (user_id)
  INCLUDE (amount, created_at)
  WHERE transaction_type = 'withdrawal'::polymarket.user_transaction_type;


CREATE INDEX  IF NOT EXISTS idx_user_trades_buy_order_id
  ON polymarket.user_trades (buy_order_id);

CREATE INDEX  IF NOT EXISTS idx_user_trades_sell_order_id
  ON polymarket.user_trades (sell_order_id);

  CREATE INDEX  IF NOT EXISTS idx_user_trades_market_id
  ON polymarket.user_trades (market_id)
  INCLUDE (timestamp, user_id);

-- 6. TIMESTAMP RANGE QUERIES (for reports)
CREATE INDEX  IF NOT EXISTS idx_user_trades_timestamp 
ON polymarket.user_trades (timestamp DESC) 
INCLUDE (market_id, user_id, price, quantity);


CREATE INDEX  IF NOT EXISTS idx_transactions_user_history 
ON polymarket.user_transactions (user_id, created_at DESC);