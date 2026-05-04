use std::str::FromStr;
use axum::extract::ws::Message as WsSendMessage;
use serde_json::json;
use utility_helpers::{log_error, log_info, ws::types::ChannelType};
use uuid::Uuid;
use crate::{SafeAppState, core::send_message};
use serde::{Deserialize, Serialize};
use prost::Message;

use proto_defs::proto_types::{ws_common_types::WsData, ws_market_price::BitcoinPricePayload as ProtoBitcoinPricePayload, ws_market_price::MarketMessage as ProtoMarketMessage    };
use proto_defs::proto_types::ws_market_price::market_message::Payload;

// Struct for parsing incoming JSON from Order Service
#[derive(Debug, serde::Deserialize)]
struct BitcoinPriceJsonPayload {
    symbol: String,
    market_id: String,           // 1 & 2 match ✅
    current_price: f64,          // 3
    target_price: f64,           // 4
    target_timestamp: i64,       // 5
    seconds_remaining: u64,      // 6
    is_above_target: bool,       // 7
    timestamp: i64,              // 8
    price_difference: f64,       // 9
    percent_to_target: f64,      // 10
    less_than_40_secs: bool,     // 11 - moved to correct position
}

pub async fn bitcoin_price_handler_bin(
    data: &WsData,
    state: &SafeAppState,
    client_id: &Uuid,
) -> usize {
    let mut served_clients = 0;
    
    // Parse the JSON params FROM the incoming message
    // (The order service sends JSON, so we parse JSON first)
    if let Ok(price_payload) = serde_json::from_str::<BitcoinPriceJsonPayload>(&data.params) {
        log_info!(
            "📊 Received Bitcoin price update from client {}: {} - Current: ${:.2}, Target: ${:.2}, Remaining: {}s, Above: {}, <40: {}",
            client_id, 
            price_payload.symbol, 
            price_payload.current_price,
            price_payload.target_price,
            price_payload.seconds_remaining,
            price_payload.is_above_target,
             price_payload.less_than_40_secs
        );
        
     // ✅ Create PROTOBUF message with ALL fields for broadcasting
// In bitcoin_price_handler_bin
let proto_payload = ProtoBitcoinPricePayload {
    symbol: price_payload.symbol.clone(),
    market_id: price_payload.market_id.clone(),
    current_price: price_payload.current_price,
    target_price: price_payload.target_price,
    target_timestamp: price_payload.target_timestamp,
    seconds_remaining: price_payload.seconds_remaining,
    is_above_target: price_payload.is_above_target,
    timestamp: price_payload.timestamp,
    price_difference: price_payload.price_difference,
    percent_to_target: price_payload.percent_to_target,
    less_than_40_secs: price_payload.less_than_40_secs,  // Moved to position 11
};
        
        // ✅ Encode as protobuf bytes
      
        let message = ProtoMarketMessage {
  payload: Some(Payload::Price(proto_payload)),
};

let data_to_send = message.encode_to_vec();
        log_info!("📦 Encoded protobuf message: {} bytes", data_to_send.len());
        
        // Broadcast to subscribed clients
        let clients = state.client_manager.write().await;
        let symbol_channel = ChannelType::BitcoinPrice(price_payload.symbol.clone());
        let clients = clients.get_clients(&symbol_channel);
        
        if let Some(clients) = clients {
            for (subscribed_client_id, client_tx) in clients.iter() {
                // Don't echo back to the sender
                if subscribed_client_id == client_id {
                    continue;
                }
                
                // ✅ Send as BINARY (protobuf)
                if let Err(e) = send_message(
                    client_tx,
                    WsSendMessage::Binary(data_to_send.clone().into()),
                )
                .await
                {
                    log_error!("Failed to send Bitcoin price to {} - {:#?}", subscribed_client_id, e);
                } else {
                    served_clients += 1;
                    log_info!("✅ Sent Bitcoin price (protobuf) to client {}", subscribed_client_id);
                }
            }
        }
        
    } else {
        log_error!(
            "Failed to parse Bitcoin price params from client {}: {}",
            client_id, data.params
        );
    }
    
    log_info!("📊 Bitcoin price update served to {} clients", served_clients);
    served_clients
}

