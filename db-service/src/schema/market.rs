use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utility_helpers::{ log_error, log_info,};
use uuid::Uuid;
use chrono::Utc;

use super::enums::{MarketStatus, Outcome};
use crate::{
    pagination::PaginatedResponse,
    utils::{CronJobName, to_cron_expression},
};

// serialized by redis
#[derive(Debug, Serialize, sqlx::FromRow, Deserialize, Default)]
pub struct Market {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub logo: Vec<String>, 
    pub status: MarketStatus,
    pub liquidity_b: Decimal,
    pub final_outcome: Outcome,
    pub market_expiry: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,

    // New fields
    pub parent_id: Option<Uuid>,            // parent market if any
    pub is_event: bool,                     // true if this is a parent event
    pub child_market_ids: Option<Vec<Uuid>>, // list of child UUIDs
    pub category: Option<String>,           // e.g., Politics, Sports
    pub resolution_criteria: Option<String>, // settlement rules
    pub slug: Option<String>,               // URL-friendly identifier
    
    // LMSR State fields (outstanding shares)
    pub q_yes: Decimal,                     // Net Yes shares sold to traders
    pub q_no: Decimal,                      // Net No shares sold to traders
}

impl Market {
  pub async fn create_new_market(
    name: String,
    description: String,
    logo: Vec<String>, 
    liquidity_b: Decimal,
    market_expiry: NaiveDateTime,
    pg_pool: &PgPool,
    parent_id: Option<Uuid>,              // New
    is_event: bool,                        // New
    child_market_ids: Option<Vec<Uuid>>,   // New
    category: Option<String>,              // New
    resolution_criteria: Option<String>, 
     slug: Option<String>,   // New
) -> Result<Self, sqlx::Error> {
    let mut tx = pg_pool.begin().await?;
    let child_ids_slice: Option<&[Uuid]> = child_market_ids.as_deref();
    // For logo - do the same
let logo_slice: &[String] = &logo;

 let initial_q_yes = Decimal::ZERO;  // Or any other starting value
let initial_q_no = Decimal::ZERO;   // Or any other starting value

let market = sqlx::query_as!(
    Market,
    r#"
    INSERT INTO polymarket.markets (
        name,
        description,
        logo,
        liquidity_b,
        market_expiry,
        parent_id,
        is_event,
        child_market_ids,
        category,
        resolution_criteria,
        slug,
        q_yes,
        q_no
    ) VALUES (
        $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
    ) RETURNING 
        id,
        name,
        description,
        logo,
        status as "status: MarketStatus",
        final_outcome as "final_outcome: Outcome",
        liquidity_b,
        market_expiry,
        created_at,
        updated_at,
        parent_id,
        is_event,
        child_market_ids,
        category,
        resolution_criteria,
        slug,
        q_yes,
        q_no
    "#,
    name,
    description,
    logo_slice,
    liquidity_b,
    market_expiry,
    parent_id,
    is_event,
    child_ids_slice,
    category,
    resolution_criteria,
    slug,
    initial_q_yes,
    initial_q_no,
)
.fetch_one(&mut *tx)
.await?;

        // create cron
        let cron_name = CronJobName::CloseMarket(market.id).to_string();

        let cron_query = format!("SELECT polymarket.close_market('{}'::uuid);", market.id); // cron function
        let cron_expression = to_cron_expression(market.market_expiry);

        sqlx::query(
            r#"
            SELECT cron.schedule(
                $1, -- cron name
                $2, -- cron run time
                $3 -- cron function
            )
            "#,
        )
        .bind(cron_name)
        .bind(cron_expression)
        .bind(cron_query)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        log_info!("Market created: {}", market.id);
        Ok(market)
    }




