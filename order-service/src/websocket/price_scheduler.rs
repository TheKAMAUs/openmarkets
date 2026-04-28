use tokio::time::{interval,sleep, Duration};
use serde_json::json;
use chrono::Utc;
use std::sync::Arc;
use prost::Message as ProstMessage;

use proto_defs::proto_types::ws_common_types::{
    Channel, OperationType, Payload, WsData, WsMessage,
};
use crate::state::AppState;
use tokio_tungstenite::tungstenite::Message as WsMessageBTCType;
use utility_helpers::{log_info, log_error};
use futures_util::SinkExt;
use tokio::sync::RwLock;
use std::time::Instant;
use db_service::schema::{
    enums::{OrderSide, OrderStatus,Outcome},

    market::Market,
};
use sqlx::types::Uuid;
use rust_decimal::Decimal;



#[derive(Debug, Clone)]
pub struct BtcPriceTracker {
    pub market_id: Uuid,           // Current active market
    pub previous_market_id: Uuid,  // Previous market (to be resolved)
    pub target_price: f64,
    pub target_timestamp: i64,
    pub current_price: f64,
    pub seconds_remaining: u64,
    pub is_above_target: bool,
    pub less_than_40_secs: bool,
}


pub async fn start_btc_ws_scheduler(app_state: Arc<AppState>) {
    log_info!("🚀 Starting BTC WebSocket scheduler...");
    
    // Shared state for tracking
    let tracker = Arc::new(RwLock::new(BtcPriceTracker {
        market_id: Uuid::nil(),
        previous_market_id: Uuid::nil(),
        target_price: 0.0,
        target_timestamp: 0,
        current_price: 0.0,
        seconds_remaining: 300,
        is_above_target: false,
        less_than_40_secs: false,
    }));
    
    // ✅ Clone app_state for Task 1
    let app_state_task1 = app_state.clone();
    let tracker_clone = tracker.clone();
    
    // Task 1: Create new market and resolve previous every 5 minutes
    tokio::spawn(async move {
           // ✅ STEP 1: Align to next 5-min boundary
        let now = Utc::now();
        let seconds = now.timestamp() % 300;
        let wait_seconds = 300 - seconds;

        log_info!("⏳ Waiting {} seconds to align with 5-min boundary...", wait_seconds);
        sleep(Duration::from_secs(wait_seconds as u64)).await;

        // ✅ STEP 2: Now start fixed interval
        let mut target_timer = interval(Duration::from_secs(300));
        
        loop {
            target_timer.tick().await;
            
          // Fetch current BTC price
            let current_price = match fetch_kraken_btc_price().await {
                Ok(price) => price,
                Err(e) => {
                    log_error!("❌ Failed to fetch BTC price: {}", e);
                    continue;
                }
            };
            
            // ✅ STEP 1: RESOLVE PREVIOUS MARKET (if exists)
            let previous_market_id = {
                let tracker = tracker_clone.read().await;
                tracker.market_id  // Current market becomes previous
            };
            
            if previous_market_id != Uuid::nil() {
                log_info!("🔍 Resolving previous market: {}", previous_market_id);
                
                // ✅ CALL RESOLVE HERE
                match resolve_bitcoin_market(&app_state_task1, previous_market_id).await {
                    Ok(outcome) => {
                        log_info!("✅ Market {} resolved with outcome {:?}", previous_market_id, outcome);
                    }
                    Err(e) => {
                        log_error!("❌ Failed to resolve market {}: {}", previous_market_id, e);
                    }
                }
            } else {
                log_info!("ℹ️ No previous market to resolve (first run)");
            }
            
            // Step 2: Create NEW Bitcoin market
            let new_market_id = create_new_bitcoin_market(&app_state_task1, current_price).await
                .unwrap_or_else(|| {
                    log_error!("❌ Failed to create Bitcoin market");
                    Uuid::nil()
                });
            
            // Step 3: Update tracker
            {
                let mut tracker = tracker_clone.write().await;
                tracker.previous_market_id = tracker.market_id;
                tracker.market_id = new_market_id;
                tracker.target_price = current_price ;
                tracker.target_timestamp = Utc::now().timestamp_millis();
                tracker.seconds_remaining = 300;
                tracker.less_than_40_secs = true;
                
                log_info!(
                    "🎯 New target set - Market: {}, Previous: {}, Target: ${:.0}", 
                    new_market_id, previous_market_id, tracker.target_price
                );
            }
        }
    });
    
    // ✅ Clone app_state for Task 2 (separate clone)
    let app_state_task2 = app_state.clone();
    let tracker_clone2 = tracker.clone();
    
    // Task 2: Per-second updates for current price and countdown
    tokio::spawn(async move {
        let mut per_second_timer = interval(Duration::from_secs(1));
        let mut last_price_fetch = Instant::now();
        let mut cached_price = 0.0;
        
        // Send initial price immediately
        if let Ok(price) = fetch_kraken_btc_price().await {
            cached_price = price;
            let mut tracker = tracker_clone2.write().await;
            tracker.current_price = price;
            tracker.is_above_target = price >= tracker.target_price;
            tracker.less_than_40_secs = tracker.seconds_remaining < 40;
            
            if tracker.target_price == 0.0 {
                tracker.target_price = price;
                tracker.target_timestamp = Utc::now().timestamp_millis();
            }
        }
        
        loop {
            per_second_timer.tick().await;
            
            // Fetch new price every 10 seconds
            if last_price_fetch.elapsed() >= Duration::from_secs(1) {
                if let Ok(price) = fetch_kraken_btc_price().await {
                    cached_price = price;
                    last_price_fetch = Instant::now();
                }
            }
            
            // Update tracker
            {
                let mut tracker = tracker_clone2.write().await;
                tracker.current_price = cached_price;
                
                if tracker.seconds_remaining > 0 {
                    tracker.seconds_remaining -= 1;
                }
                
                tracker.is_above_target = cached_price >= tracker.target_price;
                tracker.less_than_40_secs = tracker.seconds_remaining < 40;

                // ✅ Use app_state_task2 here
                send_btc_price_update(&app_state_task2, &tracker).await;
                
                log_info!(
                    "📊 BTC - Market: {} | Current: ${:.2} | Target: ${:.0} | Remaining: {}s | Above: {} | <40s: {}",
                    tracker.market_id,
                    tracker.current_price,
                    tracker.target_price,
                    tracker.seconds_remaining,
                    tracker.is_above_target,
                    tracker.less_than_40_secs
                );
            }
        }
    });
    
    log_info!("✅ BTC WebSocket scheduler started successfully");
}



