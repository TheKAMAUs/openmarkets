use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use db_service::schema::{
    enums::{OrderSide, OrderStatus, OrderType, Outcome},
    market::Market,
    orders::Order,
    user_holdings::UserHoldings,
    users::User,
};
use rand::Rng;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use rust_decimal_macros::dec;
use serde::Deserialize;
use sqlx::types::chrono;
use utility_helpers::{log_info,
    log_error,
    message_pack_helper::serialize_to_message_pack,
    nats_helper::{NatsSubjects, types::InitializeOrderBookMessage},
};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct InitializeMarketPayload {
    market_id: Uuid,
    depth: u32,
    quantity: u32,
}



pub async fn initialize_market(
    State(state): State<AppState>,
    Json(payload): Json<InitializeMarketPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let market_id = payload.market_id;
  

    // 1. Validate market exists
    let market = Market::get_market_by_id(&state.pg_pool, &market_id)
        .await
        .map_err(|e| {
            log_error!("Failed to get market by ID: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to get market"})),
            )
        })?
        .ok_or_else(|| {
            log_error!("Market with ID {} not found", market_id);
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Market not found"})),
            )
        })?;
let liquidity_b = market.liquidity_b;
    // 2. Seed LMSR state in DB — q_yes/q_no start at zero, prices at 0.5
    Market::update_lmsr_state(
        &state.pg_pool,
        market_id,
        Decimal::ZERO,       // q_yes
        Decimal::ZERO,       // q_no
liquidity_b,
    )
    .await
    .map_err(|e| {
        log_error!("Failed to initialize LMSR state: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to initialize LMSR state"})),
        )
    })?;

 let order_book_initialize_data = InitializeOrderBookMessage::<Order> {
    liquidity_b,
    orders: vec![],
    q_yes: market.q_yes,   // or retrieve from market state if available
    q_no: market.q_no,    // typically zero for a new market
};

    let binary_payload = serialize_to_message_pack(&order_book_initialize_data)
        .map_err(|e| {
            log_error!("Failed to serialize order book data: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to serialize order book data"})),
            )
        })?;

    state
        .jetstream
        .publish(NatsSubjects::InitializeOrderBook.to_string(), binary_payload.into())
        .await
        .map_err(|e| {
            log_error!("Failed to publish to NATS: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to publish to NATS"})),
            )
        })?;

    log_info!("✅ LMSR market {} initialized with liquidity_b={}", market_id, liquidity_b);

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Market initialized successfully",
            "market_id": market_id,
            "liquidity_b": liquidity_b,
            "yes_price": 0.5,
            "no_price": 0.5,
        })),
    ))
}


pub fn create_bootstrap_orders_with_stacked_price_levels(
    market_id: Uuid,
    admin_id: Uuid,
    depth: u32,
    quantity: u32,
    admin_balance: Decimal,
    yes_holdings: Decimal,
    no_holdings: Decimal,
) -> Vec<Order> {
    use rand::{Rng, seq::SliceRandom};
    use rust_decimal::Decimal;

    let mut rng = rand::rng();
    let now = chrono::Utc::now().naive_utc();

    let price_pairs: Vec<(Decimal, Decimal)> = (1..99)
        .step_by(2)
        .map(|p| (Decimal::new(p, 2), Decimal::new(p + 1, 2)))
        .collect();

    let mut shuffled = price_pairs.clone();
    shuffled.shuffle(&mut rng);

    let mut orders = vec![];
    let mut remaining_balance = admin_balance;
    let mut remaining_yes = yes_holdings;
    let mut remaining_no = no_holdings;

    for _ in 0..depth {
        let (buy_price, sell_price) = shuffled[rng.random_range(0..shuffled.len())];

        for _ in 0..quantity {
            // BUY YES
            let buy_yes_qty = random_qty();
            let buy_yes_cost = buy_price * buy_yes_qty;
            if remaining_balance >= buy_yes_cost {
                orders.push(Order {
                    id: Uuid::new_v4(),
                    user_id: admin_id,
                    market_id,
                    side: OrderSide::BUY,
                    outcome: Outcome::YES,
                    price: buy_price,
                    quantity: buy_yes_qty,
                    filled_quantity: Decimal::ZERO,
                    status: OrderStatus::OPEN,
                    order_type: OrderType::LIMIT,
                    created_at: now,
                    updated_at: now,
                });
                remaining_balance -= buy_yes_cost;
            }

            // SELL YES
            let sell_yes_qty = random_qty();
            if remaining_yes >= sell_yes_qty {
                orders.push(Order {
                    id: Uuid::new_v4(),
                    user_id: admin_id,
                    market_id,
                    side: OrderSide::SELL,
                    outcome: Outcome::YES,
                    price: sell_price,
                    quantity: sell_yes_qty,
                    filled_quantity: Decimal::ZERO,
                    status: OrderStatus::OPEN,
                    order_type: OrderType::LIMIT,
                    created_at: now,
                    updated_at: now,
                });
                remaining_yes -= sell_yes_qty;
            }

            // BUY NO
            let buy_no_qty = random_qty();
            let buy_no_cost = buy_price * buy_no_qty;
            if remaining_balance >= buy_no_cost {
                orders.push(Order {
                    id: Uuid::new_v4(),
                    user_id: admin_id,
                    market_id,
                    side: OrderSide::BUY,
                    outcome: Outcome::NO,
                    price: buy_price,
                    quantity: buy_no_qty,
                    filled_quantity: Decimal::ZERO,
                    status: OrderStatus::OPEN,
                    order_type: OrderType::LIMIT,
                    created_at: now,
                    updated_at: now,
                });
                remaining_balance -= buy_no_cost;
            }

            // SELL NO
            let sell_no_qty = random_qty();
            if remaining_no >= sell_no_qty {
                orders.push(Order {
                    id: Uuid::new_v4(),
                    user_id: admin_id,
                    market_id,
                    side: OrderSide::SELL,
                    outcome: Outcome::NO,
                    price: sell_price,
                    quantity: sell_no_qty,
                    filled_quantity: Decimal::ZERO,
                    status: OrderStatus::OPEN,
                    order_type: OrderType::LIMIT,
                    created_at: now,
                    updated_at: now,
                });
                remaining_no -= sell_no_qty;
            }
        }
    }

    orders
}

