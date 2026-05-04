use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, PgPool, Postgres};
use utility_helpers::{ log_error, log_info,};
use uuid::Uuid;
use chrono::Utc;


use crate::schema::enums::OrderType;

use super::enums::{OrderSide, OrderStatus, Outcome};

// need serialize for message pack
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub market_id: Uuid,
    pub side: OrderSide,
    pub outcome: Outcome,
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub status: OrderStatus,
    pub order_type: OrderType,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// extend order struct with new fields
#[derive(Debug, Serialize, sqlx::FromRow, Clone)]
pub struct OrderWithMarket {
    pub id: Uuid,
    pub user_id: Uuid,
    pub market_id: Uuid,
    pub side: OrderSide,
    pub outcome: Outcome,
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub status: OrderStatus,
    pub order_type: OrderType,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub liquidity_b: Decimal,
    pub q_yes: Decimal,  // Added LMSR state
    pub q_no: Decimal,   // Added LMSR state
}

impl From<OrderWithMarket> for Order {
    fn from(order: OrderWithMarket) -> Self {
        Order {
            id: order.id,
            user_id: order.user_id,
            market_id: order.market_id,
            side: order.side,
            outcome: order.outcome,
            price: order.price,
            quantity: order.quantity,
            filled_quantity: order.filled_quantity,
            status: order.status,
            created_at: order.created_at,
            updated_at: order.updated_at,
            order_type: order.order_type,
        }
    }
}

impl Order {
  pub async fn create_order(
    user_id: Uuid,
    market_id: Uuid,
    price: Decimal,
    quantity: Decimal,
    side: OrderSide,
    outcome_side: Outcome,
    order_type: OrderType,
    pool: &PgPool,
) -> Result<Order, sqlx::Error> {
    
    let total_start = Utc::now();
    log_info!("📝 [CREATE ORDER] Starting order creation");
    log_info!("👤 User: {}, 📊 Market: {}, 💰 Price: {}, 📦 Qty: {}", 
        user_id, market_id, price, quantity);
    log_info!("🔄 Side: {:?}, 🎯 Outcome: {:?}, 📋 Type: {:?}", 
        side, outcome_side, order_type);

    // Connection acquisition time
    let conn_start = Utc::now();
    let mut conn = pool.acquire().await?;
    let conn_elapsed = Utc::now().signed_duration_since(conn_start);
    log_info!("🔌 Connection acquired in: {} ms", conn_elapsed.num_milliseconds());

    // Foreign key checks (implicitly happens during INSERT)
    let fk_start = Utc::now();
    
    // The INSERT query
    let query_start = Utc::now();
    log_info!("⚡ INSERT query started at: {}", query_start.to_rfc3339());
    
    let order = sqlx::query_as!(
        Order,
        r#"
        INSERT INTO "polymarket"."orders"
        (user_id, market_id, price, quantity, side, outcome, order_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING 
        id, user_id, market_id,
        outcome as "outcome: Outcome",
        price, quantity, filled_quantity,
        status as "status: OrderStatus",
        side as "side: OrderSide",            
        created_at, updated_at,
        order_type as "order_type: OrderType"
        "#,
        user_id,
        market_id,
        price,
        
        quantity,
        side as _,
        outcome_side as _,
        order_type as _,
    )
    .fetch_one(&mut *conn)
    .await?;

    let query_elapsed = Utc::now().signed_duration_since(query_start);
    let fk_elapsed = Utc::now().signed_duration_since(fk_start);
    let total_elapsed = Utc::now().signed_duration_since(total_start);

    log_info!("✅ INSERT completed in: {} ms", query_elapsed.num_milliseconds());
    log_info!("🔍 Foreign key checks: ~{} ms", (fk_elapsed.num_milliseconds() - query_elapsed.num_milliseconds()).abs());
    log_info!("🆔 Order ID: {}", order.id);
    log_info!("⏱️  BREAKDOWN: Conn={}ms + Insert={}ms = TOTAL={}ms",
        conn_elapsed.num_milliseconds(),
        query_elapsed.num_milliseconds(),
        total_elapsed.num_milliseconds()
    );
    log_info!("✅ [COMPLETE] Order created in: {} ms", total_elapsed.num_milliseconds());
    log_info!("─────────────────────────────────────");

    log_info!("Order created - {:?}", order.id);
    Ok(order)
}

    pub async fn delete_order_by_id(order_id: Uuid, pool: &PgPool) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            DELETE FROM polymarket.orders
            WHERE id = $1
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            order_type as "order_type: OrderType",
            created_at, updated_at
            "#,
            order_id
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order deleted - {:?}", order.id);
        Ok(order)
    }

