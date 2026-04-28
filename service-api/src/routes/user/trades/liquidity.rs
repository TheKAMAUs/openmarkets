use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use rust_decimal::Decimal;
use db_service::schema::{
orders::Order,
    enums::OrderSide,
 enums::MarketStatus,
    market::Market,
    liquidity::LpPosition,
    users::User,
};
use auth_service::types::SessionTokenClaims;

use crate::{state::AppState, };

use utility_helpers::{
   log_warn, log_error, log_info,};

#[derive(Debug, Deserialize, Serialize)]
pub struct AddLiquidityRequest {
    pub market_id: Uuid,
    pub amount: Decimal,  // Amount in KES to add as liquidity
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RemoveLiquidityRequest {
    pub lp_position_id: Uuid,
    pub withdraw_amount: Decimal,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct GetFeeEarningsRequest {
    pub market_id: Option<Uuid>,
}




pub async fn add_liquidity_handler(
    State(app_state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Json(payload): Json<AddLiquidityRequest>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;
   // Start transaction with proper error handling
    let mut tx = app_state.pg_pool.begin()
        .await
        .map_err(|e| {
            log_error!("Failed to begin transaction: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": "Failed to begin transaction"})).into_response(),
            )
        })?;
    // Validate amount
    if payload.amount <= Decimal::ZERO {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"message": "Amount must be greater than 0"})).into_response(),
        ));
    }

    // Get market
    let market = Market::get_market_by_id(&app_state.pg_pool, &payload.market_id)
        .await
        .map_err(|e| {
            log_error!("Failed to get market: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": "Failed to get market"})).into_response(),
            )
        })?;

    let market = match market {
        Some(m) => m,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"message": "Market not found"})).into_response(),
            ));
        }
    };

    // Check if market is settled
    if market.status == MarketStatus::SETTLED {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"message": "Market is settled, cannot add liquidity"})).into_response(),
        ));
    }

    // ============================================
    // CHECK USER BALANCE
    // ============================================
   let balance = User::get_user_balance(&mut *tx, user_id)
    .await
    .map_err(|e| {
        log_error!("Failed to get user balance: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to get user balance"})).into_response(),
        )
    })? as Decimal;

    // Get user's locked funds from open orders
  let locked_funds = Order::get_user_order_locked_funds(&mut *tx, user_id)
    .await
    .map_err(|e| {
        log_error!("Failed to get user locked funds: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to get user locked funds"})).into_response(),
        )
    })? as Decimal;

    let total_required = locked_funds + payload.amount;

    log_info!(
        "💰 Liquidity deposit check - User: {}, Balance: {}, Locked funds: {}, Deposit amount: {}, Total required: {}",
        user_id, balance, locked_funds, payload.amount, total_required
    );

    if total_required > balance {
        log_warn!(
            "❌ Insufficient balance - User: {}, Balance: {}, Required: {}",
            user_id, balance, total_required
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"message": format!("Insufficient balance. Available: {}, Required: {}", balance, total_required)})).into_response(),
        ));
    }

    if balance < payload.amount {
        log_warn!(
            "❌ Balance too low - User: {}, Balance: {}, Deposit amount: {}",
            user_id, balance, payload.amount
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"message": "Insufficient balance to add liquidity"})).into_response(),
        ));
    }

    log_info!("✅ Balance check passed - User: {}", user_id);

    // Calculate shares
    let new_liquidity = market.liquidity_b + payload.amount;
    let shares_of_pool = payload.amount / new_liquidity;

    log_info!(
        "📊 Pool calculation - Current liquidity: {}, New liquidity: {}, User share: {:.4}%",
        market.liquidity_b, new_liquidity, (shares_of_pool * Decimal::from(100))
    );

    // Start transaction
    let mut tx = app_state.pg_pool.begin().await.map_err(|e| {
        log_error!("Failed to begin transaction: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to begin transaction"})).into_response(),
        )
    })?;

    // Add liquidity
    let lp_position = LpPosition::add_liquidity(
        &app_state.pg_pool,
        user_id,
        payload.market_id,
        payload.amount,
        shares_of_pool,
    )
    .await
    .map_err(|e| {
        log_error!("Failed to add liquidity: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": format!("Failed to add liquidity: {}", e)})).into_response(),
        )
    })?;

    // Update market liquidity_b
    Market::update_liquidity_b(
       &app_state.pg_pool,
        payload.market_id,
        payload.amount,
        true,
    )
    .await
    .map_err(|e| {
        log_error!("Failed to update market liquidity: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to update market liquidity"})).into_response(),
        )
    })?;

    // Deduct amount from user's balance
    User::update_user_balance(
         &app_state.pg_pool,
        user_id,
        payload.amount, // convert to minor units (cents)
        OrderSide::BUY,  // Using BUY to deduct balance
    )
    .await
    .map_err(|e| {
        log_error!("Failed to deduct user balance: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to deduct user balance"})).into_response(),
        )
    })?;

    // Commit transaction
    tx.commit().await.map_err(|e| {
        log_error!("Failed to commit transaction: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to commit transaction"})).into_response(),
        )
    })?;

    log_info!(
        "✅ Liquidity added successfully - User: {}, Market: {}, Amount: {}, Share: {:.4}%",
        user_id, payload.market_id, payload.amount, (shares_of_pool * Decimal::from(100))
    );

    Ok(Json(json!({
        "message": "Liquidity added successfully",
        "data": {
            "lp_position_id": lp_position.lp_position_id,
            "amount_deposited": lp_position.amount_deposited,
            "shares_of_pool": lp_position.shares_of_pool,
            "share_percentage": format!("{:.2}%", (lp_position.shares_of_pool * Decimal::from(100))),
            "market_liquidity_b": new_liquidity,
        }
    })))
}