  pub async fn get_all_markets(pg_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
    let markets = sqlx::query_as!(
        Market,
        r#"
        SELECT 
            id,
            name,
            description,
            logo,
            status as "status: MarketStatus",
            final_outcome as "final_outcome: Outcome",
            liquidity_b,
            market_expiry,
            created_at,
            updated_at,
            parent_id,
            is_event,
            child_market_ids,
            category,
            resolution_criteria,
            slug,
            q_yes,
            q_no
        FROM "polymarket"."markets"
        "#,
    )
    .fetch_all(pg_pool)
    .await?;

    Ok(markets)
}



pub async fn update_liquidity_b(
    pool: &PgPool,
    market_id: Uuid,
    amount: Decimal,
    is_adding: bool,  // true = add, false = deduct
) -> Result<(), sqlx::Error> {
    let final_amount = if is_adding {
        amount
    } else {
        -amount
    };

    sqlx::query!(
        r#"
        UPDATE polymarket.markets
        SET liquidity_b = liquidity_b + $1,
            updated_at = NOW()
        WHERE id = $2
        "#,
        final_amount,
        market_id
    )
    .execute(pool)
    .await?;

    Ok(())
}




pub async fn get_all_markets_paginated(
    pg_pool: &PgPool,
    page: u64,
    page_size: u64,
) -> Result<PaginatedResponse<Self>, sqlx::Error> {
    let offset = (page - 1) * page_size;

    let total_count = sqlx::query!(
        r#"
        SELECT COUNT(*) as total_count
        FROM "polymarket"."markets"
        "#,
    )
    .fetch_one(pg_pool)
    .await?
    .total_count
    .unwrap_or(0);

    let markets = sqlx::query_as!(
        Market,
        r#"
        SELECT 
            id,
            name,
            description,
            logo,
            status as "status: MarketStatus",
            final_outcome as "final_outcome: Outcome",
            liquidity_b,
            market_expiry,
            created_at,
            updated_at,
            parent_id,
            is_event,
            child_market_ids,
            category,
            resolution_criteria,
            slug,
            q_yes,
            q_no
        FROM "polymarket"."markets"
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        page_size as i64,
        offset as i64
    )
    .fetch_all(pg_pool)
    .await?;

    Ok(PaginatedResponse::new(
        markets,
        page,
        page_size,
        total_count as u64,
    ))
}



pub async fn get_all_market_by_status_paginated(
    pg_pool: &PgPool,
    status: MarketStatus,
    page: u64,
    page_size: u64,
) -> Result<PaginatedResponse<Self>, sqlx::Error> {
    let offset = (page - 1) * page_size;

    let query_start = Utc::now();
    log_info!("Query start: {}", query_start.to_rfc3339());

    let total_count = sqlx::query!(
        r#"
        SELECT COUNT(*) as total_count
        FROM "polymarket"."markets"
        WHERE status = $1
        "#,
        status as _
    )
    .fetch_one(pg_pool)
    .await?
    .total_count
    .unwrap_or(0);

    let query_start = Utc::now();
    log_info!("Query start: {}", query_start.to_rfc3339());

    let markets = sqlx::query_as!(
        Market,
        r#"
        SELECT 
            id,
            name,
            description,
            logo,
            status as "status: MarketStatus",
            final_outcome as "final_outcome: Outcome",
            liquidity_b,
            market_expiry,
            created_at,
            updated_at,
            parent_id,
            is_event,
            child_market_ids,
            category,
            resolution_criteria,
            slug,
            q_yes,
            q_no
        FROM "polymarket"."markets"
        WHERE status = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        status as _,
        page_size as i64,
        offset as i64
    )
    .fetch_all(pg_pool)
    .await?;

    let elapsed = Utc::now().signed_duration_since(query_start);
    log_info!("Query elapsed (ms): {}", elapsed.num_milliseconds());

    Ok(PaginatedResponse::new(
        markets,
        page,
        page_size,
        total_count as u64,
    ))
}



  pub async fn get_market_by_id(
    pg_pool: &PgPool,
    market_id: &Uuid,
) -> Result<Option<Self>, sqlx::Error> {
    let market = sqlx::query_as!(
        Market,
        r#"
        SELECT 
            id,
            name,
            description,
            logo,
            status as "status: MarketStatus",
            final_outcome as "final_outcome: Outcome",
            liquidity_b,
            market_expiry,
            created_at,
            updated_at,
            parent_id,
            is_event,
            child_market_ids,
            category,
            resolution_criteria,
            slug,
            q_yes,
            q_no
        FROM "polymarket"."markets"
        WHERE id = $1
        "#,
        market_id
    )
    .fetch_optional(pg_pool)
    .await?;

    Ok(market)
}


