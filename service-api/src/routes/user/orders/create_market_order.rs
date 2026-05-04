use async_nats::jetstream;
use auth_service::types::SessionTokenClaims;
use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::{
    enums::{MarketStatus, OrderSide, OrderStatus, OrderType, Outcome},
    market::Market,
    orders::Order,
    user_holdings::UserHoldings,
    users::User,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utility_helpers::{
    log_error, log_info,log_warn,
    message_pack_helper::serialize_to_message_pack,
    nats_helper::{NatsSubjects, types::MarketOrderCreateMessage},
};
use uuid::Uuid;
use rust_decimal_macros::dec;
use rust_decimal::MathematicalOps;

use crate::{require_field, state::AppState};

#[derive(Debug, Deserialize)]
pub struct TradeRequest {
    pub market_id: Uuid,           // Market ID in the request
    pub amount: Decimal,           // Amount in KES the user wants to stake
    pub outcome: bool,             // true = YES, false = NO
}

#[derive(Debug, Serialize)]
pub struct TradeResponse {
    pub market_id: Uuid,
    pub amount_staked: Decimal,
    pub outcome: String,
    pub shares_acquired: Decimal,
    pub potential_profit: Decimal,
    pub max_allowed_profit: Decimal,
    pub is_valid: bool,
    pub message: String,
}



#[derive(Debug, Deserialize)]
pub struct MarketOrderPayload {
    pub market_id: Option<Uuid>,
    pub outcome: Option<Outcome>,
    pub side: Option<OrderSide>,
    pub amount_spent: Option<Decimal>,
    pub price_at_execution: Option<Decimal>,
    pub shares_to_sell: Option<Decimal>,   // ← ADD
}



pub async fn create_limit_order(
    State(app_state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Json(payload): Json<MarketOrderPayload>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let market_id         = payload.market_id;
    let amount_spent      = payload.amount_spent;
    let outcome           = payload.outcome;
    let side              = payload.side;
    let price_at_execution = payload.price_at_execution;

    require_field!(market_id);
    require_field!(amount_spent);
    require_field!(outcome);
    require_field!(side);
    require_field!(price_at_execution);

    let market_id = market_id.unwrap();
    let amount_spent       = amount_spent.unwrap();
    let outcome            = outcome.unwrap();
    let side               = side.unwrap();
    let price_at_execution = price_at_execution.unwrap();
    let user_id            = claims.user_id;


    let market = Market::get_market_by_id(&app_state.pg_pool, &market_id)
        .await
        .map_err(|e| {
            log_error!("Failed to get market - {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(json!({"error": "Failed to get market"})).into_response())
        })?
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Market not found"})).into_response(),
        ))?;


    // Only validate for BUY orders (market maker protection)
    if side == OrderSide::BUY {
        let b = market.liquidity_b;
        let q_yes = market.q_yes;
        let q_no = market.q_no;

        // Calculate current price
        let exp_yes = (q_yes / b).exp();
        let exp_no = (q_no / b).exp();
        let total = exp_yes + exp_no;

        let current_price = if outcome == Outcome::YES {
            exp_yes / total
        } else {
            exp_no / total
        };

        // Calculate shares and potential profit
        let shares_acquired = if current_price.is_zero() {
            Decimal::ZERO
        } else {
            amount_spent / current_price
        };

        let potential_profit = shares_acquired - amount_spent;
        let max_loss = b * dec!(2.0).ln();
        let max_allowed_profit = max_loss * dec!(0.30);

        if potential_profit > max_allowed_profit {
            log_info!(
                "❌ Trade validation failed - potential profit {:.2} exceeds 30% of max loss ({:.2})",
                potential_profit, max_allowed_profit
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Trade exceeds allowed limits",
                    "message": format!(
                        "Potential profit {:.2} KES exceeds 30% of max loss ({:.2} KES). Please reduce your stake amount.",
                        potential_profit, max_allowed_profit
                    ),
                    "potential_profit": potential_profit,
                    "max_allowed_profit": max_allowed_profit,
                })).into_response(),
            ));
        }

        log_info!(
            "✅ Trade validation passed - potential profit {:.2} within limit ({:.2})",
            potential_profit, max_allowed_profit
        );
    }


    if market.status == MarketStatus::SETTLED {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Market is already settled, cannot create order"})).into_response(),
        ));
    }
 

    if price_at_execution <= Decimal::ZERO || price_at_execution >= Decimal::ONE {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Price at execution must be between 0 and 1"})).into_response(),
        ));
    }

    app_state
        .jetstream
    .get_or_create_stream(jetstream::stream::Config {
        name: "ORDER".into(),
        subjects: vec!["order.>".into()],

        // 🔥 REQUIRED for NGS / cloud JetStream
        max_bytes: 1024 * 1024 * 1024, // 1GB

        ..Default::default()
    })
        .await
        .map_err(|e| {
            log_error!("Failed to get or create stream: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(json!({"error": "Failed to initialize message stream"})).into_response())
        })?;

    // ── Balance / holdings gate ──────────────────────────────────────────────
    if side == OrderSide::SELL {
        let holdings = UserHoldings::get_user_holdings_by_outcome(
            &app_state.pg_pool, user_id, market_id, outcome,
        )
        .await
        .map_err(|e| {
            log_error!("Failed to get user holdings: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(json!({"error": "Failed to retrieve user holdings"})).into_response())
        })?;

        if holdings.shares <= Decimal::ZERO {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Insufficient shares to place a sell order"})).into_response(),
            ));
        }

        if let Some(shares_to_sell) = payload.shares_to_sell {
            if shares_to_sell <= Decimal::ZERO {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "shares_to_sell must be greater than zero"})).into_response(),
                ));
            }
            if holdings.shares < shares_to_sell {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Insufficient shares",
                        "available": holdings.shares,
                        "requested": shares_to_sell,
                    })).into_response(),
                ));
            }
        }
    } else {
        // BUY — check balance
        let balance = User::get_user_balance(&app_state.pg_pool, user_id)
            .await
            .map_err(|e| {
                log_error!("Failed to get user balance: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR,
                 Json(json!({"error": "Failed to retrieve user balance"})).into_response())
            })?;

        let locked = Order::get_user_order_locked_funds(&app_state.pg_pool, user_id)
            .await
            .map_err(|e| {
                log_error!("Failed to get locked funds: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR,
                 Json(json!({"error": "Failed to get user orders total amount"})).into_response())
            })?;

        log_info!(
            "Balance check — user: {}, balance: {}, locked: {}, spending: {}",
            user_id, balance, locked, amount_spent
        );

        if locked + amount_spent > balance {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Insufficient balance to place a buy order"})).into_response(),
            ));
        }
    }
    // ── End balance / holdings gate ──────────────────────────────────────────

    // Create order — this runs for BOTH buy and sell
  

    let order = Order::create_order(
        user_id,
        market_id,
        price_at_execution,
         amount_spent,  // Always amount_spent (KES) for both BUY and SELL
        side,
        outcome,
        OrderType::MARKET,
        &app_state.pg_pool,
    )
    .await
    .map_err(|e| {
        log_error!("Failed to create order: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR,
         Json(json!({"error": "Failed to create order"})).into_response())
    })?;

    let message = MarketOrderCreateMessage {
        order_id: order.id,
        budget:   amount_spent,
    };
