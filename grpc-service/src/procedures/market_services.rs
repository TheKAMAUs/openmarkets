use std::str::FromStr;

use db_service::schema::{
    enums::MarketStatus, market::Market as SchemaMarket, user_holdings::UserHoldings,
    user_trades::UserTrades,
    market_state_history::MarketStateHistory,
};
use futures::future::join_all;
use sqlx::types::Uuid;
use sqlx::types::Decimal;
use tonic::{Request, Response, Status};
use utility_helpers::redis::keys::RedisKey;
use utility_helpers::{ log_error, log_info,};
use crate::{
    generated::markets::{
        GetMarketBookResponse, GetMarketByIdResponse, GetMarketDataRequest,
        GetMarketTradesResponse, GetPaginatedMarketResponse, GetTopHoldersResponse,
        RequestForMarketBook, RequestWithMarketId, RequestWithMarketIdAndPageRequest,
        market_service_server::MarketService,
    },
    procedures::{from_db_market, to_resp_for_market_book},
    state::SafeState,
    utils::{
        clickhouse_queries::{
            MARKET_LATEST_PRICE_QUERY, MARKET_VOLUME_BASE_QUERY, ORDER_BOOK_INITIALS,
        },
        clickhouse_schema::{GetOrderBook, MarketPriceResponse, VolumeData},
    },
    validate_numbers, validate_strings,
};

pub struct MarketServiceStub {
    pub state: SafeState,
}

#[tonic::async_trait]
impl MarketService for MarketServiceStub {
    async fn get_market_data(
        &self,
        req: Request<GetMarketDataRequest>,
    ) -> Result<Response<GetPaginatedMarketResponse>, Status> {
        let page_info = req.get_ref().page_request.clone();
        if page_info.is_none() {
            return Err(Status::invalid_argument("Page request cannot be empty"));
        }
        let page_info = page_info.unwrap();
        let page_no = page_info.page;
        let page_size = page_info.page_size;

        let market_status: MarketStatus = match req.get_ref().market_status {
            0 => MarketStatus::SETTLED,
            1 => MarketStatus::OPEN,
            2 => MarketStatus::CLOSED,
            3 => MarketStatus::SETTLED,
            _ => return Err(Status::invalid_argument("Invalid market status")),
        };

        validate_numbers!(page_no);
        validate_numbers!(page_size);

        let key = RedisKey::Markets(page_no, page_size, market_status as u64);
        if page_no == 0 || page_size == 0 {
            return Err(Status::invalid_argument(
                "Page number and size must be greater than 0",
            ));
        }

        let markets = self
            .state
            .redis_helper
            .get_or_set_cache(
                key,
                || async {
                    Ok(SchemaMarket::get_all_market_by_status_paginated(
                        &self.state.db_pool,
                        market_status,
                        page_no,
                        page_size,
                    )
                    .await?)
                },
                Some(60), // Cache for 60 seconds
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to get market {e}")))?;

     use futures::future::join_all;

let market_futures = markets
    .items
    .iter()
    .map(|item| {
        let db_pool = self.state.db_pool.clone();
        async move {
            from_db_market(item, 0.5, 0.5, &self.state).await
        }
    })
    .collect::<Vec<_>>();

let markets_result = join_all(market_futures).await;

let response = GetPaginatedMarketResponse {
    markets: markets_result,
    page_info: Some(markets.page_info.into()),
};

        Ok(Response::new(response))
    }


    

async fn get_market_by_id(
    &self,
    req: Request<RequestWithMarketId>,
) -> Result<Response<GetMarketByIdResponse>, Status> {

    log_info!("🔍 [get_market_by_id] START - Request received");

    // -------------------------------
    // 1. Parse + Validate Input
    // -------------------------------
    let market_id_raw = req.into_inner().market_id;
    log_info!("📦 Raw market_id from request: {}", market_id_raw);

    validate_strings!(market_id_raw);
    log_info!("✅ String validation passed");

    let market_id = Uuid::from_str(&market_id_raw)
        .map_err(|e| {
            log_info!("❌ Invalid UUID format: {}", e);
            Status::invalid_argument("Invalid market id")
        })?;

    log_info!("✅ Parsed UUID: {}", market_id);

    // -------------------------------
    // 2. Redis cache lookup (Market only)
    // -------------------------------
    let key = RedisKey::Market(market_id);
    log_info!("🔑 Redis key: {:?}", key);

    let market_opt = self
        .state
        .redis_helper
        .get_or_set_cache(
            key,
            || async {
                log_info!("📦 Cache miss — fetching from DB");

                let result =
                    SchemaMarket::get_market_by_id(&self.state.db_pool, &market_id).await;

                match &result {
                    Ok(Some(_)) => log_info!("✅ DB found market"),
                    Ok(None) => log_info!("ℹ️ Market not found"),
                    Err(e) => log_info!("❌ DB error: {}", e),
                }

                Ok(result?)
            },
            Some(60),
        )
        .await
        .map_err(|e| {
            log_info!("❌ Redis/DB error: {}", e);
            Status::internal(format!("Failed to get market: {e}"))
        })?;

    let market = match market_opt {
        Some(m) => m,
        None => {
            log_info!("❌ Market not found");
            return Err(Status::not_found("Market not found"));
        }
    };

    log_info!("✅ Market loaded");

    // -------------------------------
    // 3. Transform market
    // -------------------------------
    let market_response =
        from_db_market(&market, 0.5, 0.5, &self.state).await;

    // -------------------------------
    // 4. Fetch volume (DB only)
    // -------------------------------
    log_info!("📊 Fetching volume...");

 // In your get_market_by_id function, replace the volume section:
let volume_row = SchemaMarket::get_volume_info_by_market_id(
    &self.state.db_pool,
    market_id,
)
.await
.map_err(|e| {
    Status::internal(format!("Volume DB error: {}", e))
})?;

// Convert VolumeInfo to VolumeData with proper f64 conversion
let volume_info = VolumeData {
    market_id,
    yes_buy_qty: volume_row.yes_buy_qty
        .unwrap_or(Decimal::ZERO)
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0),
    yes_buy_usd: volume_row.yes_buy_usd
        .unwrap_or(Decimal::ZERO)
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0),
    yes_sell_qty: volume_row.yes_sell_qty
        .unwrap_or(Decimal::ZERO)
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0),
    yes_sell_usd: volume_row.yes_sell_usd
        .unwrap_or(Decimal::ZERO)
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0),
    no_buy_qty: volume_row.no_buy_qty
        .unwrap_or(Decimal::ZERO)
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0),
    no_buy_usd: volume_row.no_buy_usd
        .unwrap_or(Decimal::ZERO)
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0),
    no_sell_qty: volume_row.no_sell_qty
        .unwrap_or(Decimal::ZERO)
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0),
    no_sell_usd: volume_row.no_sell_usd
        .unwrap_or(Decimal::ZERO)
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0),
};

 

