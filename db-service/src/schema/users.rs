use base64::{Engine, engine::general_purpose::STANDARD as base64_engine};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_sdk::{signature::Keypair, signer::Signer};
use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;
use rand::{distributions::Alphanumeric, Rng};

use utility_helpers::{log_info, log_error, symmetric::encrypt, types::UnifiedClaims,   types::AuthProvider    };

use crate::schema::enums::{
    OrderSide, 
    VerificationStatus, 
    VerificationStep,
    // Also import these if you need them elsewhere
    // VerificationDocumentType,
    // VerificationDocumentStatus,
    // AdminVerificationAction
};

#[derive(Debug, Serialize, sqlx::FromRow, Default, Deserialize)]
pub struct User {
    pub id: Uuid,

    pub google_id: Option<String>,
    pub firebase_id: Option<String>,
    pub email: String,
    pub name: String,
    pub avatar: Option<String>,
    pub last_login: NaiveDateTime,
    pub verified: bool,

    pub verification_status: VerificationStatus,
    pub verification_step: VerificationStep,
    pub verification_applied_at: Option<NaiveDateTime>,
    pub verification_reviewed_at: Option<NaiveDateTime>,
    pub verified_at: Option<NaiveDateTime>,
    pub verification_expires_at: Option<NaiveDateTime>,
    pub verification_notes: Option<String>,

    pub public_key: String,
    pub private_key: String,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub balance: Decimal,

    // ✅ ADD THESE
    pub referral_code: String,
    pub referred_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Default)]