// After creating the order, before publishing
log_info!("📤 [PUBLISH] About to publish order {} to JetStream", order.id);
log_info!("📤 [PUBLISH] Subject: {}", NatsSubjects::MarketOrderCreate.to_string());
log_info!("📤 [PUBLISH] Message content: order_id={}, budget={}", message.order_id, message.budget);

let encoded = serialize_to_message_pack(&message).map_err(|e| {
    log_error!("Failed to serialize message: {}", e);
    (StatusCode::INTERNAL_SERVER_ERROR,
     Json(json!({"error": "Failed to serialize market order create message"})).into_response())
})?;

log_info!("📤 [PUBLISH] Serialization successful, encoded size: {} bytes", encoded.len());

// Just log that we're publishing (no is_closed check)
log_info!("📤 [PUBLISH] Calling jetstream.publish()...");

match app_state
    .jetstream
    .publish(NatsSubjects::MarketOrderCreate.to_string(), encoded.into())
    .await
{
    Ok(response) => {
        log_info!("✅ [PUBLISH] Successfully published to NATS!");
        log_info!("✅ [PUBLISH] Response: {:?}", response);
    }
    Err(e) => {
        log_error!("❌ [PUBLISH] Failed to publish to NATS: {}", e);
        log_error!("❌ [PUBLISH] Error details: {:#?}", e);
        let _ = Order::update_order_status(order.id, OrderStatus::CANCELLED, &app_state.pg_pool).await;
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to publish market order create message"})).into_response(),
        ));
    }
}