    pub async fn update_order_status(
        order_id: Uuid,
        status: OrderStatus,
        pool: &PgPool,
    ) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            UPDATE polymarket.orders
            SET status = $1
            WHERE id = $2
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            order_type as "order_type: OrderType",
            created_at, updated_at
            "#,
            status as _,
            order_id
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order updated - {:?}", order.id);
        Ok(order)
    }

    pub async fn find_order_by_id(
        order_id: Uuid,
        pool: &PgPool,
    ) -> Result<Option<Order>, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            SELECT 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            order_type as "order_type: OrderType",
            created_at, updated_at    
            FROM polymarket.orders
            WHERE id = $1
            "#,
            order_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(order)
    }

    pub async fn find_order_by_id_and_status(
        order_id: Uuid,
        status: OrderStatus,
        pool: &PgPool,
    ) -> Result<Option<Order>, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            SELECT 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            order_type as "order_type: OrderType",
            created_at, updated_at            
            FROM polymarket.orders
            WHERE id = $1 AND status = $2
            "#,
            order_id,
            status as _
        )
        .fetch_optional(pool)
        .await?;

        Ok(order)
    }


    
 pub async fn find_order_by_id_with_market(
    order_id: Uuid,
    pool: &PgPool,
) -> Result<OrderWithMarket, sqlx::Error> {
    let order = sqlx::query_as!(
        OrderWithMarket,
        r#"
        SELECT 
            o.id, o.user_id, o.market_id,
            o.outcome as "outcome: Outcome",
            o.price, o.quantity, o.filled_quantity,
            o.status as "status: OrderStatus",
            o.side as "side: OrderSide",
            o.created_at, o.updated_at, m.liquidity_b,
            o.order_type as "order_type: OrderType",
            m.q_yes,
            m.q_no
        FROM polymarket.orders o
        LEFT JOIN polymarket.markets m ON o.market_id = m.id
        WHERE o.id = $1
        "#,
        order_id
    )
    .fetch_one(pool)
    .await?;

    log_info!("✅ Order found - {:?}", order.id);
    Ok(order)
}

pub async fn get_all_open_orders(pool: &PgPool) -> Result<Vec<OrderWithMarket>, sqlx::Error> {
    let orders = sqlx::query_as!(
        OrderWithMarket,
        r#"
        SELECT 
            o.id, o.user_id, o.market_id,
            o.outcome as "outcome: Outcome",
            o.price, o.quantity, o.filled_quantity,
            o.status as "status: OrderStatus",
            o.side as "side: OrderSide",
            o.created_at, o.updated_at, m.liquidity_b,
            o.order_type as "order_type: OrderType",
            m.q_yes,
            m.q_no
        FROM polymarket.orders o
        JOIN polymarket.markets m ON o.market_id = m.id
        WHERE o.status = 'open'::polymarket.order_status         
        "#,
    )
    .fetch_all(pool)
    .await?;

    log_info!("📋 Found {} open orders", orders.len());
    Ok(orders)
}

pub async fn get_all_open_or_unspecified_orders(
    pool: &PgPool,
) -> Result<Vec<OrderWithMarket>, sqlx::Error> {
    let orders = sqlx::query_as!(
        OrderWithMarket,
        r#"
        SELECT 
            o.id, o.user_id, o.market_id,
            o.outcome as "outcome: Outcome",
            o.price, o.quantity, o.filled_quantity,
            o.status as "status: OrderStatus",
            o.side as "side: OrderSide",
            o.created_at, o.updated_at, m.liquidity_b,
            o.order_type as "order_type: OrderType",
            m.q_yes,
            m.q_no
        FROM polymarket.orders o
        JOIN polymarket.markets m ON o.market_id = m.id
        WHERE o.status IN ('open'::polymarket.order_status, 'unspecified'::polymarket.order_status)
        "#,
    )
    .fetch_all(pool)
    .await?;

    log_info!("📋 Found {} open/unspecified orders", orders.len());
    Ok(orders)
}