async fn send_btc_price_update(app_state: &Arc<AppState>, tracker: &BtcPriceTracker) {
    let price_json = json!({
        "symbol": "BTCUSDT",
        "market_id": tracker.market_id.to_string(),  // ✅ Include market ID
        "current_price": tracker.current_price,
        "target_price": tracker.target_price,
        "target_timestamp": tracker.target_timestamp,
        "seconds_remaining": tracker.seconds_remaining,
        "is_above_target": tracker.is_above_target,
        "less_than_40_secs": tracker.less_than_40_secs,
        "timestamp": Utc::now().timestamp_millis(),
        "price_difference": tracker.current_price - tracker.target_price,
        "percent_to_target": ((tracker.current_price - tracker.target_price) / tracker.target_price) * 100.0,
    });

    let params_string = price_json.to_string();
    
    let message = WsMessage {
        id: Some(format!("btc_update_{}", Utc::now().timestamp_millis())),
        payload: Some(Payload {
            ops: OperationType::Post as i32,
            data: Some(WsData {
                channel: Channel::Bitcoinprice as i32,
                params: params_string,
            }),
        }),
    };

    let bin_data = message.encode_to_vec();
    
    match app_state.ws_tx.write().await.send(WsMessageBTCType::Binary(bin_data.into())).await {
        Ok(_) => {
            // log_info!("✅ BTC update sent - Market: {}, Current: ${:.2}, Target: ${:.2}", 
            //     tracker.market_id, tracker.current_price, tracker.target_price);
        }
        Err(e) => {
            log_error!("❌ Failed to send BTC update: {}", e);
        }
    }
}


