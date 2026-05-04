-- Add migration script here

-- enums
CREATE TYPE polymarket.market_status AS ENUM ('open', 'closed', 'settled');
CREATE TYPE polymarket.outcome AS ENUM ('yes', 'no', 'unspecified');
CREATE TYPE polymarket.order_side AS ENUM ('buy', 'sell');
CREATE TYPE polymarket.order_status AS ENUM ('open', 'filled', 'cancelled', 'unspecified', 'expired', 'pending_cancel', 'partial_fill', 'pending_update');
CREATE TYPE polymarket.user_transaction_type AS ENUM ('deposit', 'withdrawal', 'trade');
CREATE TYPE polymarket.user_transaction_status AS ENUM ('pending', 'complete', 'failed');
CREATE TYPE polymarket.order_type AS ENUM ('limit','market', 'stop_loss', 'take_profit');

-- Verification enums
CREATE TYPE polymarket.verification_status AS ENUM (
    'unverified',      -- User hasn't applied yet
    'pending',         -- Applied, waiting for admin review
    'approved',        -- Admin approved
    'rejected',        -- Admin rejected
    'expired',         -- Verification documents expired
    'suspended'        -- Verification temporarily suspended
);

CREATE TYPE polymarket.verification_document_type AS ENUM (
    'passport',
    'drivers_license',
    'national_id',
    'selfie'
    'residence_permit',
    'proof_of_address'
);

CREATE TYPE polymarket.verification_document_status AS ENUM (
    'pending',
    'approved',
    'rejected',
    'expired'
);

CREATE TYPE polymarket.verification_step AS ENUM (
    'identity_basic',
    'document_upload',
    'liveness_check',
    'address_verification',
    'risk_assessment',
    'completed'
);

CREATE TYPE polymarket.admin_verification_action AS ENUM (
    'approved',
    'rejected',
    'requested_revision',
    'suspended',
    'expired',
    'notes_added'
);

-- Create enum for suggestion status
CREATE TYPE polymarket.suggestion_status AS ENUM (
    'pending',
    'approved',
    'rejected',
    'in_review',
    'implemented'
);



-- users table (enhanced with verification + referral system)
CREATE TABLE IF NOT EXISTS polymarket.users (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,

    -- oAuth2 fields
    "google_id" varchar(255) UNIQUE,
    "firebase_id" varchar(255) UNIQUE,
    "email" varchar(255) UNIQUE NOT NULL,
    "name" varchar(255) NOT NULL,
    "avatar" varchar(255),
    "last_login" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "verified" boolean NOT NULL DEFAULT false,

    -- verification fields
    "verification_status" polymarket.verification_status NOT NULL DEFAULT 'unverified',
    "verification_step" polymarket.verification_step NOT NULL DEFAULT 'identity_basic',
    "verification_applied_at" timestamp,
    "verification_reviewed_at" timestamp,
    "verified_at" timestamp,
    "verification_expires_at" timestamp,
    "verification_notes" text,

    -- referral system (NEW)
    "referral_code" varchar(20) UNIQUE NOT NULL,
    "referred_by" uuid NULL REFERENCES polymarket.users(id),

    -- wallet fields
    "public_key" varchar(255) NOT NULL UNIQUE,
    "private_key" TEXT NOT NULL UNIQUE,
    "balance" decimal(20,8) NOT NULL DEFAULT 0,

    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);


-- markets
CREATE TABLE IF NOT EXISTS polymarket.markets (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "name" varchar(255) NOT NULL,
    "description" text NOT NULL,
    "logo" text[] NOT NULL DEFAULT '{}',
    "status" "polymarket"."market_status" NOT NULL DEFAULT 'open',
    "liquidity_b" decimal NOT NULL DEFAULT 0,
    "final_outcome" "polymarket"."outcome" NOT NULL DEFAULT 'unspecified',
    "market_expiry" timestamp NOT NULL,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- New fields
    "parent_id" uuid NULL REFERENCES "polymarket"."markets"("id") ON DELETE CASCADE, 
    "is_event" boolean NOT NULL DEFAULT false,
    "child_market_ids" uuid[] NULL,

    -- Additional fields
    "category" varchar(255) NULL,          -- e.g., Economy, Sports, Politics
    "resolution_criteria" text NULL,       -- e.g., How the market will be settled
    "slug" varchar(255) NULL,              -- Still allows NULL, but NOT unique anymore

    -- LMSR State fields (outstanding shares)
    "q_yes" decimal NOT NULL DEFAULT 0,    -- Net Yes shares sold to traders
    "q_no" decimal NOT NULL DEFAULT 0      -- Net No shares sold to traders
);