pub async fn get_order_by_status(
    pool: &PgPool,
    status: OrderStatus,
) -> Result<Vec<OrderWithMarket>, sqlx::Error> {
    let orders = sqlx::query_as!(
        OrderWithMarket,
        r#"
        SELECT 
            o.id, 
            o.user_id, 
            o.market_id,
            o.outcome as "outcome: Outcome",
            o.price, 
            o.quantity, 
            o.filled_quantity,
            o.status as "status: OrderStatus",
            o.side as "side: OrderSide",
            o.created_at, 
            o.updated_at, 
            m.liquidity_b,
            o.order_type as "order_type: OrderType",
            m.q_yes,      -- Added LMSR state
            m.q_no        -- Added LMSR state
        FROM polymarket.orders o
        JOIN polymarket.markets m ON o.market_id = m.id                
        WHERE o.status = $1
        "#,
        status as _
    )
    .fetch_all(pool)
    .await?;

    Ok(orders)
}


    pub async fn update(&self, pool: &PgPool) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            UPDATE "polymarket"."orders"
            SET 
                user_id = $1,
                market_id = $2,
                side = $3,
                outcome = $4,
                price = $5,
                quantity = $6,
                filled_quantity = $7,
                status = $8,
                order_type = $9
            WHERE id = $10
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            order_type as "order_type: OrderType",
            created_at, updated_at
            "#,
            self.user_id,
            self.market_id,
            self.side as _,
            self.outcome as _,
            self.price,
            self.quantity,
            self.filled_quantity,
            self.status as _,
            self.order_type as _,
            self.id,
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order updated - {:?}", order.id);
        Ok(order)
    }

    pub async fn get_buyer_and_seller_user_id(
        pg_pool: &sqlx::PgPool,
        buy_order_id: Uuid,
        sell_order_id: Uuid,
    ) -> Result<(Uuid, Uuid), sqlx::Error> {
        let order = sqlx::query!(
            r#"
            SELECT user_id FROM polymarket.orders
            WHERE id = $1 OR id = $2
            "#,
            buy_order_id,
            sell_order_id
        )
        .fetch_all(pg_pool)
        .await?;

        if order.len() != 2 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok((order[0].user_id, order[1].user_id))
    }

    pub async fn get_order_user_id(pool: &PgPool, order_id: Uuid) -> Result<Uuid, sqlx::Error> {
        let user_id = sqlx::query!(
            r#"
            SELECT user_id FROM polymarket.orders
            WHERE id = $1
            "#,
            order_id
        )
        .fetch_one(pool)
        .await?;

        Ok(user_id.user_id)
    }

    pub async fn update_order_status_and_filled_quantity(
        pool: &PgPool,
        order_id: Uuid,
        order_status: OrderStatus,
        new_filled_quantity: Decimal,
    ) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            UPDATE polymarket.orders
            SET status = $1, filled_quantity = $2
            WHERE id = $3
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            order_type as "order_type: OrderType",
            created_at, updated_at
            "#,
            order_status as _,
            new_filled_quantity,
            order_id
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order updated - {:?}", order.id);
        Ok(order)
    }

    pub async fn get_user_orders_by_paginated(
        pool: &PgPool,
        user_id: Uuid,
        status: OrderStatus,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Order>, u32), sqlx::Error> {

 let query_start = Utc::now();
    log_info!("Query start: {}", query_start.to_rfc3339());

        let offset = (page - 1) * page_size;

        let total_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM polymarket.orders
            WHERE user_id = $1 AND status = $2
            "#,
            user_id,
            status as _
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

    let elapsed = Utc::now().signed_duration_since(query_start);
    log_info!("Query elapsed (ms): {}", elapsed.num_milliseconds());
let query_start = Utc::now();


        let total_pages = (total_count as u32 + page_size - 1) / page_size;

        let orders = sqlx::query_as!(
            Order,
            r#"
            SELECT
                id,
                user_id,
                market_id,
                outcome as "outcome: Outcome",
                price,
                quantity,
                filled_quantity,
                status as "status: OrderStatus",
                side as "side: OrderSide",
                order_type as "order_type: OrderType",
                created_at,
                updated_at
            FROM polymarket.orders
            WHERE user_id = $1 AND status = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            user_id,
            status as _,
            page_size as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?;


    let elapsed = Utc::now().signed_duration_since(query_start);
    log_info!("Query elapsed (ms): {}", elapsed.num_milliseconds());

        Ok((orders, total_pages))
    }





pub async fn get_user_orders_by_market_paginated(
    pool: &PgPool,
    user_id: Uuid,
    market_id: Uuid,
    page: u32,
    page_size: u32,
    status: Option<OrderStatus>,
) -> Result<(Vec<Order>, u32), sqlx::Error> {

    let function_start = Utc::now();
    log_info!("🔍 [START] Getting orders for user={}, market={}, page={}", user_id, market_id, page);

    let offset = (page - 1) * page_size;
    
    // COUNT query timing
    let count_start = Utc::now();
    log_info!("📊 COUNT query started at: {}", count_start.to_rfc3339());
    
    let total_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM polymarket.orders
        WHERE user_id = $1 AND market_id = $2
        "#,
        user_id,
        market_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let count_elapsed = Utc::now().signed_duration_since(count_start);
    log_info!("✅ COUNT query completed in: {} ms", count_elapsed.num_milliseconds());

    let total_pages = (total_count as u32 + page_size - 1) / page_size;
    log_info!("📦 Total orders: {}, Total pages: {}", total_count, total_pages);

    if let Some(status) = status {
        // Data query with status filter
        let data_start = Utc::now();
        log_info!("🔎 DATA query (with status filter) started at: {}", data_start.to_rfc3339());
        log_info!("🎯 Status filter: {:?}", status);

        let orders = sqlx::query_as!(
            Order,
            r#"
            SELECT
                id, user_id, market_id,
                outcome as "outcome: Outcome",
                price, 
                quantity, 
                filled_quantity,
                status as "status: OrderStatus",
                side as "side: OrderSide",
                order_type as "order_type: OrderType",
                created_at, updated_at
            FROM polymarket.orders
            WHERE user_id = $1 AND market_id = $2 AND status = $3
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
            user_id,
            market_id,
            status as _,
            page_size as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?;

        let data_elapsed = Utc::now().signed_duration_since(data_start);
        log_info!("⚡ DATA query completed in: {} ms", data_elapsed.num_milliseconds());
        log_info!("📋 Found {} orders for page {}", orders.len(), page);

        let total_elapsed = Utc::now().signed_duration_since(function_start);
        log_info!("✅ [COMPLETE] Total time: {} ms", total_elapsed.num_milliseconds());
        log_info!("─────────────────────────────────────");

        Ok((orders, total_pages))

    } else {
        // Data query without status filter
        let data_start = Utc::now();
        log_info!("🔎 DATA query (no status filter) started at: {}", data_start.to_rfc3339());

        let orders = sqlx::query_as!(
            Order,
            r#"
            SELECT
                id, user_id, market_id,
                outcome as "outcome: Outcome",
                price, 
                quantity, 
                filled_quantity,
                status as "status: OrderStatus",
                side as "side: OrderSide",
                order_type as "order_type: OrderType",
                created_at, updated_at
            FROM polymarket.orders
            WHERE user_id = $1 AND market_id = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            user_id,
            market_id,
            page_size as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?;

        let data_elapsed = Utc::now().signed_duration_since(data_start);
        log_info!("⚡ DATA query completed in: {} ms", data_elapsed.num_milliseconds());
        log_info!("📋 Found {} orders for page {}", orders.len(), page);

        let total_elapsed = Utc::now().signed_duration_since(function_start);
        log_info!("✅ [COMPLETE] Total time: {} ms", total_elapsed.num_milliseconds());
        log_info!("─────────────────────────────────────");

        Ok((orders, total_pages))
    }
}





    pub async fn update_order_status_and_quantity(
        pool: &PgPool,
        order_id: Uuid,
        order_status: OrderStatus,
        new_quantity: Decimal,
    ) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            UPDATE polymarket.orders
            SET status = $1, quantity = $2
            WHERE id = $3
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            order_type as "order_type: OrderType",
            side as "side: OrderSide",
            created_at, updated_at
            "#,
            order_status as _,
            new_quantity,
            order_id
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order updated - {:?}", order.id);
        Ok(order)
    }

 pub async fn insert_multiple_orders(
    orders: &Vec<Order>,
    pool: &PgPool,
) -> Result<Vec<Order>, sqlx::Error> {
    
    let total_start = Utc::now();
    log_info!("📦 [BATCH INSERT] Starting batch insert of {} orders", orders.len());
    
    let tx_start = Utc::now();
    let mut transaction = pool.begin().await?;
    let tx_elapsed = Utc::now().signed_duration_since(tx_start);
    log_info!("🔌 Transaction started in: {} ms", tx_elapsed.num_milliseconds());

    let mut inserted_orders = Vec::new();
    let mut total_insert_time = 0;
    
    for (i, order) in orders.iter().enumerate() {
        let insert_start = Utc::now();
        
        let inserted_order = sqlx::query_as!(
            Order,
            r#"
            INSERT INTO "polymarket"."orders"
            (user_id, market_id, price, quantity, side, outcome, order_type, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            created_at, updated_at,
            order_type as "order_type: OrderType"        
            "#,
            order.user_id,
            order.market_id,
            order.price,
            order.quantity,
            order.side as _,
            order.outcome as _,
            order.order_type as _,
            order.status as _,
        )
        .fetch_one(&mut *transaction)
        .await?;

        let insert_elapsed = Utc::now().signed_duration_since(insert_start);
        total_insert_time += insert_elapsed.num_milliseconds();
        
        log_info!("  ✅ Order {}/{} inserted in {} ms", i+1, orders.len(), insert_elapsed.num_milliseconds());
        inserted_orders.push(inserted_order);
    }

    let commit_start = Utc::now();
    transaction.commit().await?;
    let commit_elapsed = Utc::now().signed_duration_since(commit_start);
    
    let total_elapsed = Utc::now().signed_duration_since(total_start);
    
    log_info!("📊 BATCH INSERT SUMMARY:");
    log_info!("  📦 Total orders: {}", orders.len());
    log_info!("  ⏱️  Average per insert: {} ms", total_insert_time / orders.len() as i64);
    log_info!("  🔄 Transaction overhead: {} ms", tx_elapsed.num_milliseconds() + commit_elapsed.num_milliseconds());
    log_info!("  ⏱️  TOTAL TIME: {} ms", total_elapsed.num_milliseconds());
    log_info!("  ⚡ Orders/second: {:.2}", (orders.len() as f64 / (total_elapsed.num_milliseconds() as f64 / 1000.0)));
    log_info!("─────────────────────────────────────");

    Ok(inserted_orders)
}

// pub async fn insert_multiple_orders(
//     orders: &[Order],
//     pool: &PgPool,
// ) -> Result<Vec<Order>, sqlx::Error> {
//     let mut transaction = pool.begin().await?;

//     // Extract columns into vectors for UNNEST()
//     let user_ids: Vec<_> = orders.iter().map(|o| o.user_id).collect();
//     let market_ids: Vec<_> = orders.iter().map(|o| o.market_id).collect();
//     let prices: Vec<_> = orders.iter().map(|o| o.price).collect();
//     let quantities: Vec<_> = orders.iter().map(|o| o.quantity).collect();
//     let sides: Vec<_> = orders.iter().map(|o| o.side as _).collect();
//     let outcomes: Vec<_> = orders.iter().map(|o| o.outcome as _).collect();
//     let order_types: Vec<_> = orders.iter().map(|o| o.order_type as _).collect();
//     let statuses: Vec<_> = orders.iter().map(|o| o.status as _).collect();

//     let inserted_orders = sqlx::query_as!(
//         Order,
//         r#"
//         INSERT INTO polymarket.orders (user_id, market_id, price, quantity, side, outcome, order_type, status)
//         SELECT *
//         FROM UNNEST(
//             $1::uuid[],
//             $2::uuid[],
//             $3::numeric[],
//             $4::numeric[],
//    $5::text[]::order_side,
//     $6::text[]::outcome,
//     $7::text[]::order_type,
//     $8::text[]::order_status  

//         )
//         RETURNING
//             id, user_id, market_id,
//             outcome as "outcome: Outcome",
//             price, quantity, filled_quantity,
//             status as "status: OrderStatus",
//             side as "side: OrderSide",
//             created_at, updated_at,
//             order_type as "order_type: OrderType"
//         "#,
//         &user_ids,
//         &market_ids,
//         &prices,
//         &quantities,
//         &sides,
//         &outcomes,
//         &order_types,
//         &statuses
//     )
//     .fetch_all(&mut *transaction)
//     .await?;

//     transaction.commit().await?;
//     Ok(inserted_orders)
// }




    pub async fn get_user_order_locked_funds(
        executor: impl Executor<'_, Database = Postgres>,
        user_id: Uuid,
    ) -> Result<Decimal, sqlx::Error> {
        let total_amount = sqlx::query_scalar!(
            r#"
        SELECT SUM(quantity) FROM polymarket.orders 
                WHERE user_id = $1 
                AND side = 'buy'::polymarket.order_side
                AND status = 'open'::polymarket.order_status
            "#,
            user_id,
        )
        .fetch_one(executor)
        .await?
        .unwrap_or(Decimal::ZERO);

        Ok(total_amount)
    }


    
    pub async fn get_user_locked_stokes(
        executor: impl Executor<'_, Database = Postgres>,
        user_id: Uuid,
        outcome_side: Outcome,
    ) -> Result<Decimal, sqlx::Error> {
        let locked_stokes = sqlx::query_scalar!(
            r#"
            SELECT SUM(quantity) FROM polymarket.orders 
                WHERE user_id = $1
                AND outcome = $2
                AND side = 'sell'::polymarket.order_side
                AND status = 'open'::polymarket.order_status
            "#,
            user_id,
            outcome_side as _
        )
        .fetch_one(executor)
        .await?
        .unwrap_or(Decimal::ZERO);
        Ok(locked_stokes)
    }

// Fetch all OPEN LIMIT orders for a market — the price check happens in Rust
// so we pull all of them and filter, keeping the query simple
pub async fn get_open_limit_orders_for_market(
    pool: &PgPool,
    market_id: Uuid,
) -> Result<Vec<Order>, sqlx::Error> {
    sqlx::query_as!(
        Order,
        r#"
        SELECT 
            id,
            user_id,
            market_id,
            outcome as "outcome: Outcome",      -- 👈 Add this
            price,
            quantity,
            filled_quantity,
            status as "status: OrderStatus",    -- 👈 Add this
            side as "side: OrderSide",          -- 👈 Add this
            order_type as "order_type: OrderType", -- 👈 Add this
            created_at,
            updated_at
        FROM "polymarket"."orders"
        WHERE market_id = $1
          AND status = 'open'
          AND order_type = 'limit'
        ORDER BY created_at ASC
        "#,
        market_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_open_stop_orders_for_market(
    pool: &PgPool,
    market_id: Uuid,
) -> Result<Vec<Order>, sqlx::Error> {
    sqlx::query_as!(
        Order,
        r#"
        SELECT 
            id,
            user_id,
            market_id,
            outcome as "outcome: Outcome",      -- 👈 Add this
            price,
            quantity,
            filled_quantity,
            status as "status: OrderStatus",    -- 👈 Add this
            side as "side: OrderSide",          -- 👈 Add this
            order_type as "order_type: OrderType", -- 👈 Add this
            created_at,
            updated_at
        FROM "polymarket"."orders"
        WHERE market_id = $1
          AND status = 'open'
          AND order_type = 'stop_loss'
        ORDER BY created_at ASC
        "#,
        market_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_user_market_buy_orders(
    pool: &PgPool,
    user_id: Uuid,
    market_id: Uuid,
) -> Result<Vec<Order>, sqlx::Error> {
    sqlx::query_as!(
        Order,
        r#"
        SELECT
            id,
            user_id,
            market_id,
            outcome as "outcome: Outcome",
            price,
            quantity,
            filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            order_type as "order_type: OrderType",
            created_at,
            updated_at
        FROM "polymarket"."orders"
        WHERE user_id = $1
          AND market_id = $2
          AND side = 'buy'
        ORDER BY created_at ASC
        "#,
        user_id,
        market_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_user_market_sell_orders_except(
    pool: &PgPool,
    user_id: Uuid,
    market_id: Uuid,
    exclude_order_id: Uuid,
) -> Result<Vec<Order>, sqlx::Error> {
    sqlx::query_as!(
        Order,
        r#"
        SELECT
            id,
            user_id,
            market_id,
            outcome as "outcome: Outcome",
            price,
            quantity,
            filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            order_type as "order_type: OrderType",
            created_at,
            updated_at
        FROM "polymarket"."orders"
        WHERE user_id = $1
          AND market_id = $2
          AND side = 'sell'
          AND id != $3
        ORDER BY created_at ASC
        "#,
        user_id,
        market_id,
        exclude_order_id
    )
    .fetch_all(pool)
    .await
}









}







#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::DateTime;
    use utility_helpers::types::GoogleClaims;

    use super::*;
    use crate::schema::{market::Market, users::User};

    #[tokio::test]
    // #[ignore = "just like this"]
    async fn test_create_order() {
        dotenv::dotenv().ok();
        let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();

        let user = User::create_new_user(
            &pool,
            &GoogleClaims {
                email: "temp@gmail.com".to_string(),
                exp: 0,
                name: "temp".to_string(),
                picture: "temp".to_string(),
                sub: "temp".to_string(),
            },
        )
        .await
        .unwrap();

        let date_time = DateTime::parse_from_rfc3339("2025-06-20T12:28:33.675Z").unwrap();
        let market_expiry = date_time.naive_utc();

        let market = Market::create_new_market(
            "Test Market 0".to_string(),
            "Test Description".to_string(),
            "Test Logo".to_string(),
            Decimal::new(100, 2),
            market_expiry,
            &pool,
        )
        .await
        .unwrap();

        // values are taken from the database
        let user_id = user.id;
        let market_id = market.id;

        let price = Decimal::from_str("0.5").unwrap();
        let quantity = Decimal::from_str("1.0").unwrap();
        let side = OrderSide::BUY;

        let order = Order::create_order(
            user_id,
            market_id,
            price,
            quantity,
            side.clone(),
            Outcome::YES,
            OrderType::LIMIT,
            &pool,
        )
        .await
        .unwrap();

        assert_eq!(order.user_id, user_id);
        assert_eq!(order.market_id, market_id);
        assert_eq!(order.price, price);
        assert_eq!(order.quantity, quantity);
        assert_eq!(order.side, side);
        assert_eq!(order.filled_quantity, Decimal::ZERO);
        assert_eq!(order.status, OrderStatus::UNSPECIFIED);
        assert_eq!(order.outcome, Outcome::YES);
        assert_eq!(order.created_at, order.updated_at);

        // Clean up
        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."orders"
            WHERE id = $1
            "#,
            order.id
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."markets"
            WHERE id = $1
            "#,
            market.id
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."users"
            WHERE id = $1
            "#,
            user.id
        )
        .execute(&pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_update_order_status_filled_quantity() {
        dotenv::dotenv().ok();
        let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();

        let user = User::create_new_user(
            &pool,
            &GoogleClaims {
                email: "nami".to_string(),
                exp: 0,
                name: "nami".to_string(),
                picture: "nami".to_string(),
                sub: "nami".to_string(),
            },
        )
        .await
        .unwrap();

        let date_time = DateTime::parse_from_rfc3339("2025-06-20T12:28:33.675Z").unwrap();
        let market_expiry = date_time.naive_utc();

        let market = Market::create_new_market(
            "Test Market 0".to_string(),
            "Test Description".to_string(),
            "Test Logo".to_string(),
            Decimal::new(100, 2),
            market_expiry,
            &pool,
        )
        .await
        .unwrap();

        // values are taken from the database
        let user_id = user.id;
        let market_id = market.id;
        let price = Decimal::from_str("0.5").unwrap();
        let quantity = Decimal::from_str("1.0").unwrap();
        let side = OrderSide::BUY;
        let order = Order::create_order(
            user_id,
            market_id,
            price,
            quantity,
            side.clone(),
            Outcome::YES,
            OrderType::LIMIT,
            &pool,
        )
        .await
        .unwrap();

        assert_eq!(order.user_id, user_id);
        assert_eq!(order.market_id, market_id);
        assert_eq!(order.price, price);
        assert_eq!(order.quantity, quantity);
        assert_eq!(order.side, side);
        assert_eq!(order.filled_quantity, Decimal::ZERO);
        assert_eq!(order.status, OrderStatus::UNSPECIFIED);
        assert_eq!(order.outcome, Outcome::YES);

        // Update the order status to FILLED and set filled quantity
        let new_filled_quantity = Decimal::from_str("1.0").unwrap();
        let updated_order = Order::update_order_status_and_filled_quantity(
            &pool,
            order.id,
            OrderStatus::FILLED,
            new_filled_quantity,
        )
        .await
        .unwrap();

        assert_eq!(updated_order.id, order.id);
        assert_eq!(updated_order.status, OrderStatus::FILLED);
        assert_eq!(updated_order.filled_quantity, new_filled_quantity);

        // Clean up
        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."orders"
            WHERE id = $1
            "#,
            updated_order.id
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."markets"
            WHERE id = $1
            "#,
            market.id
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."users"
            WHERE id = $1
            "#,
            user.id
        )
        .execute(&pool)
        .await
        .unwrap();

        log_info!("Order updated - {:?}", updated_order.id);
    }
}