pub struct UserBalance {
    pub balance: Decimal,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct UserProfileInsights {
    // Base user fields
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub avatar:Option<String>,
    pub public_key: String,
    pub balance: Decimal,
    pub last_login: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub verified: bool,
    pub referral_code: String,
    pub referred_by: Option<Uuid>,
    
    // Verification fields - NOW USING ENUMS (not Strings)
    pub verification_status: VerificationStatus,  // Enum instead of String
    pub verification_step: VerificationStep,      // Enum instead of String
    pub verification_applied_at: Option<NaiveDateTime>,
    pub verification_reviewed_at: Option<NaiveDateTime>,
    pub verified_at: Option<NaiveDateTime>,
    pub verification_expires_at: Option<NaiveDateTime>,
    pub verification_notes: Option<String>,
    
    // Orders
    pub open_orders: Option<i64>,
    pub partial_orders: Option<i64>,
    pub total_orders: Option<i64>,
    pub avg_fill_ratio: Option<Decimal>,

    // Trades
    pub total_trades: Option<i64>,
    pub total_volume: Option<Decimal>,
    pub avg_trade_price: Option<Decimal>,
    pub max_trade_qty: Option<Decimal>,
    pub first_trade_at: Option<NaiveDateTime>,
    pub last_trade_at: Option<NaiveDateTime>,
    pub markets_traded: Option<i64>,

    // Transactions
    pub total_deposit: Option<Decimal>,
    pub total_withdraw: Option<Decimal>,
    pub last_deposit: Option<NaiveDateTime>,
    pub last_withdraw: Option<NaiveDateTime>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]  // ✅ Add Serialize here!
pub struct UserProfitRanking {
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub avatar: Option<String>,
    pub net_profit: Decimal,
    pub winning_trades: i64,
    pub losing_trades: i64,
    pub total_trades: i64,
    pub win_rate: f64,
    pub rank: i64,
}



#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Discussion {
    pub id: Uuid,
    pub market_id: Uuid,
    pub user_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub upvotes: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// For creating a new discussion
#[derive(Debug, Deserialize)]
pub struct CreateDiscussionRequest {
    pub market_id: Uuid,
    pub content: String,
    pub parent_id: Option<Uuid>,  // None for top-level, Some for reply
}

// For API response with user info

#[derive(Debug, Serialize)]
pub struct DiscussionResponse {
    pub id: Uuid,
    pub market_id: Uuid,
    pub user: UserInfo,
    pub content: String,
    pub upvotes: i32,
    pub reply_count: i64,  // Number of replies
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
}


impl User {
pub async fn create_new_user(
    pool: &PgPool,
    claims: &UnifiedClaims,
) -> Result<Self, sqlx::Error> {
   // log_info! uses direct formatting (no field names)

  log_info!(
        "Provider: {:?}, ID: {}, Email: {}",
        claims.provider,
        claims.sub,
        claims.email
    );


    let new_key_pair = Keypair::new();
    let private_key = new_key_pair.to_base58_string();
    let public_key = new_key_pair.pubkey().to_string();

    let encrypted_private_key_bytes = encrypt(private_key.as_bytes())
        .map_err(|e| {
            log_error!(
                error = %e,
                "Failed to encrypt private key"
            );
            sqlx::Error::Decode("Failed to encrypt private key".into())
        })?;

    let encrypted_private_key = base64_engine.encode(encrypted_private_key_bytes);


    // ✅ ADD THIS
let referral_code = Self::generate_referral_code();

    match sqlx::query_as!(
           User,
    r#"
    INSERT INTO "polymarket"."users" (
    google_id,
    firebase_id,
    email,
    name,
    avatar,
    public_key,
    private_key,
    referral_code,   -- ✅ ADD THIS
    verified,
    verification_status,
    verification_step,
    verification_applied_at,
    verification_reviewed_at,
    verified_at,
    verification_expires_at,
    verification_notes
)
VALUES (
    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16

        ) RETURNING 
    id,
    google_id,
    firebase_id,
    email,
    name,
    avatar,
    last_login,
    verified,

    verification_status as "verification_status: VerificationStatus",
    verification_step as "verification_step: VerificationStep",
    verification_applied_at,
    verification_reviewed_at,
    verified_at,
    verification_expires_at,
    verification_notes,

    public_key,
    private_key,

    created_at,
    updated_at,
    balance,

    referral_code,
    referred_by
        "#,
        // 1. google_id
        if matches!(claims.provider, AuthProvider::Google) { 
            Some(&claims.sub)  // ✅ Use Option<&str> for NULLable column
        } else { 
            None 
        },
        
        // 2. firebase_id
        if matches!(claims.provider, AuthProvider::Firebase) { 
            Some(&claims.sub)  // ✅ Use Option<&str> for NULLable column
        } else { 
            None 
        },
        

        claims.email,
        claims.name,
       claims.picture.as_deref(),  // ✅ Can be NULL, no need for unwrap_or_default
  public_key,
encrypted_private_key,
referral_code, // ✅ NEW FIELD
false,
        VerificationStatus::Unverified as _,
        VerificationStep::IdentityBasic as _,
        None as Option<NaiveDateTime>,
        None as Option<NaiveDateTime>,
        None as Option<NaiveDateTime>,
        None as Option<NaiveDateTime>,
        None as Option<String>
    )
    .fetch_one(pool)
    .await {
        Ok(user) => {
         log_info!(
    user_id = %user.id,
    google_id = ?user.google_id,  // ✅ Use ? for Option
    "User created successfully"
);
            Ok(user)
        }
        Err(e) => {
            // Log the detailed error with all relevant context

log_error!(
    error = %e,
    google_id = %claims.sub,
    email = %claims.email,
    name = %claims.name,
    "Failed to insert user into database"
);    
            // Check for specific error types
            if let sqlx::Error::Database(db_err) = &e {
                if let Some(constraint) = db_err.constraint() {
                    log_error!(
                        constraint = %constraint,
                        "Database constraint violation"
                    );
                }
            }
            
            Err(e)
        }
    }
}

pub fn generate_referral_code() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect::<String>()
        .to_uppercase()
}

pub async fn create_or_update_existing_user(
    pool: &PgPool,
    claims: &UnifiedClaims,
) -> Result<(Self, bool), sqlx::Error> {
    // First, fetch existing user with proper type annotations
let existing_user = match claims.provider {
    AuthProvider::Firebase => {
        // First try to find by firebase_id
        let user = sqlx::query_as!(
            User,
            r#"
          SELECT 
                id, google_id, firebase_id, email, name, avatar, last_login, verified,
                verification_status as "verification_status: VerificationStatus",
                verification_step as "verification_step: VerificationStep",
                verification_applied_at,
                verification_reviewed_at,
                verified_at,
                verification_expires_at,
                verification_notes,
                public_key,
                private_key,
                created_at,
                updated_at,
                balance,
                referral_code,
                referred_by
            FROM "polymarket"."users" 
            WHERE firebase_id = $1
            "#,
            &claims.sub
        )
        .fetch_optional(pool)
        .await?;
        
        // If not found by firebase_id, try by email
        if user.is_none() {
            sqlx::query_as!(
                User,
                r#"
             SELECT 
                id, google_id, firebase_id, email, name, avatar, last_login, verified,
                verification_status as "verification_status: VerificationStatus",
                verification_step as "verification_step: VerificationStep",
                verification_applied_at,
                verification_reviewed_at,
                verified_at,
                verification_expires_at,
                verification_notes,
                public_key,
                private_key,
                created_at,
                updated_at,
                balance,
                referral_code,
                referred_by
                FROM "polymarket"."users" 
                WHERE email = $1
                "#,
                &claims.email
            )
            .fetch_optional(pool)
            .await?
        } else {
            user
        }
    },
    AuthProvider::Google => {
        // First try to find by google_id
        let user = sqlx::query_as!(
            User,
            r#"
        SELECT 
                id, google_id, firebase_id, email, name, avatar, last_login, verified,
                verification_status as "verification_status: VerificationStatus",
                verification_step as "verification_step: VerificationStep",
                verification_applied_at,
                verification_reviewed_at,
                verified_at,
                verification_expires_at,
                verification_notes,
                public_key,
                private_key,
                created_at,
                updated_at,
                balance,
                referral_code,
                referred_by
            FROM "polymarket"."users" 
            WHERE google_id = $1
            "#,
            &claims.sub
        )
        .fetch_optional(pool)
        .await?;
        
        // If not found by google_id, try by email
        if user.is_none() {
            sqlx::query_as!(
                User,
                r#"
              SELECT 
                id, google_id, firebase_id, email, name, avatar, last_login, verified,
                verification_status as "verification_status: VerificationStatus",
                verification_step as "verification_step: VerificationStep",
                verification_applied_at,
                verification_reviewed_at,
                verified_at,
                verification_expires_at,
                verification_notes,
                public_key,
                private_key,
                created_at,
                updated_at,
                balance,
                referral_code,
                referred_by
                FROM "polymarket"."users" 
                WHERE email = $1
                "#,
                &claims.email
            )
            .fetch_optional(pool)
            .await?
        } else {
            user
        }
    },
};

    if let Some(user) = existing_user {
        // ✅ FIX: Handle Option<String> for picture
        let avatar = claims.picture.clone().unwrap_or_default();
        
        // Update existing user
        let updated_user = sqlx::query_as!(
            User,
            r#"
            UPDATE "polymarket"."users" SET
                email = $1,
                name = $2,
                avatar = $3,
                last_login = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $4
            RETURNING 
                id, google_id, firebase_id, email, name, avatar, last_login, verified,
                verification_status as "verification_status: VerificationStatus",
                verification_step as "verification_step: VerificationStep",
                verification_applied_at,
                verification_reviewed_at,
                verified_at,
                verification_expires_at,
                verification_notes,
                public_key,
                private_key,
                created_at,
                updated_at,
                balance,
                referral_code,
                referred_by
            "#,
            &claims.email,           // ✅ String
            &claims.name,             // ✅ String (UnifiedClaims.name is String)
            &avatar,                  // ✅ String (converted from Option)
            &user.id                  // ✅ Uuid
        )
        .fetch_one(pool)
        .await?;

        log_info!("User updated {}", updated_user.id);
        Ok((updated_user, false))
    } else {
        let new_user = Self::create_new_user(pool, claims).await?;
        Ok((new_user, true))
    }
}