// -------------------------------
// 5. GET LATEST PRICE FROM HISTORY
// -------------------------------
log_info!("💰 Fetching latest price from history...");



let price_history = MarketStateHistory::get_latest_market_state(
    &self.state.db_pool,
    market_id,
   
)
.await
.map_err(|e| Status::internal(format!("Price fetch failed: {}", e)))?;

let latest_price = price_history.into_iter().last();



let (yes_price_decimal, no_price_decimal) = latest_price
    .map(|row| (row.price_yes, row.price_no))
  .unwrap_or((
        Decimal::new(5, 1),  // 5 with 1 decimal place = 0.5
        Decimal::new(5, 1),
    ));

// Convert to f64 for the response
let yes_price = yes_price_decimal.to_string().parse::<f64>().unwrap_or(0.5);
let no_price = no_price_decimal.to_string().parse::<f64>().unwrap_or(0.5);

log_info!(
    "📊 Latest price - YES: {}, NO: {}",
    yes_price,
    no_price
);

// -------------------------------
// 6. Build response
// -------------------------------
let price_info = MarketPriceResponse {
    market_id,
    latest_yes_price: yes_price,
    latest_no_price: no_price,
};

    let resp = GetMarketByIdResponse {
        market: Some(market_response),
        volume_info: Some(volume_info.into()),
        market_price: Some(price_info.into()),
    };

    log_info!("✅ [get_market_by_id] SUCCESS");

    Ok(Response::new(resp))

}

    
// async fn get_market_by_id(
//     &self,
//     req: Request<RequestWithMarketId>,
// ) -> Result<Response<GetMarketByIdResponse>, Status> {
    
//     log_info!("🔍 [get_market_by_id] START - Request received");
    