async fn create_new_bitcoin_market(app_state: &Arc<AppState>, current_price: f64) -> Option<Uuid> {
 // ✅ Use exact price (already 2 decimals from Kraken)
    let target_price = current_price;
    
  // Always "above" condition
let condition = "above";
    
     let expiry = Utc::now() + chrono::Duration::minutes(5);
    let expiry_naive = expiry.naive_utc();

    
    // ✅ Simple name: just BTC/USDT
    let name = "BTCUSDT".to_string();
    
    // ✅ Keep description as is
    let description = format!(
        "Will BTC/USDT be {} ${:.2} at {} UTC? Current price: ${:.2}",
        condition, target_price, expiry.format("%H:%M"), current_price
    );

 // Store target price and condition in resolution_criteria as JSON
    let resolution_criteria = json!({
        "target_price": target_price,
        "condition": condition,
        "resolved_price": null
    }).to_string();

    log_info!("🆕 Creating new Bitcoin market: {} with target ${:.0}", name, target_price);

    // ✅ Use your existing Market::create_new_market method
    let market = match Market::create_new_market(
        name,
        description.clone(),    
   vec!["https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcTJHh416u6o5_GJUMGventF-FwPg7MN-YvPlA&s".to_string()],
       Decimal::new(400000, 2),// 4000.00// liquidity_b
        expiry_naive,
        &app_state.db_pool,
        None,                             // parent_id
        false,                            // is_event
        None,                             // child_market_ids
        Some("Crypto".to_string()),       // category
        Some(resolution_criteria),        // resolution_criteria - stores target/condition
        None,                             // slug
    ).await {
        Ok(m) => m,
        Err(e) => {
            log_error!("❌ Failed to create Bitcoin market: {}", e);
            return None;
        }
    };

    log_info!("✅ New Bitcoin market created: {} - {}", market.id, description);
    

    
    Some(market.id,)
}



async fn resolve_bitcoin_market(
    app_state: &Arc<AppState>,
    market_id: Uuid,
) -> Result<Outcome, String> {
    log_info!("🔍 Resolving Bitcoin market: {}", market_id);

    // ✅ Use your existing Market::get_market_by_id method
    let market = Market::get_market_by_id(&app_state.db_pool, &market_id)
        .await
        .map_err(|e| format!("Failed to get market: {:?}", e))?
        .ok_or_else(|| format!("Market {} not found", market_id))?;

    // Verify it's a Crypto market
    if market.category.as_deref() != Some("Crypto") {
        return Err(format!("Market {} is not a Crypto market", market_id));
    }

    // Parse resolution criteria to get target price and condition
    let criteria: serde_json::Value = market
        .resolution_criteria
        .as_ref()
        .ok_or_else(|| format!("No resolution criteria for market {}", market_id))
        .and_then(|c| {
            serde_json::from_str(c).map_err(|e| format!("Failed to parse criteria: {}", e))
        })?;

    let target_price = criteria["target_price"]
        .as_f64()
        .ok_or_else(|| format!("Missing target_price in criteria"))?;
    
    let condition = criteria["condition"]
        .as_str()
        .ok_or_else(|| format!("Missing condition in criteria"))?;

    // Fetch current BTC price for resolution
    let current_price = fetch_kraken_btc_price()
        .await
        .map_err(|e| format!("Failed to fetch BTC price: {}", e))?;

    // Determine winner as Outcome
    let winning_outcome = match condition {
        "above" => {
            if current_price >= target_price {
                Outcome::YES
            } else {
                Outcome::NO
            }
        }
        "below" => {
            if current_price <= target_price {
                Outcome::YES
            } else {
                Outcome::NO
            }
        }
        _ => Outcome::NO,
    };

    log_info!(
        "📊 Market {} resolved: Target=${:.2}, Current=${:.2}, Condition={}, Winner={:?}",
        market_id, target_price, current_price, condition, winning_outcome
    );

    // ✅ Call settle_market with the Outcome directly
    Market::settle_market(
        &app_state.db_pool,
        &market_id,
        winning_outcome,        // ✅ Already an Outcome
        Some(current_price),    // ✅ Pass the resolved price
    )
    .await
    .map_err(|e| format!("Failed to settle market: {}", e))?;

    log_info!("✅ Market {} resolved and settled successfully", market_id);
    
    Ok(winning_outcome)
}






