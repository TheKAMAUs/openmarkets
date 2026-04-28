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
    market::Market,

};
use auth_service::types::SessionTokenClaims;

use crate::{state::AppState, };

use utility_helpers::{
   log_warn, log_error, log_info,};



pub async fn get_btc_markets_handler(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    
    let markets = Market::get_btcusdt_markets_updated_last_20min(&app_state.pg_pool)
        .await
        .map_err(|e| {
            log_error!("Failed to fetch BTCUSDT markets: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Failed to fetch markets"
                }))
                .into_response(),
            )
        })?;

    Ok(Json(json!({
        "message": "BTCUSDT markets updated in last 20 minutes",
        "data": markets
    })))
}