pub async fn get_total_fee_earnings_handler(
    State(app_state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;

    let total_fees = LpPosition::get_fee_earnings(&app_state.pg_pool, user_id)
        .await
        .map_err(|e| {
            log_error!("Failed to get fee earnings: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": "Failed to get fee earnings"})).into_response(),
            )
        })?;

    Ok(Json(json!({
        "message": "Fee earnings fetched successfully",
        "data": {
            "total_fees_earned": total_fees
        }
    })))
}




pub async fn remove_liquidity_handler(
    State(app_state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Json(payload): Json<RemoveLiquidityRequest>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;

    // Validate amount
    if payload.withdraw_amount <= Decimal::ZERO {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"message": "Withdraw amount must be greater than 0"})).into_response(),
        ));
    }

    // Get LP position to verify ownership
    let lp_position = LpPosition::get_lp_position(&app_state.pg_pool, payload.lp_position_id)
        .await
        .map_err(|e| {
            log_error!("Failed to fetch LP position: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": "Failed to fetch LP position"})).into_response(),
            )
        })?;

    let lp_position = match lp_position {
        Some(pos) => pos,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"message": "LP position not found"})).into_response(),
            ));
        }
    };

    // Verify ownership
    if lp_position.user_id != user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"message": "You don't own this LP position"})).into_response(),
        ));
    }

    // Check if enough balance to withdraw
    if payload.withdraw_amount > lp_position.amount_deposited {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": format!("Cannot withdraw more than available. Available: {}", lp_position.amount_deposited)
            })).into_response(),
        ));
    }

    // Remove liquidity using existing method
    let updated_position = LpPosition::remove_liquidity(
        &app_state.pg_pool,
        payload.lp_position_id,
        payload.withdraw_amount,
    )
    .await
    .map_err(|e| {
        log_error!("Failed to remove liquidity: {e:?}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to remove liquidity"})).into_response(),
        )
    })?;

    // Update market liquidity_b (deduct the amount)
    Market::update_liquidity_b(
        &app_state.pg_pool,
        lp_position.market_id,
        payload.withdraw_amount,
        false, // false = deduct
    )
    .await
    .map_err(|e| {
        log_error!("Failed to update market liquidity: {e:?}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to update market liquidity"})).into_response(),
        )
    })?;

    // ✅ Add withdrawn amount back to user's balance
    User::update_user_balance(
        &app_state.pg_pool,
        user_id,
        payload.withdraw_amount,
        OrderSide::SELL, // BUY adds to balance (positive amount)
    )
    .await
    .map_err(|e| {
        log_error!("Failed to update user balance: {e:?}");
        // Attempt to rollback the liquidity removal? Or just log and continue
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to update user balance"})).into_response(),
        )
    })?;

    log_info!(
        "✅ User {} removed {} from LP position {}. New balance updated.",
        user_id, payload.withdraw_amount, lp_position.lp_position_id
    );

    // Get updated user balance for response
    let updated_balance = User::get_user_balance(&app_state.pg_pool, user_id)
        .await
        .map_err(|e| {
            log_error!("Failed to fetch updated balance: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": "Failed to fetch updated balance"})).into_response(),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Liquidity removed successfully",
            "data": {
                "lp_position_id": updated_position.lp_position_id,
                "withdrawn_amount": payload.withdraw_amount,
                "remaining_amount": updated_position.amount_deposited,
                "remaining_shares": updated_position.shares_of_pool,
                "is_active": updated_position.is_active,
                "new_balance": updated_balance  // ✅ Include new balance in response
            }
        })),
    ))
}





pub async fn get_fee_earnings_handler(
    State(app_state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;

    // Get all LP positions for the user
    let positions = LpPosition::get_lp_positions_by_user(&app_state.pg_pool, user_id)
        .await
        .map_err(|e| {
            log_error!("Failed to get LP positions: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": "Failed to get LP positions"})).into_response(),
            )
        })?;

    // Calculate total fees
    let total_fees: Decimal = positions.iter()
        .map(|p| p.total_fees_earned)
        .sum();

let position_details: Vec<serde_json::Value> = positions
    .into_iter()
    .map(|pos| {
        json!({
            "lp_position_id": pos.lp_position_id,
            "market_id": pos.market_id,
            "amount_deposited": pos.amount_deposited,
            "shares_of_pool": pos.shares_of_pool,      // ← ADD THIS
            "fees_earned": pos.total_fees_earned,
            "is_active": pos.is_active,
        })
    })
    .collect();

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Fee earnings fetched successfully",
            "data": {
                "total_fees_earned": total_fees,
                "positions": position_details
            }
        })),
    ))
}