log_info!(
    "✅ Order created and published — id: {}, amount_spent: {}, price_at_execution: {}",
    order.id, amount_spent, price_at_execution
);

    Ok((StatusCode::CREATED, Json(json!({
        "message": "Market order created successfully",
        "order": {
            "id": order.id,
            "user_id": order.user_id,
            "market_id": order.market_id,
            "amount_spent": amount_spent,
            "price_at_execution": price_at_execution,
            "shares_to_sell": payload.shares_to_sell,
            "outcome": outcome,
            "side": side,
            "status": order.status,
        }
    }))))
}







// Main handler - returns Response to avoid type mismatch
pub async fn fetch_and_validate_market(
    State(app_state): State<AppState>,
    Json(trade_request): Json<TradeRequest>,
) -> Response {
    match fetch_and_validate_market_inner(&app_state, trade_request).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => e,
    }
}

// Inner function with your error handling pattern
async fn fetch_and_validate_market_inner(
    app_state: &AppState,
    trade_request: TradeRequest,
) -> Result<TradeResponse, Response> {
    // Step 1: Get market by ID
    let market = Market::get_market_by_id(&app_state.pg_pool, &trade_request.market_id)
        .await
        .map_err(|e| {
            log_error!("Failed to get market - {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to get market"})),
            )
                .into_response()
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Market not found"})),
            )
                .into_response()
        })?;

    // Step 2: Get liquidity parameter b
    let b = market.liquidity_b;

    // Step 3: Get current q_yes and q_no
    let q_yes = market.q_yes;
    let q_no = market.q_no;

    // Step 4: Calculate current prices using LMSR formula
    let exp_yes = (q_yes / b).exp();
    let exp_no = (q_no / b).exp();
    let total = exp_yes + exp_no;

    let current_yes_price = exp_yes / total;
    let current_no_price = exp_no / total;

    // Step 5: Calculate shares acquired based on amount and current price
    let current_price = if trade_request.outcome {
        current_yes_price
    } else {
        current_no_price
    };

    let shares_acquired = if current_price.is_zero() {
        Decimal::ZERO
    } else {
        trade_request.amount / current_price
    };

    // Step 6: Calculate potential profit if user wins
    // Profit = (shares * 1 KES per share) - amount_staked
    let potential_profit = shares_acquired - trade_request.amount;

    // Step 7: Calculate max loss for this market
    // For binary LMSR: max_loss = b * ln(2)
    let max_loss = b * dec!(2.0).ln();

    // Step 8: Calculate 30% of max loss as the profit limit
    let max_allowed_profit = max_loss * dec!(0.30);

    // Step 9: Validate that potential profit doesn't exceed 30% of max loss
    let is_valid = potential_profit <= max_allowed_profit;

    let message = if is_valid {
        format!(
            "Trade valid: Potential profit {:.2} KES is within 30% limit of max loss ({:.2} KES)",
            potential_profit, max_allowed_profit
        )
    } else {
        format!(
            "Trade rejected: Potential profit {:.2} KES exceeds 30% of max loss ({:.2} KES). Please reduce your stake amount.",
            potential_profit, max_allowed_profit
        )
    };

    Ok(TradeResponse {
        market_id: trade_request.market_id,
        amount_staked: trade_request.amount,
        outcome: if trade_request.outcome {
            "YES".to_string()
        } else {
            "NO".to_string()
        },
        shares_acquired,
        potential_profit,
        max_allowed_profit,
        is_valid,
        message,
    })
}






