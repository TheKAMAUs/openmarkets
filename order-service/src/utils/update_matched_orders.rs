use std::sync::Arc;

use db_service::schema::{
    enums::{OrderSide, OrderStatus,Outcome},
    orders::Order,
    user_holdings::UserHoldings,
    user_trades::UserTrades,
    users::User,
    market::Market,
    liquidity::LpPosition,
    market_state_history::MarketStateHistory,
};
use rust_decimal::Decimal;
use std::collections::VecDeque;

use utility_helpers::{
    log_error, log_info,log_warn,
    message_pack_helper::serialize_to_message_pack,
    nats_helper::{NatsSubjects, types::MarketOrderCreateMessage},
};
use crate::{
    order_book::market_book::LmsrExecutionResult,
    order_book::outcome_book::OrderBookMatchedOutput, state::AppState, utils::OrderServiceError,
};

use sqlx::types::Uuid;
use rust_decimal_macros::dec;


pub async fn update_lmsr_orders(
    lmsr_results: Vec<LmsrExecutionResult>,
    app_state: Arc<AppState>,
    order: &Order,
) -> Result<(), OrderServiceError> {
    println!("🔵 [DEBUG] update_lmsr_orders called - Results count: {}, Order ID: {}", lmsr_results.len(), order.id);
    
    for (idx, result) in lmsr_results.iter().enumerate() {
        println!("🔵 [DEBUG] Processing result {} - Order ID: {}, Shares: {}, Total Cost: {}", 
            idx, result.order_id, result.shares_bought, result.total_cost);
        
        // 1. Persist the filled order state
        println!("🔵 [DEBUG] Step 1: Updating order status to FILLED for order {}", result.order_id);
        Order::update_order_status_and_filled_quantity(
            &app_state.db_pool,
            result.order_id,
            OrderStatus::FILLED,
            result.shares_bought,
        )
        .await
        .map_err(|e| format!("Failed to update order {}: {:#?}", result.order_id, e))?;
        println!("✅ [DEBUG] Order {} status updated to FILLED", result.order_id);

        println!("🔵 [DEBUG] Step 2: Starting transaction for order {}", result.order_id);
        let mut tx = app_state.db_pool.begin().await?;
        println!("✅ [DEBUG] Transaction started");

        // 2. Record the trade — LMSR has no opposite user
        println!("🔵 [DEBUG] Step 3: Creating user trade for order {}", result.order_id);
        UserTrades::create_user_trade(
            &mut *tx,
            result.order_id,
            result.order_id,
            order.user_id,
            order.market_id,
            order.outcome,
            result.average_price,
            result.shares_bought,
            order.side,
        )
        .await
        .map_err(|e| format!("Failed to create user trade: {:#?}", e))?;
        println!("✅ [DEBUG] User trade created");

        // 3. Update holdings
        let holding_delta = match order.side {
            OrderSide::BUY => result.shares_bought,
            OrderSide::SELL => -result.shares_bought,
        };
        println!("🔵 [DEBUG] Step 4: Updating holdings - Delta: {}", holding_delta);

        UserHoldings::update_user_holdings(
            &mut *tx,
            order.user_id,
            order.market_id,
            holding_delta,
            order.outcome,
        )  
        .await
        .map_err(|e| format!("Failed to update holdings: {:#?}", e))?;
        println!("✅ [DEBUG] Holdings updated");

        // 4. Debit/credit the trader's balance
        match order.side {
            OrderSide::BUY => {
                println!("🔵 [DEBUG] Step 5a: Processing BUY order - Updating user balance");
                User::update_user_balance(
                    &app_state.db_pool,
                    order.user_id,
                    result.total_cost ,
                    order.side,
                )
                .await
                .map_err(|e| format!("Failed to update user balance for BUY: {:#?}", e))?;
                println!("✅ [DEBUG] BUY balance updated");
            },
OrderSide::SELL => {
    // 1. Fetch order history
    let buys = Order::get_user_market_buy_orders(
        &app_state.db_pool,
        order.user_id,
        order.market_id
    ).await?;

    let sells = Order::get_user_market_sell_orders_except(
        &app_state.db_pool,
        order.user_id,
        order.market_id,
        order.id
    ).await?;

    // 2. FIFO lots
    let mut lots = build_fifo_buy_lots(buys);

    // 3. Apply past sells
    for s in sells {
        let past_sell_shares = s.quantity / s.price;
        apply_sell_to_lots(&mut lots, s.price, past_sell_shares);
    }

    // 4. Shares for this sell
    let current_sell_shares = result.shares_bought;  // ← use REAL executed shares

    // 5. Realized PnL
    let realized_profit = calculate_realized_profit(
        &mut lots,
        result.average_price,
        current_sell_shares
    );

    // ================================
    // 6. Apply FEES ONLY ON PROFIT
    // ================================

    let mut payout_after_fees = result.total_cost; // base earnings from LMSR

    if realized_profit > dec!(0) {
        let trading_fee = realized_profit * dec!(0.01);
        let platform_fee = (realized_profit - trading_fee) * dec!(0.05);
        let profit_after_fees = realized_profit - trading_fee - platform_fee;

        payout_after_fees = 
            (result.total_cost - realized_profit) + profit_after_fees;

        println!("💰 PROFIT detected => Applying fees");
        println!("   Raw Profit: {}", realized_profit);
        println!("   Trading fee (1%): {}", trading_fee);
        println!("   Platform fee (5%): {}", platform_fee);
        println!("   Net Profit: {}", profit_after_fees);
        println!("   Final User Payout: {}", payout_after_fees);
        
        // ================================
        // 7. DISTRIBUTE FEES TO LPs (Only when profit)
        // ================================
        println!("🔵 [DEBUG] Step 9: Fee distribution for SELL order");
        log_info!("💰 Processing fee distribution for SELL order {}", order.id);
        
        // Use the already calculated trading_fee (1% of profit)
    let lp_fee_pool = trading_fee * dec!(0.65);   // 65% of trading fee to LPs
    let platform_fee_total = trading_fee * dec!(0.35);  // 35% of trading fee to platform
        
     println!("🔵 [DEBUG] Fee breakdown - Trading Fee (1% of profit): {}, LP Pool (65%): {}, Platform (35%): {}", 
        trading_fee, lp_fee_pool, platform_fee_total);
    log_info!(
        "💰 Fee breakdown - Trading fee: {}, LP pool: {}, Platform: {}",
        trading_fee, lp_fee_pool, platform_fee_total
    );
        // ===============================================
// REFERRAL REWARD (10% of platform fee)
// ===============================================
println!("🔵 [DEBUG] Checking referral status for user {}", order.user_id);

// Fetch the user who executed this trade
let user = User::get_user_by_id(&app_state.db_pool, order.user_id)
    .await
    .map_err(|e| format!("Failed to fetch user: {:#?}", e))?;

if let Some(referrer_id) = user.referred_by {
    println!("🟦 User {} was referred by {}", user.id, referrer_id);

    // 10% of platform fee goes to the referrer
    let referral_reward = platform_fee_total * dec!(0.10);

    println!(
        "🏆 Referral Reward: {} receives 10% of platform fee = {}",
        referrer_id, referral_reward
    );

    // Update the referrer's balance
    User::update_user_balance(
        &app_state.db_pool,
        referrer_id,
        referral_reward,
        OrderSide::BUY // or a neutral enum, depending on your system
    )
    .await
    .map_err(|e| format!("Failed to update referrer balance: {:#?}", e))?;

    println!("✅ Referral reward added to user {}", referrer_id);
} else {
    println!("⚪ No referrer for user {}", order.user_id);
}
        // Get LPs and distribute fees
        println!("🔵 [DEBUG] Fetching LPs for market {}", order.market_id);
        let lps: Vec<LpPosition> = LpPosition::get_lps_by_market(&app_state.db_pool, order.market_id).await
            .map_err(|e| format!("Failed to get LPs: {:#?}", e))?;
        println!("✅ [DEBUG] Found {} LPs", lps.len());
        
        let total_liquidity = result.liquidity_b;
        println!("🔵 [DEBUG] Total liquidity: {}", total_liquidity);
        
        if total_liquidity > Decimal::ZERO && !lps.is_empty() {
            for (lp_idx, lp) in lps.iter().enumerate() {
                // Calculate total value = amount_deposited + fees already earned
                let lp_total_value = lp.amount_deposited;
                let lp_share = lp_total_value / total_liquidity;
                let lp_fee = lp_fee_pool * lp_share;
                
                println!("🔵 [DEBUG] LP {} - User: {}, Amount: {}, Share: {:.4}%, Fee: {}", 
                    lp_idx, lp.user_id, lp.amount_deposited, lp_share * dec!(100), lp_fee);
                log_info!(
                    "💰 LP {} - Share: {:.4}%, Fee earned: {}",
                    lp.user_id, lp_share * dec!(100), lp_fee
                );
                
                // Update both fees_earned and shares_of_pool
                LpPosition::update_fees_earned_and_shares(
                    &app_state.db_pool, 
                    lp.lp_position_id, 
                    lp_fee,
                    lp_share  // This is the percentage (0.0 to 1.0)
                ).await
                    .map_err(|e| format!("Failed to update LP fees: {:#?}", e))?;
                
                // // Optionally update LP's user balance
                // User::update_user_balance(
                //     &app_state.db_pool, 
                //     lp.user_id, 
                //     lp_fee, 
                //     OrderSide::BUY
                // ).await
                //     .map_err(|e| format!("Failed to update LP balance: {:#?}", e))?;
                
                println!("✅ [DEBUG] LP {} fees updated", lp_idx);
            }
        } else {
            println!("⚠️ [DEBUG] No LPs found or total_liquidity is zero for market {}", order.market_id);
            log_warn!("💰 No LPs found or total_liquidity is zero for market {}", order.market_id);
        }
        
        // Optional: Distribute platform fee to platform wallet
        // platform_fee_total goes to the platform's revenue account
        
    } else {
        println!("🔴 No profit → No fees applied");
        println!("   Loss or break-even: {}", realized_profit);
    }

    // 8. Update user balance (already includes fee deduction)
    User::update_user_balance(
        &app_state.db_pool,
        order.user_id,
        payout_after_fees,
        order.side
    )
    .await
    .map_err(|e| format!("Failed to update user balance for SELL: {:#?}", e))?;

    println!("✅ SELL balance updated");
}
        }

        // 5. Persist LMSR state
        println!("🔵 [DEBUG] Step 6: Persisting LMSR state - q_yes: {}, q_no: {}, liquidity_b: {}", 
            result.q_yes, result.q_no, result.liquidity_b);
        Market::update_lmsr_state(
            &app_state.db_pool,
            order.market_id,
            result.q_yes,
            result.q_no,
            result.liquidity_b,
        )
        .await
        .map_err(|e| format!("Failed to persist LMSR state: {:#?}", e))?;
        println!("✅ [DEBUG] LMSR state persisted");


MarketStateHistory::create_market_state_snapshot(
    &app_state.db_pool,
    order.market_id,
    result.q_yes,
    result.q_no,
    result.liquidity_b,
    result.new_yes_price,
    result.new_no_price,
)
.await
.map_err(|e| format!("Failed to insert market state snapshot: {:#?}", e))?;


        println!("🔵 [DEBUG] Step 7: Committing transaction");
        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {:#?}", e))?;
        println!("✅ [DEBUG] Transaction committed");

        // 6. Check pending limit orders
        println!("🔵 [DEBUG] Step 8: Checking pending limit orders");
        if let Err(e) = trigger_eligible_limit_orders(
            &app_state,
            order.market_id,
            result.new_yes_price,
            result.new_no_price,
        )
        .await
        {
            println!("⚠️ [DEBUG] Failed to trigger limit orders: {:#?}", e);
            log_error!("Failed to trigger limit orders after price update: {:#?}", e);
        }
        println!("✅ [DEBUG] Limit orders check completed");

//         // 7. Fee distribution to LPs - ONLY for SELL orders
//         if order.side == OrderSide::SELL {
//             println!("🔵 [DEBUG] Step 9: Fee distribution for SELL order");
//             log_info!("💰 Processing fee distribution for SELL order {}", order.id);
            
//             let total_fee = result.total_cost * dec!(0.01);
//             let lp_fee_pool = total_fee * dec!(0.65);
//             let platform_fee = total_fee * dec!(0.35);
            
//             println!("🔵 [DEBUG] Fee breakdown - Total: {}, LP Pool: {}, Platform: {}", 
//                 total_fee, lp_fee_pool, platform_fee);
//             log_info!(
//                 "💰 Fee breakdown - Total fee: {}, LP pool: {}, Platform: {}",
//                 total_fee, lp_fee_pool, platform_fee
//             );
            
//             // Get LPs and distribute fees
//             println!("🔵 [DEBUG] Fetching LPs for market {}", order.market_id);
//             let lps: Vec<LpPosition> = LpPosition::get_lps_by_market(&app_state.db_pool, order.market_id).await
//                 .map_err(|e| format!("Failed to get LPs: {:#?}", e))?;
//             println!("✅ [DEBUG] Found {} LPs", lps.len());
            
//             let total_liquidity = result.liquidity_b;
//             println!("🔵 [DEBUG] Total liquidity: {}", total_liquidity);
            
//             if total_liquidity > Decimal::ZERO && !lps.is_empty() {
//            for (lp_idx, lp) in lps.iter().enumerate() {

//   // Calculate total value = amount_deposited + fees already earned
//     let lp_total_value = lp.amount_deposited + lp.total_fees_earned;

//  let lp_share = lp_total_value / total_liquidity;
//     let lp_fee = lp_fee_pool * lp_share;
    
//     println!("🔵 [DEBUG] LP {} - User: {}, Amount: {}, Share: {:.4}%, Fee: {}", 
//         lp_idx, lp.user_id, lp.amount_deposited, lp_share * dec!(100), lp_fee);
//     log_info!(
//         "💰 LP {} - Share: {:.4}%, Fee earned: {}",
//         lp.user_id, lp_share * dec!(100), lp_fee
//     );
    
//     // Update both fees_earned and shares_of_pool
//     LpPosition::update_fees_earned_and_shares(
//         &app_state.db_pool, 
//         lp.lp_position_id, 
//         lp_fee,
//         lp_share  // This is the percentage (0.0 to 1.0)
//     ).await
//         .map_err(|e| format!("Failed to update LP fees: {:#?}", e))?;
    
// //     User::update_user_balance(&app_state.db_pool, lp.user_id, lp_fee, OrderSide::BUY).await
// //         .map_err(|e| format!("Failed to update LP balance: {:#?}", e))?;
// //     println!("✅ [DEBUG] LP {} fees updated", lp_idx);
// }
//             } else {
//                 println!("⚠️ [DEBUG] No LPs found or total_liquidity is zero for market {}", order.market_id);
//                 log_warn!("💰 No LPs found or total_liquidity is zero for market {}", order.market_id);
//             }
//         }
        
        println!("✅ [DEBUG] Result {} processing completed", idx);
    }

    println!("✅ [DEBUG] update_lmsr_orders completed successfully");
    Ok(())
}




/// Fetch all OPEN limit orders AND stop-loss orders for this market whose price condition is now met,
/// and publish them to the matching engine via NATS.
async fn trigger_eligible_limit_orders(
    app_state: &Arc<AppState>,
    market_id: Uuid,
    new_yes_price: Decimal,
    new_no_price: Decimal,
) -> Result<(), OrderServiceError> {
    log_info!("🔍 [TRIGGER] Checking for eligible limit/stop orders in market: {}", market_id);
    
    // Fetch all open limit orders for this market
    let pending_limit_orders = Order::get_open_limit_orders_for_market(
        &app_state.db_pool,
        market_id,
    )
    .await
    .map_err(|e| format!("❌ Failed to fetch pending limit orders: {:#?}", e))?;

    // Fetch all open stop-loss orders for this market
    let pending_stop_orders = Order::get_open_stop_orders_for_market(
        &app_state.db_pool,
        market_id,
    )
    .await
    .map_err(|e| format!("❌ Failed to fetch pending stop orders: {:#?}", e))?;

    log_info!("📋 [TRIGGER] Found {} limit orders and {} stop orders to evaluate", 
        pending_limit_orders.len(), pending_stop_orders.len());

    let mut triggered_count = 0;

    // Process LIMIT orders
    for pending in pending_limit_orders {
        let current_price = match pending.outcome {
            Outcome::YES => new_yes_price,
            Outcome::NO  => new_no_price,
            _            => continue,
        };

        // LIMIT order conditions
        let should_trigger = match pending.side {
            OrderSide::BUY  => current_price <= pending.price,  // Buy when price drops to limit
            OrderSide::SELL => current_price >= pending.price,  // Sell when price rises to limit
        };

        if !should_trigger {
            log_info!("⏸️  [LIMIT] Order {} — {:?} {:?} | limit: {}, current: {} (NOT met)", 
                pending.id, pending.side, pending.outcome, pending.price, current_price);
            continue;
        }

        log_info!(
            "✅ [LIMIT] Order {} — {:?} {:?} | limit: {}, current: {} (MET!)",
            pending.id, pending.side, pending.outcome, pending.price, current_price
        );

        // Mark as pending execution
        match Order::update_order_status(
            pending.id,
            OrderStatus::PendingUpdate,
            &app_state.db_pool,
        )
        .await
        {
            Ok(_) => log_info!("✏️  [LIMIT] Order {} marked as PendingUpdate", pending.id),
            Err(e) => {
                log_error!("❌ [LIMIT] Failed to mark order {} as pending: {:#?}", pending.id, e);
                continue;
            }
        };

        // Publish to NATS
        let message = MarketOrderCreateMessage {
            order_id: pending.id,
            budget: pending.quantity,
        };

        match serialize_to_message_pack(&message) {
            Ok(encoded) => {
                match app_state
                    .jetstream
                    .publish(NatsSubjects::MarketOrderCreate.to_string(), encoded.into())
                    .await
                {
                    Ok(_) => {
                        log_info!("📤 [LIMIT] Order {} published to NATS", pending.id);
                        triggered_count += 1;
                    }
                    Err(e) => {
                        log_error!("❌ [LIMIT] Failed to publish order {}: {:#?}", pending.id, e);
                        let _ = Order::update_order_status(pending.id, OrderStatus::OPEN, &app_state.db_pool).await;
                        log_info!("🔄 [LIMIT] Order {} reverted to OPEN", pending.id);
                    }
                }
            }
            Err(e) => {
                log_error!("❌ [LIMIT] Failed to serialize order {}: {:#?}", pending.id, e);
                let _ = Order::update_order_status(pending.id, OrderStatus::OPEN, &app_state.db_pool).await;
            }
        }
    }

    // Process STOP-LOSS orders
    for pending in pending_stop_orders {
        let current_price = match pending.outcome {
            Outcome::YES => new_yes_price,
            Outcome::NO  => new_no_price,
            _            => continue,
        };

        // STOP-LOSS conditions (opposite of limit orders)
        // BUY stop: trigger when price RISES to or above stop price (momentum/breakout)
        // SELL stop: trigger when price FALLS to or below stop price (stop loss)
        let should_trigger = match pending.side {
            OrderSide::BUY  => current_price >= pending.price,  // Buy on breakout
            OrderSide::SELL => current_price <= pending.price,  // Sell on stop loss
        };

        if !should_trigger {
            log_info!("⏸️  [STOP] Order {} — {:?} {:?} | stop: {}, current: {} (NOT met)", 
                pending.id, pending.side, pending.outcome, pending.price, current_price);
            continue;
        }

        log_info!(
            "🛑 [STOP] Order {} — {:?} {:?} | stop: {}, current: {} (MET!)",
            pending.id, pending.side, pending.outcome, pending.price, current_price
        );

        // Mark as pending execution
        match Order::update_order_status(
            pending.id,
            OrderStatus::PendingUpdate,
            &app_state.db_pool,
        )
        .await
        {
            Ok(_) => log_info!("✏️  [STOP] Order {} marked as PendingUpdate", pending.id),
            Err(e) => {
                log_error!("❌ [STOP] Failed to mark order {} as pending: {:#?}", pending.id, e);
                continue;
            }
        };

        // Publish to NATS
        let message = MarketOrderCreateMessage {
            order_id: pending.id,
            budget: pending.quantity,
        };

        match serialize_to_message_pack(&message) {
            Ok(encoded) => {
                match app_state
                    .jetstream
                    .publish(NatsSubjects::MarketOrderCreate.to_string(), encoded.into())
                    .await
                {
                    Ok(_) => {
                        log_info!("📤 [STOP] Order {} published to NATS", pending.id);
                        triggered_count += 1;
                    }
                    Err(e) => {
                        log_error!("❌ [STOP] Failed to publish order {}: {:#?}", pending.id, e);
                        let _ = Order::update_order_status(pending.id, OrderStatus::OPEN, &app_state.db_pool).await;
                        log_info!("🔄 [STOP] Order {} reverted to OPEN", pending.id);
                    }
                }
            }
            Err(e) => {
                log_error!("❌ [STOP] Failed to serialize order {}: {:#?}", pending.id, e);
                let _ = Order::update_order_status(pending.id, OrderStatus::OPEN, &app_state.db_pool).await;
            }
        }
    }

    if triggered_count > 0 {
        log_info!("🎯 [TRIGGER] Successfully triggered {} order(s) for market {}", triggered_count, market_id);
    } else {
        log_info!("💤 [TRIGGER] No orders were triggered for market {}", market_id);
    }

    Ok(())
}


#[derive(Debug, Clone)]
struct BuyLot {
    price: Decimal,
    shares: Decimal,
}

fn build_fifo_buy_lots(buy_orders: Vec<Order>) -> VecDeque<BuyLot> {
    let mut lots = VecDeque::new();

    for o in buy_orders {
        // shares = budget / price
        let shares = if o.price > dec!(0) {
            o.quantity / o.price
        } else {
            dec!(0)
        };

        lots.push_back(BuyLot {
            price: o.price,
            shares,
        });
    }

    lots
}

fn apply_sell_to_lots(
    lots: &mut VecDeque<BuyLot>,
    sell_price: Decimal,
    mut sell_shares: Decimal,
) {
    while sell_shares > dec!(0) && !lots.is_empty() {
        let mut lot = lots.pop_front().unwrap();

        let used = sell_shares.min(lot.shares);

        lot.shares -= used;
        sell_shares -= used;

        if lot.shares > dec!(0) {
            lots.push_front(lot);
        }
    }
}




fn calculate_realized_profit(
    lots: &mut VecDeque<BuyLot>,
    sell_price: Decimal,
    mut sell_shares: Decimal,
) -> Decimal {
    let mut total_profit = dec!(0);
    let mut lot_index = 0;

    println!("================ FIFO PROFIT CALC START ================");
    println!("📉 Sell Price: {}", sell_price);
    println!("📦 Total Sell Shares: {}", sell_shares);
    println!("--------------------------------------------------------");

    while sell_shares > dec!(0) && !lots.is_empty() {
        let mut lot = lots.pop_front().unwrap();

        lot_index += 1;

        println!("🧾 Processing Lot #{}", lot_index);
        println!("   Lot Price: {}", lot.price);
        println!("   Available Shares in Lot: {}", lot.shares);
        println!("   Remaining Sell Shares: {}", sell_shares);

        // number of shares removed from this lot
        let used = sell_shares.min(lot.shares);

        println!("   👉 Shares Used from Lot: {}", used);

        let cost_basis = lot.price * used;
        let proceeds = sell_price * used;

        let profit = proceeds - cost_basis;
        total_profit += profit;

        println!("   💰 Cost Basis: {}", cost_basis);
        println!("   💵 Proceeds: {}", proceeds);
        println!("   📊 Profit from this lot: {}", profit);
        println!("   📈 Running Total Profit: {}", total_profit);

        lot.shares -= used;
        sell_shares -= used;

        if lot.shares > dec!(0) {
            println!("   ↩ Remaining shares in lot: {}", lot.shares);
            lots.push_front(lot);
        } else {
            println!("   ❌ Lot fully consumed");
        }

        println!("--------------------------------------------------------");
    }

    println!("================ FIFO PROFIT CALC END ================");
    println!("🏁 FINAL REALIZED PROFIT: {}", total_profit);

    total_profit
}








// pub async fn update_lmsr_orders(
//     lmsr_results: Vec<LmsrExecutionResult>,
//     app_state: Arc<AppState>,
//     order: &Order,
// ) -> Result<(), OrderServiceError> {
//     for result in lmsr_results {
//         // 1. Persist the filled order state
//         Order::update_order_status_and_filled_quantity(
//             &app_state.db_pool,
//             result.order_id,
//             OrderStatus::FILLED,
//             result.shares_bought,
//         )
//         .await
//         .map_err(|e| format!("Failed to update order {}: {:#?}", result.order_id, e))?;

//         let mut tx = app_state.db_pool.begin().await?;

//         // 2. Record the trade — LMSR has no opposite user
//         UserTrades::create_user_trade(
//             &mut *tx,
//             result.order_id,
//             result.order_id,
//             order.user_id,
//             order.market_id,
//             order.outcome,
//             result.average_price,
//             result.shares_bought,
//             order.side,
//         )
//         .await
//         .map_err(|e| format!("Failed to create user trade: {:#?}", e))?;

//         // 3. Update holdings
//         let holding_delta = match order.side {
//             OrderSide::BUY => result.shares_bought,
//             OrderSide::SELL => -result.shares_bought,
//         };

//         UserHoldings::update_user_holdings(
//             &mut *tx,
//             order.user_id,
//             order.market_id,
//             holding_delta,
//             order.outcome,
//         )
//         .await
//         .map_err(|e| format!("Failed to update holdings: {:#?}", e))?;

//         // 4. Debit/credit the trader's balance
//         match order.side {
//             OrderSide::BUY => {
//                 User::update_user_balance(
//                     &app_state.db_pool,
//                     order.user_id,
//                     result.total_cost * Decimal::from(100),
//                     order.side,
//                 )
//                 .await
//                 .map_err(|e| format!("Failed to update user balance for BUY: {:#?}", e))?;
//             },
//             OrderSide::SELL => {
//                 let total_fee = result.total_cost * dec!(0.01);
//                 let after_trading_fee = result.total_cost - total_fee;
//                 let platform_fee = after_trading_fee * dec!(0.05);
//                 let payout_after_fees = (after_trading_fee - platform_fee) * Decimal::from(100);
                
//                 log_info!(
//                     "💰 SELL order {} - Total cost: {}, Trading fee (1%): {}, After trading fee: {}, Platform fee (5%): {}, Payout after all fees: {}",
//                     order.id, 
//                     result.total_cost, 
//                     total_fee, 
//                     after_trading_fee,
//                     platform_fee,
//                     payout_after_fees
//                 );
                
//                 User::update_user_balance(
//                     &app_state.db_pool,
//                     order.user_id,
//                     payout_after_fees,
//                     order.side,
//                 )
//                 .await
//                 .map_err(|e| format!("Failed to update user balance for SELL: {:#?}", e))?;
//             }
//         }

//         // 5. Persist LMSR state
//         Market::update_lmsr_state(
//             &app_state.db_pool,
//             order.market_id,
//             result.q_yes,
//             result.q_no,
//             result.liquidity_b,
//         )
//         .await
//         .map_err(|e| format!("Failed to persist LMSR state: {:#?}", e))?;

//         tx.commit()
//             .await
//             .map_err(|e| format!("Failed to commit transaction: {:#?}", e))?;

//         // 6. Check pending limit orders
//         if let Err(e) = trigger_eligible_limit_orders(
//             &app_state,
//             order.market_id,
//             result.new_yes_price,
//             result.new_no_price,
//         )
//         .await
//         {
//             log_error!("Failed to trigger limit orders after price update: {:#?}", e);
//         }

//         // 7. Fee distribution to LPs - ONLY for SELL orders
//         if order.side == OrderSide::SELL {
//             log_info!("💰 Processing fee distribution for SELL order {}", order.id);
            
//             let total_fee = result.total_cost * dec!(0.01);
//             let lp_fee_pool = total_fee * dec!(0.65);
//             let platform_fee = total_fee * dec!(0.35);
            
//             log_info!(
//                 "💰 Fee breakdown - Total fee: {}, LP pool: {}, Platform: {}",
//                 total_fee, lp_fee_pool, platform_fee
//             );
            
//             // Get LPs and distribute fees
//            // To this with explicit type:
//     let lps: Vec<LpPosition> = LpPosition::get_lps_by_market(&app_state.db_pool, order.market_id).await
//         .map_err(|e| format!("Failed to get LPs: {:#?}", e))?;
            
//             let total_liquidity = result.liquidity_b;
            
//             if total_liquidity > Decimal::ZERO && !lps.is_empty() {
//                 for lp in lps {
//                     let lp_share = lp.amount_deposited / total_liquidity;
//                     let lp_fee = lp_fee_pool * lp_share;
                    
//                     log_info!(
//                         "💰 LP {} - Share: {:.4}, Fee earned: {}",
//                         lp.user_id, lp_share, lp_fee
//                     );
                    
//                     LpPosition::update_fees_earned(&app_state.db_pool, lp.lp_position_id, lp_fee).await
//                         .map_err(|e| format!("Failed to update LP fees: {:#?}", e))?;
                    
//                     User::update_user_balance(&app_state.db_pool, lp.user_id, lp_fee * Decimal::from(100), OrderSide::BUY).await
//                         .map_err(|e| format!("Failed to update LP balance: {:#?}", e))?;
//                 }
//             } else {
//                 log_warn!("💰 No LPs found or total_liquidity is zero for market {}", order.market_id);
//             }
//         }
//     }

//     Ok(())
// }