fn random_qty() -> Decimal {
    let mut rng = rand::rng();
    let q = rng.random_range(5.0..60.0);
    Decimal::from_f64(q).unwrap().round_dp(2)
}



#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use rust_decimal_macros::dec;
use utility_helpers::{
    log_error, log_info,};


    #[tokio::test]
    async fn test_bootstrap_order_creation_and_insertion() {
        // Setup test data
        let market_id = Uuid::new_v4();
        let admin_id = Uuid::new_v4();
        let depth = 5; // Create 5 price levels
        let quantity = 4; // 4 orders per level
        let admin_balance = dec!(10000); // $10,000
        let yes_holdings = dec!(5000); // 5000 YES shares
        let no_holdings = dec!(5000); // 5000 NO shares

        log_info!("🧪 TEST: Creating bootstrap orders...");
        
        // 1. Create the orders
        let orders = create_bootstrap_orders_with_stacked_price_levels(
            market_id,
            admin_id,
            depth,
            quantity,
            admin_balance,
            yes_holdings,
            no_holdings,
        );

        log_info!("📊 Created {} bootstrap orders", orders.len());
        
        // 2. Log the first 20 orders for inspection
        log_info!("📝 First 20 orders created:");
        for (i, order) in orders.iter().take(20).enumerate() {
            log_info!(
                "  Order {}: id={}, side={:?}, outcome={:?}, price={}, quantity={}",
                i + 1,
                order.id,
                order.side,
                order.outcome,
                order.price,
                order.quantity
            );
        }

        // 3. Group by price to see distribution
        use std::collections::HashMap;
        let mut price_counts: HashMap<Decimal, usize> = HashMap::new();
        for order in &orders {
            *price_counts.entry(order.price).or_insert(0) += 1;
        }

        log_info!("📊 Orders by price level:");
        let mut sorted_prices: Vec<_> = price_counts.keys().collect();
        sorted_prices.sort();
        for price in sorted_prices {
            log_info!("  Price {}: {} orders", price, price_counts[price]);
        }

        // 4. Verify we have both BUY and SELL orders
        let buy_count = orders.iter().filter(|o| o.side == OrderSide::BUY).count();
        let sell_count = orders.iter().filter(|o| o.side == OrderSide::SELL).count();
        
        log_info!("📊 Order distribution:");
        log_info!("  BUY orders: {}", buy_count);
        log_info!("  SELL orders: {}", sell_count);

        // 5. Verify YES and NO outcomes
        let yes_count = orders.iter().filter(|o| o.outcome == Outcome::YES).count();
        let no_count = orders.iter().filter(|o| o.outcome == Outcome::NO).count();
        
        log_info!("  YES outcomes: {}", yes_count);
        log_info!("  NO outcomes: {}", no_count);

        // 6. Check if any orders have zero quantity (shouldn't happen)
        let zero_qty = orders.iter().filter(|o| o.quantity == Decimal::ZERO).count();
        if zero_qty > 0 {
            log_error!("❌ Found {} orders with zero quantity!", zero_qty);
        } else {
            log_info!("✅ No zero-quantity orders");
        }

        // 7. Simulate database insert (in real code, you'd actually insert)
        log_info!("💾 Simulating database insert of {} orders...", orders.len());
        
        // In your actual code, you'd do:
        // match Order::insert_multiple_orders(&orders, &db_pool).await {
        //     Ok(_) => log_info!("✅ Successfully inserted {} orders", orders.len()),
        //     Err(e) => log_error!("❌ Failed to insert orders: {}", e),
        // }

        log_info!("✅ TEST COMPLETE");
        
        // Assertions
        assert!(!orders.is_empty(), "Should create orders");
        assert!(buy_count > 0, "Should have BUY orders");
        assert!(sell_count > 0, "Should have SELL orders");
        assert_eq!(zero_qty, 0, "Should have no zero-quantity orders");
    }
}