// pub async fn create_limit_order(
//     State(app_state): State<AppState>,
//     Extension(claims): Extension<SessionTokenClaims>,
//     Json(payload): Json<MarketOrderPayload>,
// ) -> Result<impl IntoResponse, (StatusCode, Response)> {
//     let market_id = payload.market_id;
//     let amount_spent = payload.amount_spent;
//     let outcome = payload.outcome;
//     let side = payload.side;
//     let price_at_execution = payload.price_at_execution;

//     require_field!(market_id);
//     require_field!(amount_spent);
//     require_field!(outcome);
//     require_field!(side);
//     require_field!(price_at_execution);

//     let market_id = market_id.unwrap();
//     let amount_spent = amount_spent.unwrap();
//     let outcome = outcome.unwrap();
//     let side = side.unwrap();
//     let price_at_execution = price_at_execution.unwrap();
//     let user_id = claims.user_id;

//     // Validate price
//     if price_at_execution <= Decimal::ZERO || price_at_execution >= Decimal::ONE {
//         return Err((
//             StatusCode::BAD_REQUEST,
//             Json(json!({"error": "Price at execution must be between 0 and 1"})).into_response(),
//         ));
//     }

//     // Get market
//     let market = Market::get_market_by_id(&app_state.pg_pool, &market_id)
//         .await
//         .map_err(|e| {
//             log_error!("Failed to get market - {:?}", e);
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(json!({"error": "Failed to get market"})).into_response(),
//             )
//         })?;

//     let market = match market {
//         Some(m) => m,
//         None => {
//             return Err((
//                 StatusCode::NOT_FOUND,
//                 Json(json!({"error": "Market not found"})).into_response(),
//             ));
//         }
//     };

//     if market.status == MarketStatus::SETTLED {
//         return Err((
//             StatusCode::BAD_REQUEST,
//             Json(json!({"error": "Market is already settled, cannot create order"})).into_response(),
//         ));
//     }

//     // Get or create NATS stream
//     app_state
//         .jetstream
//         .get_or_create_stream(jetstream::stream::Config {
//             name: "ORDER".into(),
//             subjects: vec!["order.>".into()],
//             ..Default::default()
//         })
//         .await
//         .map_err(|e| {
//             log_error!("Failed to get or create stream: {}", e);
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(json!({"error": "Failed to initialize message stream"})).into_response(),
//             )
//         })?;

//     // Handle SELL order - check holdings
//     if side == OrderSide::SELL {
//         let holdings = UserHoldings::get_user_holdings_by_outcome(
//             &app_state.pg_pool,
//             user_id,
//             market_id,
//             outcome,
//         )
//         .await
//         .map_err(|e| {
//             log_error!("Failed to get user holdings: {}", e);
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(json!({"error": "Failed to retrieve user holdings"})).into_response(),
//             )
//         })?;

//         let shares_to_sell = amount_spent / price_at_execution;

//         if holdings.shares < shares_to_sell {
//             log_warn!(
//                 "❌ Insufficient shares - User: {}, Has: {}, Needs: {}",
//                 user_id, holdings.shares, shares_to_sell
//             );
//             return Err((
//                 StatusCode::BAD_REQUEST,
//                 Json(json!({
//                     "error": "Insufficient shares to place a sell order",
//                     "available": holdings.shares,
//                     "requested": shares_to_sell,
//                 })).into_response(),
//             ));
//         }

//         log_info!(
//             "✅ SELL order - User: {}, Shares to sell: {}, Price: {}",
//             user_id, shares_to_sell, price_at_execution
//         );
//     } 
//     // Handle BUY order - check balance
//     else {
//         let mut tx = app_state.pg_pool.begin().await.map_err(|e| {
//             log_error!("Failed to begin transaction - {:?}", e);
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(json!({"error": "Failed to begin transaction"})).into_response(),
//             )
//         })?;

//         let balance = User::get_user_balance(&mut *tx, user_id)
//             .await
//             .map_err(|e| {
//                 log_error!("Failed to get user balance: {}", e);
//                 (
//                     StatusCode::INTERNAL_SERVER_ERROR,
//                     Json(json!({"error": "Failed to retrieve user balance"})).into_response(),
//                 )
//             })?;