-- orders
CREATE TABLE IF NOT EXISTS polymarket.orders (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES polymarket.users("id"),
    "market_id" uuid NOT NULL REFERENCES polymarket.markets("id"),
    "side" polymarket.order_side NOT NULL,
    "outcome" polymarket.outcome NOT NULL DEFAULT 'unspecified',
    "price" decimal NOT NULL,
    "quantity" decimal NOT NULL CHECK ("quantity" >= 0),
    "filled_quantity" decimal NOT NULL DEFAULT 0,
    "status" polymarket.order_status NOT NULL DEFAULT 'unspecified',
    "order_type" polymarket.order_type NOT NULL,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE IF NOT EXISTS polymarket.market_state_history (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,

    "market_id" uuid NOT NULL REFERENCES polymarket.markets("id"),

    "q_yes" decimal NOT NULL,
    "q_no" decimal NOT NULL,
    "liquidity_b" decimal NOT NULL CHECK ("liquidity_b" >= 0),

    "price_yes" decimal NOT NULL CHECK ("price_yes" >= 0),
    "price_no" decimal NOT NULL CHECK ("price_no" >= 0),

    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE IF NOT EXISTS polymarket.lp_positions (
    "lp_position_id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES polymarket.users("id"),
    "market_id" uuid NOT NULL REFERENCES polymarket.markets("id"),
    "amount_deposited" decimal NOT NULL DEFAULT 0 CHECK ("amount_deposited" >= 0),
    "shares_of_pool" decimal NOT NULL DEFAULT 0 CHECK ("shares_of_pool" >= 0),
    "total_fees_earned" decimal NOT NULL DEFAULT 0 CHECK ("total_fees_earned" >= 0),
    "withdrawn_amount" decimal NOT NULL DEFAULT 0 CHECK ("withdrawn_amount" >= 0),
    "is_active" boolean NOT NULL DEFAULT TRUE,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);


-- user_trades
CREATE TABLE IF NOT EXISTS polymarket.user_trades (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES polymarket.users("id"),
    "buy_order_id" uuid NOT NULL REFERENCES polymarket.orders("id"),
    "sell_order_id" uuid NOT NULL REFERENCES polymarket.orders("id"),
    "trade_type" polymarket.order_side NOT NULL, -- we are storing this to prevent joins for optimizing query performance (for order.type == buy then trade_type == sell and vice versa)
    "market_id" uuid NOT NULL REFERENCES polymarket.markets("id"),
    "outcome" polymarket.outcome NOT NULL,
    "price" decimal NOT NULL,
    "quantity" decimal NOT NULL CHECK ("quantity" > 0),
    "timestamp" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- user_holdings
CREATE TABLE IF NOT EXISTS polymarket.user_holdings (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES polymarket.users("id"),
    "market_id" uuid NOT NULL REFERENCES polymarket.markets("id"),    
    "shares" decimal NOT NULL DEFAULT 0,
    "outcome" polymarket.outcome NOT NULL DEFAULT 'unspecified',
    "settled" boolean NOT NULL DEFAULT false,  -- New field
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (user_id, market_id, outcome)
);

-- user_transactions
CREATE TABLE IF NOT EXISTS polymarket.user_transactions (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES polymarket.users("id"),
    "amount" decimal NOT NULL CHECK ("amount" > 0),
    "transaction_type" polymarket.user_transaction_type NOT NULL,
    "transaction_status" polymarket.user_transaction_status NOT NULL,
    "tx_hash" varchar(255) NOT NULL,
    "confirmed_at" timestamp,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Simple discussions table
CREATE TABLE IF NOT EXISTS polymarket.discussions (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "market_id" uuid NOT NULL REFERENCES polymarket.markets("id") ON DELETE CASCADE,
    "user_id" uuid NOT NULL REFERENCES polymarket.users("id") ON DELETE CASCADE,
    "parent_id" uuid REFERENCES polymarket.discussions("id") ON DELETE CASCADE, -- for replies
    
    "content" text NOT NULL,
    "upvotes" integer NOT NULL DEFAULT 0,
    
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add this simple votes table first
CREATE TABLE IF NOT EXISTS polymarket.discussion_votes (
    "discussion_id" uuid NOT NULL REFERENCES polymarket.discussions("id") ON DELETE CASCADE,
    "user_id" uuid NOT NULL REFERENCES polymarket.users("id") ON DELETE CASCADE,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("discussion_id", "user_id")
);



-- Simple suggestions table with admin notes
CREATE TABLE IF NOT EXISTS polymarket.market_suggestions (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES polymarket.users("id") ON DELETE CASCADE,
    
    "title" varchar(255) NOT NULL,
    "description" text NOT NULL,
    "category" varchar(100),
    
    "upvotes" integer NOT NULL DEFAULT 0,
    
    "status" varchar(50) NOT NULL DEFAULT 'pending', -- pending, approved, rejected
    "admin_notes" text,  -- ✅ Added for admin feedback
    
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP  -- ✅ Added for tracking updates
);

-- Add index for better performance
CREATE INDEX idx_suggestions_status ON polymarket.market_suggestions("status");

-- Simple votes table
CREATE TABLE IF NOT EXISTS polymarket.suggestion_votes (
    "suggestion_id" uuid NOT NULL REFERENCES polymarket.market_suggestions("id") ON DELETE CASCADE,
    "user_id" uuid NOT NULL REFERENCES polymarket.users("id") ON DELETE CASCADE,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("suggestion_id", "user_id")
);

-- verification documents table
CREATE TABLE IF NOT EXISTS polymarket.user_verification_documents (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES polymarket.users("id") ON DELETE CASCADE,
    "document_type" polymarket.verification_document_type NOT NULL,
    "document_url" text NOT NULL,
    "document_hash" text,
    "status" polymarket.verification_document_status NOT NULL DEFAULT 'pending',
    "rejection_reason" text,
    "uploaded_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "reviewed_at" timestamp,
    "reviewed_by" uuid REFERENCES polymarket.users("id"),
    "expires_at" timestamp,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- verification audit log
CREATE TABLE IF NOT EXISTS polymarket.verification_audit_log (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES polymarket.users("id") ON DELETE CASCADE,
    "admin_id" uuid REFERENCES polymarket.users("id"),
    "action" polymarket.admin_verification_action NOT NULL,
    "previous_status" polymarket.verification_status,
    "new_status" polymarket.verification_status,
    "notes" text,
    "document_id" text,  -- Added field for document ID
    "metadata" jsonb,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);



-- deposits (M-Pesa incoming payments)
CREATE TABLE IF NOT EXISTS polymarket.deposits (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES polymarket.users(id) ON DELETE CASCADE,
    
    -- M-Pesa callback fields
    "transaction_type" varchar(50) NOT NULL,           -- e.g., "CustomerPayBillOnline"
    "transaction_id" varchar(50) UNIQUE NOT NULL,      -- M-Pesa transaction ID (e.g., "NFI4SB21X3")
    "transaction_time" timestamp NOT NULL,              -- Time of transaction
    "amount" decimal(20,8) NOT NULL,                    -- Transaction amount
    "business_shortcode" varchar(20) NOT NULL,          -- Paybill/Till number
    "bill_ref_number" varchar(50),                       -- Account reference (usually user ID or phone)
    "invoice_number" varchar(50),                        
    "org_account_balance" decimal(20,8),                 -- Organization account balance after transaction
    "third_party_transaction_id" varchar(100),           -- Third party transaction ID
    
    -- Customer information
    "phone_number" varchar(20) NOT NULL,                 -- Customer phone number (MSISDN)
    "first_name" varchar(100),
    "middle_name" varchar(100),
    "last_name" varchar(100),
    
    -- Additional M-Pesa fields
    "raw_callback_data" jsonb,                           -- Store complete callback payload
    "result_code" integer DEFAULT 0,                      -- 0 = success, others = error codes
    "result_desc" varchar(255),                           -- Result description
    
    -- System fields
    "status" varchar(50) NOT NULL DEFAULT 'pending',      -- pending, completed, failed, reversed
    "processed_at" timestamp,                              -- When deposit was credited to user
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexes for common queries
    CONSTRAINT fk_deposits_user FOREIGN KEY (user_id) REFERENCES polymarket.users(id)
);


-- withdrawals (M-Pesa B2C payments)
CREATE TABLE IF NOT EXISTS polymarket.withdrawals (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES polymarket.users(id) ON DELETE CASCADE,
    
    -- Request fields
    "amount" decimal(20,8) NOT NULL,                    -- Withdrawal amount
    "phone_number" varchar(20) NOT NULL,                 -- Recipient phone number
    "reference" varchar(100) NOT NULL,                   -- Withdrawal reference (unique)
    
    -- M-Pesa B2C callback fields
    "conversation_id" varchar(50),                        -- M-Pesa conversation ID
    "originator_conversation_id" varchar(50),             -- Originator conversation ID
    "transaction_id" varchar(50) UNIQUE,                  -- M-Pesa transaction ID (if successful)
    "transaction_time" timestamp,                          -- Time of transaction
    
    -- B2C specific fields
    "transaction_amount" decimal(20,8),                   -- Amount actually transacted
    "receiver_party_public_name" varchar(255),            -- Receiver name from M-Pesa
    "transaction_completed_time" timestamp,                -- When transaction was completed
    "b2c_charges" decimal(20,8),                          -- B2C charges incurred
    "b2c_utility_account_balance" decimal(20,8),          -- Utility account balance
    "b2c_working_account_balance" decimal(20,8),          -- Working account balance
    
    -- Result fields
    "result_code" integer,                                 -- 0 = success, others = error
    "result_desc" varchar(255),                            -- Result description
    
    -- Additional data
    "raw_callback_data" jsonb,                            -- Store complete callback payload
    "error_message" text,                                  -- Error message if failed
    
    -- System fields
    "status" varchar(50) NOT NULL DEFAULT 'pending',       -- pending, processing, completed, failed, reversed
    "requested_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "processed_at" timestamp,                               -- When withdrawal was processed
    "completed_at" timestamp,                               -- When withdrawal was completed/failed
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_withdrawals_user FOREIGN KEY (user_id) REFERENCES polymarket.users(id),
    CONSTRAINT unique_withdrawal_reference UNIQUE (reference)
);