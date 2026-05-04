use axum::{Router, routing::{delete, get, patch, post},};

use crate::state::AppState;

mod get_user_trades;
mod profitsnloss;
mod liquidity;
pub mod btc_markets;


pub fn router() -> Router<AppState> {
Router::new()
    .route("/", get(get_user_trades::get_user_trades))  // ✅ Semicolon here!
    .route("/winners", get(profitsnloss::get_leaderboard_handler))
    // Add more routes as needed
      .route("/liquidity/add", post(liquidity::add_liquidity_handler))
        .route("/liquidity/remove", post(liquidity::remove_liquidity_handler))
        .route("/liquidity/fees", get(liquidity::get_fee_earnings_handler))
        .route("/liquidity/fees/total", get(liquidity::get_total_fee_earnings_handler))
        // .route("/api/liquidity/positions", get(get_lp_positions_handler))
        // .route("/api/liquidity/position/:id", get(get_lp_position_by_id_handler))
        // .route("/api/liquidity/pool/:market_id", get(get_pool_stats_handler))
}