//     let market_id = req.into_inner().market_id;
//     log_info!("📦 Raw market_id from request: {}", market_id);
    
//     validate_strings!(market_id);
//     log_info!("✅ String validation passed");

//     let market_id = Uuid::from_str(&market_id)
//         .map_err(|e| {
//             log_info!("❌ Invalid UUID format: {}", e);
//             Status::invalid_argument("Invalid market id")
//         })?;
//     log_info!("✅ Parsed UUID: {}", market_id);

//     let key = RedisKey::Market(market_id);
//     log_info!("🔑 Redis key: {:?}", key);

//     log_info!("🔄 Attempting to get from cache or DB...");
//     let market = self
//         .state
//         .redis_helper
//         .get_or_set_cache(
//             key,
//             || async {
//                 log_info!("📦 Cache miss - fetching from database");
//                 let result = SchemaMarket::get_market_by_id(&self.state.db_pool, &market_id).await;
//                 match &result {
//                     Ok(Some(_)) => log_info!("✅ Database found market"),
//                     Ok(None) => log_info!("ℹ️ Database returned None - market not found"),
//                     Err(e) => log_info!("❌ Database error: {}", e),
//                 }
//                 Ok(result?)
//             },
//             Some(60), // Cache for 60 seconds
//         )
//         .await
//         .map_err(|e| {
//             log_info!("❌ Redis/DB error: {}", e);
//             Status::internal(format!("Failed to get market id {e}"))
//         })?;
    
//     log_info!("🔍 Cache/DB result: market is {:?}", if market.is_some() { "SOME" } else { "NONE" });

//     if let Some(market) = market {
//         log_info!("✅ Market found in DB/cache");
//         let market = from_db_market(&market, 0.5, 0.5, &self.state );
//         log_info!("📊 Converted to response format");

//         log_info!("🔄 Fetching ClickHouse data...");
//         // we are not caching the volume info in Redis, as it changes frequently

//         let get_volume_info_future = self
//             .state
//             .clickhouse_client
//             .query(MARKET_VOLUME_BASE_QUERY)
//             .bind(market_id)
//             .bind("365 DAY") // 100 years = 36500 days
//             .fetch_all::<VolumeData>();  // ← Changed to fetch_all
        
//         log_info!("📊 Volume query prepared");
        
//         let get_market_price_future = self
//             .state
//             .clickhouse_client
//             .query(MARKET_LATEST_PRICE_QUERY)
//             .bind(market_id)
//             .fetch_all::<MarketPriceResponse>();  // ← Changed to fetch_all
        
//         log_info!("💰 Price query prepared");

//         log_info!("⏳ Waiting for ClickHouse queries to complete...");
        
//        // Change this part:
// let (volume_rows, price_rows) =
//     tokio::try_join!(get_volume_info_future, get_market_price_future)
//         .map_err(|e| {
//             log_info!("❌ ClickHouse error: {}", e);
//             Status::internal(format!("Failed to fetch market data: {}", e))
//         })?;

// log_info!("✅ ClickHouse queries completed");

// // Handle the Vec results like the working code
// let volume_info_resp = if let Some(vol) = volume_rows.first() {
//     log_info!("✅ Volume data found");
//     vol.clone()
// } else {
//     log_info!("⚠️ No volume data found - using defaults");
//     VolumeData {
//         market_id,
//         ..Default::default()
//     }
// };

// let market_price_resp = if let Some(price) = price_rows.first() {
//     log_info!("✅ Price data found");
//     price.clone()
// } else {
//     log_info!("⚠️ No price data found - using defaults");
//     MarketPriceResponse {
//         market_id,
//         ..Default::default()
//     }
// };

// let response = GetMarketByIdResponse {
//     market: Some(market.await),
//     volume_info: Some(volume_info_resp.into()),      // Add .into()
//     market_price: Some(market_price_resp.into()),    // Add .into()
// };
        
//         log_info!("✅ [get_market_by_id] SUCCESS - returning response");
//         return Ok(Response::new(response));
//     }