 pub async fn find_by_referral_code(
        pool: &PgPool,
        code: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, google_id, firebase_id, email, name, avatar, last_login, verified,
                verification_status as "verification_status: VerificationStatus",
                verification_step as "verification_step: VerificationStep",
                verification_applied_at,
                verification_reviewed_at,
                verified_at,
                verification_expires_at,
                verification_notes,
                public_key,
                private_key,
                created_at,
                updated_at,
                balance,
                referral_code,
                referred_by
            FROM polymarket.users
            WHERE referral_code = $1
            "#,
            code
        )
        .fetch_optional(pool)
        .await
    }


    pub async fn set_referred_by(
        pool: &PgPool,
        user_id: Uuid,
        referrer_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE polymarket.users
            SET referred_by = $1
            WHERE id = $2
            "#,
            referrer_id,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }


pub async fn get_user_by_id<'a>(
    executor: impl Executor<'a, Database = Postgres>,
    user_id: Uuid,
) -> Result<Self, sqlx::Error> {
    let user = sqlx::query_as!(
        User,
        r#"
       SELECT 
               id, google_id, firebase_id, email, name, avatar, last_login, verified,
                verification_status as "verification_status: VerificationStatus",
                verification_step as "verification_step: VerificationStep",
                verification_applied_at,
                verification_reviewed_at,
                verified_at,
                verification_expires_at,
                verification_notes,
                public_key,
                private_key,
                created_at,
                updated_at,
                balance,
                referral_code,
                referred_by
        FROM "polymarket"."users" 
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(executor)
    .await?;

    Ok(user)
}


    pub async fn get_user_balance(
        executor: impl Executor<'_, Database = Postgres>,
        user_id: Uuid,
    ) -> Result<Decimal, sqlx::Error> {
        let balance = sqlx::query_as!(
            UserBalance,
            r#"
            SELECT balance FROM polymarket.users WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(executor)
        .await?;

        Ok(balance.balance)
    }

    pub async fn get_two_users_balance<'a>(
        executor: impl Executor<'a, Database = Postgres>,
        user_1_id: Uuid,
        user_2_id: Uuid,
    ) -> Result<(Decimal, Decimal), sqlx::Error> {
        let balances = sqlx::query_as!(
            UserBalance,
            r#"
            SELECT balance from polymarket.users where id in (
                $1, $2
            );
            "#,
            user_1_id,
            user_2_id
        )
        .fetch_all(executor)
        .await?;

        if balances.len() != 2 {
            return Err(sqlx::Error::RowNotFound);
        }

        let user_1_balance = balances[0].balance;
        let user_2_balance = balances[1].balance;
        Ok((user_1_balance, user_2_balance))
    }

    pub async fn update_two_users_balance<'a>(
        executor: impl Executor<'a, Database = Postgres>,
        user_1_id: Uuid,
        user_2_id: Uuid,
        balance_to_update: Decimal,
        user_1_side: OrderSide,
    ) -> Result<(), sqlx::Error> {
        // user_1_side buy then current balance + user_1_new_balance else current balance - user_1_new_balance
        // user_2_side buy then current balance + user_2_new_balance else current balance - user_2_new_balance
        sqlx::query!(
            r#"
            UPDATE polymarket.users
            SET balance = CASE
                WHEN id = $1 THEN balance + ($2::numeric * (CASE WHEN $3 = 'sell'::polymarket.order_side THEN 1 ELSE -1 END))
                WHEN id = $4 THEN balance + ($2::numeric * (CASE WHEN $3 = 'buy'::polymarket.order_side THEN 1 ELSE -1 END))
            END
            WHERE id IN ($1, $4);
            "#,
            user_1_id,
            balance_to_update,
            user_1_side as _,
            user_2_id,
        )
        .execute(executor)
        .await?;

        Ok(())
    }


