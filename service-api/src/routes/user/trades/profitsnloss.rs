




use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::{
  users::UserProfitRanking,
    users::User,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utility_helpers::{
    log_error, log_info,
    message_pack_helper::serialize_to_message_pack,
    nats_helper::{NatsSubjects, types::MarketOrderCreateMessage},
};
use uuid::Uuid;

use crate::{ state::AppState};

#[derive(Debug, Serialize)]
struct LeaderboardResponse {
    success: bool,
    winners: Vec<UserProfitRanking>,  // Still needs Serialize though
    losers: Vec<UserProfitRanking>,
}


pub async fn get_leaderboard_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    log_info!("📊 Leaderboard endpoint called");
    
    // Use the same function with different ordering
    let winners = match User::get_users_ranked_by_profit(&state.pg_pool, 15, 15).await {
        Ok(users) => {
            log_info!("✅ Found {} winners", users.len());
            users
        },
        Err(e) => {
            log_error!("Failed to fetch winners: {}", e);
            let error_response = serde_json::json!({
                "error": "Failed to fetch leaderboard"
            }).to_string();
            return Err((StatusCode::INTERNAL_SERVER_ERROR, error_response.into_response()));
        }
    };
    
    let losers = match User::get_top_losers(&state.pg_pool, 15).await {
        Ok(users) => {
            log_info!("✅ Found {} losers", users.len());
            users
        },
        Err(e) => {
            log_error!("Failed to fetch losers: {}", e);
            let error_response = serde_json::json!({
                "error": "Failed to fetch leaderboard"
            }).to_string();
            return Err((StatusCode::INTERNAL_SERVER_ERROR, error_response.into_response()));
        }
    };
    
    log_info!("📊 Returning leaderboard with {} winners and {} losers", winners.len(), losers.len());
    
    Ok(Json(json!({
        "success": true,
        "winners": winners,
        "losers": losers
    })).into_response())
}