//     log_info!("❌ Market not found with id: {}", market_id);
//     Err(Status::not_found(format!(
//         "Market with {market_id} not found"
//     )))
// }




  async fn get_market_book(
    &self,
    req: Request<RequestForMarketBook>,
) -> Result<Response<GetMarketBookResponse>, Status> {
    
    log_info!("📚 [get_market_book] START - Request received");
    
    let market_id = &req.get_ref().market_id;
    let depth = req.get_ref().depth;
    
    log_info!("📦 Market ID: {}, Depth: {}", market_id, depth);
    
    validate_numbers!(depth);
    log_info!("✅ Depth validation passed");
    
    validate_strings!(market_id);
    log_info!("✅ Market ID string validation passed");

    let market_id = Uuid::from_str(&market_id)
        .map_err(|e| {
            log_info!("❌ Invalid UUID format: {}", e);
            Status::invalid_argument("Invalid market id")
        })?;
    log_info!("✅ Parsed UUID: {}", market_id);

    log_info!("🔄 Querying ClickHouse for order book...");
    log_info!("📊 Query: ORDER_BOOK_INITIALS");
    log_info!("🎯 Parameters: depth={}, depth={}, depth={}, depth={}, market_id={}", 
        depth, depth, depth, depth, market_id);

    let order_book_initials = self
        .state
        .clickhouse_client
        .query(ORDER_BOOK_INITIALS)
        .bind(depth)
        .bind(depth)
        .bind(depth)
        .bind(depth)
        .bind(market_id)
        .fetch_optional::<GetOrderBook>()
        .await
        .map_err(|e| {
            log_info!("❌ ClickHouse query failed: {}", e);
            Status::internal(format!("Failed to fetch market book: {}", e))
        })?;

    log_info!("🔍 ClickHouse result: {:?}", 
        if order_book_initials.is_some() { "SOME" } else { "NONE" });

    if order_book_initials.is_none() {
        log_info!("❌ No order book found for market: {}", market_id);
        return Err(Status::not_found(format!(
            "Market book for market id {market_id} not found"
        )));
    }

    log_info!("✅ Order book found, converting to response format");
    let order_book = to_resp_for_market_book(order_book_initials.unwrap());
    
    log_info!("✅ [get_market_book] SUCCESS - returning response");
    let response = Response::new(order_book);

    Ok(response)
}





    async fn get_top_holders(
        &self,
        req: Request<RequestWithMarketId>,
    ) -> Result<Response<GetTopHoldersResponse>, Status> {
        let market_id = req.into_inner().market_id;
        validate_strings!(market_id);

        let market_id = Uuid::from_str(&market_id)
            .map_err(|_| Status::invalid_argument("Invalid market id"))?;

        let top_holders = UserHoldings::get_top_holders(
            &self.state.db_pool,
            market_id,
            self.state.admin_username.clone(),
            10,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to get top holders: {}", e)))?;

        let response = GetTopHoldersResponse {
            market_id: market_id.to_string(),
            top_holders: top_holders.into_iter().map(Into::into).collect(),
        };

        Ok(Response::new(response))
    }

    async fn get_market_trades(
        &self,
        req: Request<RequestWithMarketIdAndPageRequest>,
    ) -> Result<Response<GetMarketTradesResponse>, Status> {
        let market_id = req.get_ref().market_id.clone();
        let page_request = req.get_ref().page_request;

        if page_request.is_none() {
            return Err(Status::invalid_argument("Page request cannot be empty"));
        }
        let page_request = page_request.unwrap();

        validate_strings!(market_id);
        validate_numbers!(page_request.page);
        validate_numbers!(page_request.page_size);
        let market_id = Uuid::from_str(&market_id)
            .map_err(|_| Status::invalid_argument("Invalid market id"))?;

        let paginated_response = UserTrades::get_market_trades_paginated(
            market_id,
            self.state.admin_username.clone(),
            page_request.page,
            page_request.page_size,
            &self.state.db_pool,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to get market trades: {}", e)))?;

        let response = GetMarketTradesResponse {
            market_id: market_id.to_string(),
            trades: paginated_response
                .items
                .into_iter()
                .map(Into::into)
                .collect(),
            page_info: Some(paginated_response.page_info.into()),
        };

        Ok(Response::new(response))
    }
}



#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use sqlx::types::Uuid;

    use crate::utils::clickhouse_schema::{GetOrderBook, MarketPriceResponse, VolumeData};

    #[tokio::test]
    #[ignore = "Requires market id"]
    async fn test_get_market_data() {
        let client = clickhouse::Client::default()
            .with_url("http://localhost:8123")
            .with_database("polyMarket")
            .with_user("polyMarket")
            .with_password("polyMarket");
        let market_id = Uuid::from_str("91afed7f-6004-4968-984f-cdc968ae6013").unwrap();
        let depth = 10;

        let resp = client
            .query(
                r#"
                 SELECT
                    market_id,
                    ts,
                    created_at,

                    CAST(arraySlice(
                        arrayFilter(x -> x.2 > 0, yes_bids), 1, ?
                        ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS yes_bids,
                    CAST(arraySlice(
                        arrayFilter(x -> x.2 > 0, yes_asks), 1, ?
                        ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS yes_asks,
                    CAST(arraySlice(
                        arrayFilter(x -> x.2 > 0, no_bids), 1, ?
                        ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS no_bids,
                    CAST(arraySlice(
                        arrayFilter(x -> x.2 > 0, no_asks), 1, ?
                        ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS no_asks
                FROM market_order_book WHERE market_id = ?
                ORDER BY ts DESC
                LIMIT 1
            "#,
            )
            .bind(depth)
            .bind(depth)
            .bind(depth)
            .bind(depth)
            .bind(market_id)
            .fetch_optional::<GetOrderBook>()
            .await
            .inspect_err(|e| {
                log_info!("Error fetching market data: {}", e);
            })
            .unwrap();

        assert!(resp.is_some(), "Response should not be empty");
    }

    #[tokio::test]
    #[ignore = "Requires market id"]
    async fn test_get_market_volume() {
        let client = clickhouse::Client::default()
            .with_url("http://localhost:8123")
            .with_database("polyMarket")
            .with_user("polyMarket")
            .with_password("polyMarket");

        let market_id = Uuid::from_str("91afed7f-6004-4968-984f-cdc968ae6013").unwrap();

        let result = client
            .query(
                r#"
                   SELECT
                        market_id,

                        -- YES - BUY
                        toFloat64(SUM(if(outcome = 'yes' AND side = 'buy', quantity, 0))) AS yes_buy_qty,
                        toFloat64(SUM(if(outcome = 'yes' AND side = 'buy', amount, 0))) AS yes_buy_usd,

                        -- YES - SELL
                        toFloat64(SUM(if(outcome = 'yes' AND side = 'sell', quantity, 0))) AS yes_sell_qty,
                        toFloat64(SUM(if(outcome = 'yes' AND side = 'sell', amount, 0))) AS yes_sell_usd,

                        -- NO - BUY
                        toFloat64(SUM(if(outcome = 'no' AND side = 'buy', quantity, 0))) AS no_buy_qty,
                        toFloat64(SUM(if(outcome = 'no' AND side = 'buy', amount, 0))) AS no_buy_usd,

                        -- NO - SELL
                        toFloat64(SUM(if(outcome = 'no' AND side = 'sell', quantity, 0))) AS no_sell_qty,
                        toFloat64(SUM(if(outcome = 'no' AND side = 'sell', amount, 0))) AS no_sell_usd

                    FROM market_volume_data
                    WHERE
                        market_id = ? AND
                        ts >= now() - INTERVAL ?
                    GROUP BY market_id
                "#,
            )
            .bind(market_id)
            .bind("1 DAY") 
            .fetch_one::<VolumeData>()
            .await;

        let result = result
            .inspect_err(|e| {
                log_info!("Error fetching market volume data: {}", e);
            })
            .unwrap();
        log_info!("Market Volume Data: {:#?}", result);
        assert_eq!(result.market_id, market_id);
    }

    #[tokio::test]
    async fn test_get_latest_market_price() {
        let client = clickhouse::Client::default()
            .with_url("http://localhost:8123")
            .with_database("polyMarket")
            .with_user("polyMarket")
            .with_password("polyMarket");

        let market_id = Uuid::from_str("91afed7f-6004-4968-984f-cdc968ae6013").unwrap();

        let result = client
            .query(
                r#"
                SELECT
                    market_id,
                    toFloat64(argMax(yes_price, ts)) AS latest_yes_price,
                    toFloat64(argMax(no_price, ts)) AS latest_no_price
                FROM market_price_data
                WHERE market_id = ?
                GROUP BY market_id
                "#,
            )
            .bind(market_id)
            .fetch_optional::<MarketPriceResponse>()
            .await;

        let result = result
            .inspect_err(|e| {
                log_info!("Error fetching latest market price: {}", e);
            })
            .unwrap();
        log_info!("Latest Market Price: {:#?}", result);
    }
}