pub async fn update_user_balance(
    pool: &PgPool,                     // changed from executor to pool
    user_id: Uuid,
    amount: Decimal,
    side: OrderSide,
) -> Result<(), sqlx::Error> {
    match side {
        OrderSide::BUY => {
            sqlx::query!(
                r#"
                UPDATE "polymarket"."users"
                SET balance = balance - $1
                WHERE id = $2
                "#,
                amount,
                user_id
            )
            .execute(pool)             // pass pool directly
            .await?;
        }
        OrderSide::SELL => {
            sqlx::query!(
                r#"
                UPDATE "polymarket"."users"
                SET balance = balance + $1
                WHERE id = $2
                "#,
                amount,
                user_id
            )
            .execute(pool)             // pass pool directly
            .await?;
        }
    }

    Ok(())
}




    pub async fn get_all_user_ids(pool: &PgPool) -> Result<Vec<Uuid>, sqlx::Error> {
        let user_ids = sqlx::query!(
            r#"
            SELECT id FROM "polymarket"."users"
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(user_ids.into_iter().map(|u| u.id).collect())
    }

pub async fn get_or_create_admin(pool: &PgPool) -> Result<Self, sqlx::Error> {
    let admin_email = "arshil@admin.com";
    let admin_name = "Admin";
    let admin_avatar = "https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcT0WHVQ_TkwqOR6gZfW47X9XGJEIZzKiZc8CA&s";
    let admin_google_id = "admin_google_id";
    let admin_balance = Decimal::new(1_000_000, 2); // 10,000.00
let referral_code = Self::generate_referral_code();

let admin_firebase_id = "admin_firebase_id";
  let admin = sqlx::query_as!(
    User,
    r#"
    INSERT INTO "polymarket"."users" (
        google_id,
        firebase_id,
        email,
        name,
        avatar,
        public_key,
        private_key,
        referral_code,
        verified,
        verification_status,
        verification_step,
        verification_applied_at,
        verification_reviewed_at,
        verified_at,
        verification_expires_at,
        verification_notes
    )
    VALUES (
        $1, $2, $3, $4, $5,
        'no_puk',
        'no_prk',
        $6,
        $7,
        $8,
        $9,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP,
        NULL,
        NULL
    )
    ON CONFLICT (google_id) DO UPDATE SET
        email = EXCLUDED.email,
        name = EXCLUDED.name,
        avatar = EXCLUDED.avatar,
        verified = EXCLUDED.verified,
        verification_status = EXCLUDED.verification_status,
        verification_step = EXCLUDED.verification_step,
        updated_at = CURRENT_TIMESTAMP,
        last_login = CURRENT_TIMESTAMP
    RETURNING 
        id,
        google_id,
        firebase_id,
        email,
        name,
        avatar,
        last_login,
        verified,
        verification_status as "verification_status: VerificationStatus",
        verification_step as "verification_step: VerificationStep",
        verification_applied_at,
        verification_reviewed_at,
        verified_at,
        verification_expires_at,
        verification_notes,
        public_key,
        private_key,
        created_at,
        updated_at,
        balance,
        referral_code,
        referred_by
    "#,
    admin_google_id,
    admin_firebase_id,
    admin_email,
    admin_name,
    admin_avatar,
    referral_code,                        // ✅ NEW
    false,                                // verified
    VerificationStatus::Approved as _,     // $8
    VerificationStep::Completed as _       // $9
)
.fetch_one(pool)
.await?;

    Ok(admin)
}

pub async fn get_user_metadata(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<UserProfileInsights, sqlx::Error> {
    let user = sqlx::query_as!(
        UserProfileInsights,
        r#"
            WITH 
            holdings AS (
                SELECT 
                    uh.market_id,
                    uh.outcome,
                    uh.shares
                FROM polymarket.user_holdings uh
                WHERE uh.user_id = $1
            ),

            orders AS (
                SELECT 
                    COUNT(*) FILTER (WHERE status = 'open') AS open_orders,
                    COUNT(*) FILTER (WHERE status = 'partial_fill') AS partial_orders,
                    COUNT(*) AS total_orders,
                    AVG(filled_quantity / NULLIF(quantity, 0)) AS avg_fill_ratio
                FROM polymarket.orders
                WHERE user_id = $1
            ),

            trades AS (
                SELECT 
                    COUNT(*) AS total_trades,
                    SUM(quantity) AS total_volume,
                    AVG(price) AS avg_trade_price,
                    MAX(quantity) AS max_trade_qty,
                    MIN(created_at) AS first_trade_at,
                    MAX(created_at) AS last_trade_at,
                    COUNT(DISTINCT market_id) AS markets_traded
                FROM polymarket.user_trades
                WHERE user_id = $1
            ),

            txns AS (
                SELECT
                    SUM(amount) FILTER (WHERE transaction_type = 'deposit') AS total_deposit,
                    SUM(amount) FILTER (WHERE transaction_type = 'withdrawal') AS total_withdraw,
                    MAX(created_at) FILTER (WHERE transaction_type = 'deposit') AS last_deposit,
                    MAX(created_at) FILTER (WHERE transaction_type = 'withdrawal') AS last_withdraw
                FROM polymarket.user_transactions
                WHERE user_id = $1
            )

         SELECT
    u.id,
    u.name,
    u.email,
    u.avatar,
    u.public_key,
    u.balance,
    u.last_login,
    u.created_at,
    u.verified,

    -- ✅ ADD THESE
    u.referral_code,
    u.referred_by,

                -- Verification fields with type annotations (just like order_status pattern)
                u.verification_status as "verification_status: VerificationStatus",
                u.verification_step as "verification_step: VerificationStep",
                u.verification_applied_at,
                u.verification_reviewed_at,
                u.verified_at,
                u.verification_expires_at,
                u.verification_notes,
                
                 -- Orders
                         o.open_orders,
                        o.partial_orders,
                        o.total_orders,
                        o.avg_fill_ratio,

                -- Trades
                COALESCE(t.total_trades, 0) as "total_trades: i64",
                t.total_volume,
                t.avg_trade_price,
                t.max_trade_qty,
                t.first_trade_at,
                t.last_trade_at,
                COALESCE(t.markets_traded, 0) as "markets_traded: i64",

                -- Txns
                x.total_deposit,
                x.total_withdraw,
                x.last_deposit,
                x.last_withdraw

            FROM polymarket.users u
            LEFT JOIN orders o ON true
            LEFT JOIN trades t ON true
            LEFT JOIN txns x ON true
            WHERE u.id = $1;
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

    pub async fn deposit_funds(
        pool: &PgPool,
        user_id: Uuid,
        amount: Decimal,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE "polymarket"."users"
            SET balance = balance + $1
            WHERE id = $2
            "#,
            amount,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }


pub async fn withdraw_funds(
    pool: &PgPool,
    user_id: Uuid,
    amount: Decimal,
) -> Result<(), sqlx::Error> {
    // Optional: ensure the user has enough balance
    let current_balance: Decimal = sqlx::query_scalar!(
        r#"
        SELECT balance
        FROM "polymarket"."users"
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    if current_balance < amount {
        return Err(sqlx::Error::RowNotFound); // Or define a custom error for insufficient funds
    }

    sqlx::query!(
        r#"
        UPDATE "polymarket"."users"
        SET balance = balance - $1
        WHERE id = $2
        "#,
        amount,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get all users ranked by profit (highest winners first)
pub async fn get_users_ranked_by_profit(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<UserProfitRanking>, sqlx::Error> {
    let rows = sqlx::query_as!(
        UserProfitRanking,
        r#"
        WITH user_stats AS (
            SELECT 
                u.id,
                u.name,
                u.email,
                u.avatar,
                -- Net profit from trades
                (
                    SELECT SUM(
                        CASE 
                            WHEN trade_type = 'sell' THEN quantity
                            ELSE -quantity
                        END
                    )
                    FROM polymarket.user_trades
                    WHERE user_id = u.id
                ) as net_profit,
                -- Winning trades
                (
                    SELECT COUNT(*)
                    FROM polymarket.user_holdings uh
                    JOIN polymarket.markets m ON m.id = uh.market_id
                    WHERE uh.user_id = u.id
                    AND m.status = 'settled'
                    AND uh.outcome = m.final_outcome
                ) as winning_trades,
                -- Losing trades
                (
                    SELECT COUNT(*)
                    FROM polymarket.user_holdings uh
                    JOIN polymarket.markets m ON m.id = uh.market_id
                    WHERE uh.user_id = u.id
                    AND m.status = 'settled'
                    AND uh.outcome != m.final_outcome
                ) as losing_trades
            FROM polymarket.users u
            WHERE EXISTS (
                SELECT 1 FROM polymarket.user_trades WHERE user_id = u.id
            )
        )
        SELECT
            u.id as "user_id!",
            u.name,
            u.email,
            u.avatar,
            u.net_profit as "net_profit!",
            u.winning_trades as "winning_trades!",
            u.losing_trades as "losing_trades!",
            (u.winning_trades + u.losing_trades) as "total_trades!",
            CASE 
                WHEN (u.winning_trades + u.losing_trades) > 0 
                THEN (u.winning_trades::float * 100.0 / (u.winning_trades + u.losing_trades)::float)
                ELSE 0.0 
            END as "win_rate!",
            ROW_NUMBER() OVER (ORDER BY u.net_profit DESC) as "rank!"
        FROM user_stats u
        ORDER BY u.net_profit DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
/// Get top losers
pub async fn get_top_losers(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<UserProfitRanking>, sqlx::Error> {
    let rows = sqlx::query_as!(
        UserProfitRanking,
        r#"
        WITH user_stats AS (
            SELECT 
                u.id,
                u.name,
                u.email,
                u.avatar,
                COALESCE((
                    SELECT SUM(
                        CASE 
                            WHEN trade_type = 'sell' THEN quantity
                            ELSE -quantity
                        END
                    )
                    FROM polymarket.user_trades
                    WHERE user_id = u.id
                ), 0) as net_profit,
                COALESCE((
                    SELECT COUNT(*)
                    FROM polymarket.user_holdings uh
                    JOIN polymarket.markets m ON m.id = uh.market_id
                    WHERE uh.user_id = u.id
                    AND m.status = 'settled'
                    AND uh.outcome = m.final_outcome
                ), 0) as winning_trades,
                COALESCE((
                    SELECT COUNT(*)
                    FROM polymarket.user_holdings uh
                    JOIN polymarket.markets m ON m.id = uh.market_id
                    WHERE uh.user_id = u.id
                    AND m.status = 'settled'
                    AND uh.outcome != m.final_outcome
                ), 0) as losing_trades
            FROM polymarket.users u
            WHERE EXISTS (
                SELECT 1 FROM polymarket.user_trades WHERE user_id = u.id
            )
        )
        SELECT
            u.id as "user_id!",
            u.name,
            u.email,
            u.avatar,
            u.net_profit as "net_profit!",
            u.winning_trades as "winning_trades!",
            u.losing_trades as "losing_trades!",
            (u.winning_trades + u.losing_trades) as "total_trades!",
            CASE 
                WHEN (u.winning_trades + u.losing_trades) > 0 
                THEN (u.winning_trades::float * 100.0 / (u.winning_trades + u.losing_trades)::float)
                ELSE 0.0 
            END as "win_rate!",
            ROW_NUMBER() OVER (ORDER BY u.net_profit ASC) as "rank!"
        FROM user_stats u
        WHERE u.net_profit < 0
        ORDER BY u.net_profit ASC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

   pub async fn create(
        pool: &sqlx::PgPool,
        req: CreateDiscussionRequest,
        user_id: Uuid,
    ) -> Result<Discussion, sqlx::Error> {
        sqlx::query_as!(
            Discussion,
            r#"
            INSERT INTO polymarket.discussions (market_id, user_id, parent_id, content)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            req.market_id,
            user_id,
            req.parent_id,
            req.content
        )
        .fetch_one(pool)
        .await
    }

    

// Get all discussions for a market (including replies)
pub async fn get_all_for_market(
    pool: &sqlx::PgPool,
    market_id: Uuid,
) -> Result<Vec<DiscussionResponse>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"    
        SELECT 
            d.id,
            d.market_id,
            u.id as user_id,
            u.name as user_name,
            u.avatar as user_avatar,
            d.content,
            d.upvotes,
            (SELECT COUNT(*) FROM polymarket.discussions WHERE parent_id = d.id) as "reply_count?: i64",  -- ✅ Add ? to make it Option
            d.created_at
        FROM polymarket.discussions d
        JOIN polymarket.users u ON u.id = d.user_id
        WHERE d.market_id = $1
        ORDER BY 
            CASE WHEN d.parent_id IS NULL THEN 0 ELSE 1 END,
            d.created_at DESC
        "#,
        market_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| DiscussionResponse {
        id: r.id,
        market_id: r.market_id,
        user: UserInfo {
            id: r.user_id,
            name: r.user_name,
            avatar: r.user_avatar,
        },
        content: r.content,
        upvotes: r.upvotes,
        reply_count: r.reply_count.unwrap_or(0),
        created_at: r.created_at,
    }).collect())
}



pub async fn upvote(
    pool: &sqlx::PgPool,
    discussion_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Insert vote - ignore if already voted
    let result = sqlx::query!(
        r#"
        INSERT INTO polymarket.discussion_votes (discussion_id, user_id)
        VALUES ($1, $2)
        ON CONFLICT (discussion_id, user_id) DO NOTHING
        "#,
        discussion_id,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    // If vote was inserted (not a duplicate), increment upvotes
    if result.rows_affected() > 0 {
        sqlx::query!(
            r#"
            UPDATE polymarket.discussions 
            SET upvotes = upvotes + 1
            WHERE id = $1
            "#,
            discussion_id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await
}

pub async fn remove_upvote(
    pool: &sqlx::PgPool,
    discussion_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    let result = sqlx::query!(
        r#"
        DELETE FROM polymarket.discussion_votes
        WHERE discussion_id = $1 AND user_id = $2
        "#,
        discussion_id,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() > 0 {
        sqlx::query!(
            r#"
            UPDATE polymarket.discussions 
            SET upvotes = upvotes - 1
            WHERE id = $1
            "#,
            discussion_id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await
}



}

#[cfg(test)]
mod tests {
    use std::env;

    use utility_helpers::symmetric::decrypt;

    use super::*;

    async fn cleanup_test_user(pool: &PgPool, user_id: Uuid) {
        sqlx::query(r#"DELETE FROM "polymarket"."users" WHERE id = $1"#)
            .bind(user_id)
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_new_user() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        let unique_id = Uuid::new_v4();
        let unique_email = format!("test_{}@gmail.com", unique_id);
        let unique_sub = format!("test_google_id_{}", unique_id);

        let google_claims = GoogleClaims {
            sub: unique_sub,
            email: unique_email,
            exp: 60 * 60 * 24 * 3, // 3 days,
            name: "Test User".to_string(),
            picture: "https://example.com/avatar.png".to_string(),
        };

        let user = User::create_new_user(&pool, &google_claims).await.unwrap();

        let decoded_private_key = base64_engine.decode(&user.private_key).unwrap();
        let decrypted_private_key = decrypt(&decoded_private_key).unwrap();
        let _decrypted_private_key_str = String::from_utf8(decrypted_private_key).unwrap();

        assert!(!user.private_key.is_empty());
        assert!(!user.public_key.is_empty());
        assert_eq!(user.name, "Test User");
        assert_eq!(user.avatar, "https://example.com/avatar.png");
        assert_eq!(user.balance, Decimal::ZERO);
        assert_eq!(user.created_at, user.updated_at);

        // Clean up
        cleanup_test_user(&pool, user.id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_create_or_update_existing_user_new_user() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        let unique_id = Uuid::new_v4();
        let unique_email = format!("test_{}@gmail.com", unique_id);
        let unique_sub = format!("test_google_id_{}", unique_id);

        let google_claims = GoogleClaims {
            sub: unique_sub,
            email: unique_email,
            exp: 60 * 60 * 24 * 3,
            name: "Test User New".to_string(),
            picture: "https://example.com/avatar_new.png".to_string(),
        };

        let (user, is_new) = User::create_or_update_existing_user(&pool, &google_claims)
            .await
            .unwrap();

        // Verify it's a new user
        assert!(is_new);
        assert_eq!(user.name, "Test User New");
        assert_eq!(user.avatar, "https://example.com/avatar_new.png");
        assert_eq!(user.google_id, google_claims.sub);
        assert_eq!(user.email, google_claims.email);

        // Clean up
        cleanup_test_user(&pool, user.id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_create_or_update_existing_user_existing_user() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        let unique_id = Uuid::new_v4();
        let unique_email = format!("test_{}@gmail.com", unique_id);
        let unique_sub = format!("test_google_id_{}", unique_id);

        // First create a user
        let google_claims_initial = GoogleClaims {
            sub: unique_sub.clone(),
            email: unique_email.clone(),
            exp: 60 * 60 * 24 * 3,
            name: "Test User Initial".to_string(),
            picture: "https://example.com/avatar_initial.png".to_string(),
        };

        let (initial_user, is_new_initial) =
            User::create_or_update_existing_user(&pool, &google_claims_initial)
                .await
                .unwrap();
        assert!(is_new_initial);

        // Now update the user
        let google_claims_updated = GoogleClaims {
            sub: unique_sub,
            email: unique_email,
            exp: 60 * 60 * 24 * 3,
            name: "Test User Updated".to_string(),
            picture: "https://example.com/avatar_updated.png".to_string(),
        };

        let (updated_user, is_new_updated) =
            User::create_or_update_existing_user(&pool, &google_claims_updated)
                .await
                .unwrap();

        // Verify it's an updated user
        assert!(!is_new_updated);
        assert_eq!(updated_user.id, initial_user.id); // Same ID
        assert_eq!(updated_user.name, "Test User Updated"); // Name updated
        assert_eq!(
            updated_user.avatar,
            "https://example.com/avatar_updated.png"
        ); // Avatar updated

        // Keys should remain the same
        assert_eq!(updated_user.public_key, initial_user.public_key);
        assert_eq!(updated_user.private_key, initial_user.private_key);

        // Clean up
        cleanup_test_user(&pool, initial_user.id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_user_by_id() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        let unique_id = Uuid::new_v4();
        let unique_email = format!("test_{}@gmail.com", unique_id);
        let unique_sub = format!("test_google_id_{}", unique_id);

        let google_claims = GoogleClaims {
            sub: unique_sub,
            email: unique_email,
            exp: 60 * 60 * 24 * 3,
            name: "Test User Get".to_string(),
            picture: "https://example.com/avatar_get.png".to_string(),
        };

        // Create user first
        let created_user = User::create_new_user(&pool, &google_claims).await.unwrap();

        // Get user by ID
        let fetched_user = User::get_user_by_id(&pool, created_user.id).await.unwrap();

        // Verify fetched user matches created user
        assert_eq!(fetched_user.id, created_user.id);
        assert_eq!(fetched_user.name, created_user.name);
        assert_eq!(fetched_user.email, created_user.email);
        assert_eq!(fetched_user.public_key, created_user.public_key);
        assert_eq!(fetched_user.private_key, created_user.private_key);

        // Clean up
        cleanup_test_user(&pool, created_user.id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_user_by_id_nonexistent() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        // Generate a random UUID that shouldn't exist in the database
        let nonexistent_id = Uuid::new_v4();

        // Attempt to get user by non-existent ID
        let result = User::get_user_by_id(&pool, nonexistent_id).await;

        // Verify the error is RowNotFound
        assert!(result.is_err());
        match result {
            Err(sqlx::Error::RowNotFound) => (),
            _ => panic!("Expected RowNotFound error"),
        }

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_two_users_balance() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        // Create two users
        let user1_claims = GoogleClaims {
            sub: format!("test_google_id_{}", Uuid::new_v4()),
            email: format!("test_{}@gmail.com", Uuid::new_v4()),
            exp: 60 * 60 * 24 * 3,
            name: "Test User 1".to_string(),
            picture: "https://example.com/avatar1.png".to_string(),
        };

        let user2_claims = GoogleClaims {
            sub: format!("test_google_id_{}", Uuid::new_v4()),
            email: format!("test_{}@gmail.com", Uuid::new_v4()),
            exp: 60 * 60 * 24 * 3,
            name: "Test User 2".to_string(),
            picture: "https://example.com/avatar2.png".to_string(),
        };

        let user1 = User::create_new_user(&pool, &user1_claims).await.unwrap();
        let user2 = User::create_new_user(&pool, &user2_claims).await.unwrap();

        // Get balances
        let (user1_balance, user2_balance) = User::get_two_users_balance(&pool, user1.id, user2.id)
            .await
            .unwrap();

        // Verify initial balances are zero
        assert_eq!(user1_balance, Decimal::ZERO);
        assert_eq!(user2_balance, Decimal::ZERO);

        // Clean up
        cleanup_test_user(&pool, user1.id).await;
        cleanup_test_user(&pool, user2.id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_two_users_balance_one_nonexistent() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        // Create one user
        let user1_claims = GoogleClaims {
            sub: format!("test_google_id_{}", Uuid::new_v4()),
            email: format!("test_{}@gmail.com", Uuid::new_v4()),
            exp: 60 * 60 * 24 * 3,
            name: "Test User 1".to_string(),
            picture: "https://example.com/avatar1.png".to_string(),
        };

        let user1 = User::create_new_user(&pool, &user1_claims).await.unwrap();
        let nonexistent_id = Uuid::new_v4();

        // Get balances with one nonexistent user
        let result = User::get_two_users_balance(&pool, user1.id, nonexistent_id).await;

        // Should fail since one user doesn't exist
        assert!(result.is_err());
        match result {
            Err(sqlx::Error::RowNotFound) => (),
            _ => panic!("Expected RowNotFound error"),
        }

        // Clean up
        cleanup_test_user(&pool, user1.id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_create_new_user_with_empty_fields() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        let unique_id = Uuid::new_v4();
        let unique_sub = format!("test_google_id_{}", unique_id);

        // Test with empty name and picture
        let google_claims = GoogleClaims {
            sub: unique_sub,
            email: format!("test_{}@gmail.com", unique_id),
            exp: 60 * 60 * 24 * 3,
            name: "".to_string(),
            picture: "".to_string(),
        };

        let user = User::create_new_user(&pool, &google_claims).await.unwrap();

        assert_eq!(user.name, "");
        assert_eq!(user.avatar, "");
        assert!(!user.private_key.is_empty());
        assert!(!user.public_key.is_empty());

        // Clean up
        cleanup_test_user(&pool, user.id).await;
        pool.close().await;
    }
}