//         let total_user_locked_funds = Order::get_user_order_locked_funds(&mut *tx, user_id)
//             .await
//             .map_err(|e| {
//                 log_error!("Failed to get user orders total amount - {:?}", e);
//                 (
//                     StatusCode::INTERNAL_SERVER_ERROR,
//                     Json(json!({"error": "Failed to get user orders total amount"})).into_response(),
//                 )
//             })?;

//         // Commit transaction before balance check (no more DB operations needed)
//         tx.commit().await.map_err(|e| {
//             log_error!("Failed to commit transaction - {:?}", e);
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(json!({"error": "Failed to commit transaction"})).into_response(),
//             )
//         })?;

//         // Balance check
//         log_info!(
//             "💰 Balance check - User: {}, Balance: {}, Locked funds: {}, Amount to spend: {}",
//             user_id, balance, total_user_locked_funds, amount_spent
//         );

//         if total_user_locked_funds + amount_spent > balance {
//             log_warn!(
//                 "❌ Insufficient balance - User: {}, Balance: {}, Locked funds: {}, Total needed: {}",
//                 user_id, balance, total_user_locked_funds, total_user_locked_funds + amount_spent
//             );
//             return Err((
//                 StatusCode::BAD_REQUEST,
//                 Json(json!({"error": "Insufficient balance to place a buy order"})).into_response(),
//             ));
//         }

//         if balance < amount_spent {
//             log_warn!(
//                 "❌ Balance too low - User: {}, Balance: {}, Amount needed: {}",
//                 user_id, balance, amount_spent
//             );
//             return Err((
//                 StatusCode::BAD_REQUEST,
//                 Json(json!({"error": "Insufficient balance to place a buy order"})).into_response(),
//             ));
//         }

//         log_info!(
//             "✅ BUY order - User: {}, Amount to spend: {}, Price: {}",
//             user_id, amount_spent, price_at_execution
//         );
//     }

//     // Create order (LIMIT order, not MARKET)
//     let order = Order::create_order(
//         user_id,
//         market_id,
//         price_at_execution,
//         amount_spent,
//         side,
//         outcome,
//         OrderType::MARKET,
//         &app_state.pg_pool,
//     )
//     .await
//     .map_err(|e| {
//         log_error!("Failed to create order: {}", e);
//         (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(json!({"error": "Failed to create order"})).into_response(),
//         )
//     })?;

//     // Prepare message for matching engine
//     let message = MarketOrderCreateMessage {
//         order_id: order.id,
//         budget: amount_spent,
//     };

//     let encoded = serialize_to_message_pack(&message)
//         .map_err(|e| {
//             log_error!("Failed to serialize order create message: {}", e);
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(json!({"error": "Failed to serialize order message"})).into_response(),
//             )
//         })?;

//     let subject = NatsSubjects::MarketOrderCreate;

//     if let Err(e) = app_state
//         .jetstream
//         .publish(subject.to_string(), encoded.into())
//         .await
//     {
//         log_error!("Failed to publish order create message: {}", e);
        
//         // Update order status to cancelled
//         let _ = Order::update_order_status(order.id, OrderStatus::CANCELLED, &app_state.pg_pool).await;
        
//         return Err((
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(json!({"error": "Failed to publish order message"})).into_response(),
//         ));
//     }

//     log_info!(
//         "Limit order created and published - order_id: {}, amount_spent: {}, price: {}, side: {:?}, outcome: {:?}",
//         order.id, amount_spent, price_at_execution, side, outcome
//     );

//     let response = json!({
//         "message": "Limit order created successfully",
//         "order": {
//             "id": order.id,
//             "user_id": order.user_id,
//             "market_id": order.market_id,
//             "amount_spent": amount_spent,
//             "price_at_execution": price_at_execution,
//             "outcome": outcome,
//             "side": side,
//             "status": order.status,
//             "order_type": "LIMIT",
//         },
//     });

//     Ok((StatusCode::CREATED, Json(response)))
// }