   pub async fn update_child_market_ids_field(
        parent_id: &Uuid,
        child_ids: &[Uuid],
        pg_pool: &PgPool,
    ) -> Result<(), sqlx::Error> {


// ✅ Correct - just use the reference directly
let slice: &[Uuid] = child_ids;

sqlx::query!(
    r#"
    UPDATE polymarket.markets
    SET child_market_ids = $1
    WHERE id = $2
    "#,
    slice,     // CORRECT
    parent_id
)
        .execute(pg_pool)
        .await?;

        Ok(())
    }


 pub async fn update_lmsr_state(
       pg_pool: &PgPool,
    market_id: Uuid,
    q_yes: Decimal,
    q_no: Decimal,
    
    liquidity_b: Decimal,  
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE polymarket.markets
        SET q_yes = $1,
            q_no = $2,
    
            liquidity_b = $3,
            updated_at = NOW()
        WHERE id = $4
        "#,
        q_yes,
        q_no,

        liquidity_b,
        market_id
    )
    .execute(  pg_pool)
    .await?;

    Ok(())
}


  pub async fn get_all_open_markets(pg_pool: &PgPool) -> Result<Vec<Market>, sqlx::Error> {
    let markets = sqlx::query_as!(
        Market,
        r#"
        SELECT 
            id,
            name,
            description,
            logo,
            status as "status: MarketStatus",
            final_outcome as "final_outcome: Outcome",
            liquidity_b,
            market_expiry,
            created_at,
            updated_at,
            parent_id,
            is_event,
            child_market_ids,
            category,
            resolution_criteria,
            slug,
            q_yes,
            q_no
        FROM "polymarket"."markets"
        WHERE status = 'open'::polymarket.market_status
        "#,
    )
    .fetch_all(pg_pool)
    .await?;

    Ok(markets)
}

pub async fn get_btcusdt_markets_updated_last_20min(
    pg_pool: &PgPool,
) -> Result<Vec<Market>, sqlx::Error> {

    let markets = sqlx::query_as!(
        Market,
        r#"
        SELECT 
            id,
            name,
            description,
            logo,
            status as "status: MarketStatus",
            final_outcome as "final_outcome: Outcome",
            liquidity_b,
            market_expiry,
            created_at,
            updated_at,
            parent_id,
            is_event,
            child_market_ids,
            category,
            resolution_criteria,
            slug,
            q_yes,
            q_no
        FROM "polymarket"."markets"
        WHERE name ILIKE '%BTCUSDT%'
          AND updated_at >= NOW() - INTERVAL '30 minutes'
        ORDER BY updated_at DESC
        "#,
    )
    .fetch_all(pg_pool)
    .await?;

    Ok(markets)
}




  pub async fn settle_market(
    pg_pool: &PgPool,
    market_id: &Uuid,
    final_outcome: Outcome,
    resolved_price: Option<f64>,  // ✅ Add resolved_price parameter
) -> Result<(), sqlx::Error> {
    let mut tx = pg_pool.begin().await?;

    // ✅ First, get the current resolution_criteria
    let current_criteria = sqlx::query!(
        r#"
        SELECT resolution_criteria
        FROM polymarket.markets
        WHERE id = $1
        "#,
        market_id
    )
    .fetch_optional(&mut *tx)
    .await?;

       // ✅ Update resolution_criteria with resolved_price if provided
    if let (Some(price), Some(criteria_row)) = (resolved_price, current_criteria) {
        if let Some(criteria_str) = criteria_row.resolution_criteria {
            if let Ok(mut criteria) = serde_json::from_str::<serde_json::Value>(&criteria_str) {
                criteria["resolved_price"] = serde_json::json!(price);
                let updated_criteria = criteria.to_string();
                
                sqlx::query!(
                    r#"
                    UPDATE polymarket.markets
                    SET resolution_criteria = $1
                    WHERE id = $2
                    "#,
                    updated_criteria,
                    market_id
                )
                .execute(&mut *tx)
                .await?;
            }
        }
    }


    // 1. Updating the market status to settled
    sqlx::query!(
        r#"
        UPDATE polymarket.markets
        SET status = 'settled'::polymarket.market_status,
            final_outcome = $2::polymarket.outcome,
            updated_at = NOW()
        WHERE id = $1
        "#,
        market_id,
        final_outcome as _
    )
    .execute(&mut *tx)
    .await?;

    // 2. Expiring all open orders in the market
    sqlx::query!(
        r#"
        UPDATE polymarket.orders
        SET status = 'expired'::polymarket.order_status
        WHERE market_id = $1 AND status in (
            'open'::polymarket.order_status,
            'partial_fill'::polymarket.order_status,
            'pending_update'::polymarket.order_status,
            'pending_cancel'::polymarket.order_status
        )
        "#,
        market_id
    )
    .execute(&mut *tx)
    .await?;

    // 3. Credit the balance to the user's holdings
    sqlx::query!(
        r#"
        UPDATE polymarket.users u
        SET balance = balance + (payout.total * 1) -- Each share is worth 1 after settlement
        FROM (
            SELECT user_id, SUM(shares) AS total
            FROM polymarket.user_holdings
            WHERE market_id = $1 AND outcome = $2::polymarket.outcome AND settled = false
            GROUP BY user_id
        ) AS payout
        WHERE u.id = payout.user_id
        "#,
        market_id,
        final_outcome as _
    )
    .execute(&mut *tx)
    .await?;

    // 4. Mark all holdings for this market as settled
    sqlx::query!(
        r#"
        UPDATE polymarket.user_holdings
        SET settled = true
        WHERE market_id = $1
        "#,
        market_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}


}

