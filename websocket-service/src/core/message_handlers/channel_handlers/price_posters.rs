use std::str::FromStr;

use axum::extract::ws::Message as WsSendMessage;
use prost::Message;
use proto_defs::proto_types::{ws_common_types::WsData, ws_market_price::WsParamsPayload,  ws_market_price::MarketMessage as ProtoMarketMessage  };
use utility_helpers::{log_error, log_info, ws::types::ChannelType};
use uuid::Uuid;

use proto_defs::proto_types::ws_market_price::market_message::Payload;

use crate::{SafeAppState, core::send_message};

pub async fn price_poster_handler_bin(
    data: &WsData,
    state: &SafeAppState,
    client_id: &Uuid,
) -> usize {
    let mut served_clients = 0;
    if let Ok(msg_payload) = serde_json::from_str::<WsParamsPayload>(&data.params) {
        // broadcast the message to all clients
        let clients = state.client_manager.write().await;
        let market_id = Uuid::from_str(&msg_payload.market_id).unwrap_or_else(|_| {
            log_error!(
                "Invalid market ID from client {client_id}: {}",
                msg_payload.market_id
            );
            return Uuid::nil();
        });
let msg = ProtoMarketMessage {
    payload: Some(Payload::Params(msg_payload)),
};

let data_to_send = msg.encode_to_vec();


        let clients = clients.get_clients(&ChannelType::PriceUpdate(market_id));
     

        if let Some(clients) = clients {
            for (client_id, client_tx) in clients.iter() {
                if let Err(e) = send_message(
                    client_tx,
                    WsSendMessage::Binary(data_to_send.clone().into()),
                )
                .await
                {
                    log_error!("Failed to send message to {client_id} - {e:#?}");
                } else {
                    served_clients += 1;
                }
            }
        }
    } else {
        log_error!(
            "Failed to parse params from client {client_id}: {}",
            data.params
        );
    }

    log_info!("Served {served_clients} clients");

    served_clients
}