async fn fetch_kraken_btc_price() -> anyhow::Result<f64> {
    let url = "https://api.kraken.com/0/public/Ticker?pair=BTCUSDT";
    let client = reqwest::Client::new();
    
    let resp = client
        .get(url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch Kraken price: {}", e))?;
    
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))?;
    
    let price = body["result"]["XBTUSDT"]["c"][0]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to extract price from response"))?
        .parse::<f64>()
        .map_err(|e| anyhow::anyhow!("Failed to parse price as f64: {}", e))?;
    
    Ok(price)
}






// pub async fn start_btc_ws_scheduler(app_state: Arc<AppState>) {
//     log_info!("🚀 Starting BTC WebSocket scheduler...");
    
//     tokio::spawn(async move {
//         log_info!("⏰ BTC scheduler task spawned, will run every 5 minutes");
//         let mut timer = interval(Duration::from_secs(5)); // 5 sec

//         // Send first price immediately
//         log_info!("📊 Sending initial BTC price...");
//         send_btc_price(&app_state).await;

//         loop {
//             log_info!("⏰ Waiting 5 minutes until next BTC price update...");
//             timer.tick().await;
//             log_info!("⏰ Timer ticked - fetching and sending BTC price update");
//             send_btc_price(&app_state).await;
//         }
//     });
    
//     log_info!("✅ BTC WebSocket scheduler started successfully");
// }

// async fn send_btc_price(app_state: &Arc<AppState>) {
//     log_info!("🔄 Fetching BTC price from Kraken...");
    
//     match fetch_kraken_btc_price().await {
//         Ok(price) => {
//             log_info!("✅ Successfully fetched BTC price: ${:.2}", price);
            
//             // Create PriceData protobuf message
//             // ✅ Create JSON string for params field
//             let price_json = json!({
//                 "symbol": "BTCUSDT",
//                 "price": price,
//                 "timestamp": Utc::now().timestamp_millis()
//             });

     

//             let params_bytes = price_json.to_string();
            
//             // Create WsMessage
//             let message = WsMessage {
//                 id: None,
//                 payload: Some(Payload {
//                     ops: OperationType::Post as i32,
//                     data: Some(WsData {
//                         channel: Channel::Bitcoinprice as i32,
//                         params: params_bytes,
//                     }),
//                 }),
//             };

//             let bin_data = message.encode_to_vec();
//             log_info!("📦 Serialized protobuf message: {} bytes", bin_data.len());

//             log_info!("📤 Sending BTC price via WebSocket...");
            
//             // Use your AppState's ws_tx
//             match app_state.ws_tx.write().await.send(WsMessageBTCType::Binary(bin_data.into())).await {
//                 Ok(_) => {
//                     log_info!("✅ BTC price sent via WebSocket: ${:.2}", price);
//                 }
//                 Err(e) => {
//                     log_error!("❌ Failed to send BTC price over WS: {}", e);
//                 }
//             }
//         }
//         Err(e) => {
//             log_error!("❌ Error fetching BTC price: {}", e);
//         }
//     }
// }