#[cfg(test)]
mod tests {
    use std::env;

    use chrono::DateTime;

    use super::*;

    #[tokio::test]
    async fn test_create_new_market() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pg_pool = PgPool::connect(&database_url).await.unwrap();

        let date_time = DateTime::parse_from_rfc3339("2025-06-20T12:28:33.675Z").unwrap();
        let market_expiry = date_time.naive_utc();

        let market = Market::create_new_market(
            "Test Market 0".to_string(),
            "Test Description".to_string(),
            "Test Logo".to_string(),
            Decimal::new(100, 2),
            market_expiry,
            &pg_pool,
        )
        .await
        .unwrap();

        assert_eq!(market.name, "Test Market 0");
        assert_eq!(market.description, "Test Description");
        assert_eq!(market.logo, "Test Logo");
        assert_eq!(market.liquidity_b, Decimal::new(100, 2));
        // Clean up the test market
        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."markets" 
            WHERE id = $1
            "#,
            market.id
        )
        .execute(&pg_pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_get_all_markets() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pg_pool = PgPool::connect(&database_url).await.unwrap();

        let markets = Market::get_all_markets(&pg_pool).await;

        assert!(markets.is_ok());
    }

    #[tokio::test]
    async fn test_get_all_markets_paginated() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pg_pool = PgPool::connect(&database_url).await.unwrap();

        let paginated_response = Market::get_all_markets_paginated(&pg_pool, 1, 10)
            .await
            .unwrap();
        assert_eq!(paginated_response.page_info.page, 1);
        assert_eq!(paginated_response.page_info.page_size, 10);
    }

    #[tokio::test]
    async fn test_get_market_by_id() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pg_pool = PgPool::connect(&database_url).await.unwrap();

        let date_time = DateTime::parse_from_rfc3339("2025-06-20T12:28:33.675Z").unwrap();
        let market_expiry = date_time.naive_utc();

        let market = Market::create_new_market(
            "Test Market 0".to_string(),
            "Test Description".to_string(),
            "Test Logo".to_string(),
            Decimal::new(100, 2),
            market_expiry,
            &pg_pool,
        )
        .await
        .unwrap();

        let fetched_market = Market::get_market_by_id(&pg_pool, &market.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(fetched_market.id, market.id);
        assert_eq!(fetched_market.name, market.name);
        assert_eq!(fetched_market.description, market.description);
        assert_eq!(fetched_market.logo, market.logo);
        assert_eq!(fetched_market.liquidity_b, market.liquidity_b);

        // Clean up the test market
        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."markets" 
            WHERE id = $1
            "#,
            market.id
        )
        .execute(&pg_pool)
        .await
        .unwrap();
    }
}
