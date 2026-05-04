use std::sync::Arc;

use db_service::schema::{enums::OrderStatus,enums::OrderSide, enums::Outcome, enums::OrderType,
    orders::Order};
use rust_decimal::Decimal;
use utility_helpers::{
    log_error, log_info,};
use uuid::Uuid;
use rust_decimal::MathematicalOps;
use rust_decimal_macros::dec;

use crate::{
    state::AppState,
    utils::{
        OrderServiceError, update_matched_orders::update_lmsr_orders,
        update_services::update_service_state,
    },
};

pub async fn create_order_handler(
    app_state: Arc<AppState>,
    order_id: Uuid,
    market_order_budget_opt: Option<Decimal>,
) -> Result<(), OrderServiceError> {
    log_info!("Processing order creation for order_id: {}", order_id);
    
    let order = Order::find_order_by_id_with_market(order_id, &app_state.db_pool)
        .await
        .map_err(|e| {
            log_error!("Failed to find order {}: {:#?}", order_id, e);
            format!("Failed to find order {:#?}", e)
        })?;

    log_info!("Successfully retrieved order from database: {:?}", order);

    // open orders are already added to order book during initialization
    if order.status == OrderStatus::OPEN {
        log_info!("Order {} already open, skipping processing", order_id);
        return Ok(());
    }

    log_info!("Processing order {} with status {:?}", order_id, order.status);

    // working on unspecified status order
    let (matched_order, updated_raw_order) = {
        // sync block
        {
            let mut order_raw = Order {
                id: order.id,
                status: OrderStatus::OPEN,
                created_at: order.created_at,
                filled_quantity: order.filled_quantity,
                market_id: order.market_id,
                outcome: order.outcome,
                price: order.price,
                quantity: order.quantity,
                side: order.side,
                updated_at: order.updated_at,
                user_id: order.user_id,
                order_type: order.order_type,
            };

            log_info!("Acquiring order book write lock for market: {}", order_raw.market_id);
            let mut order_book = app_state.order_book.write();
            log_info!("Order book write lock acquired for market: {}", order_raw.market_id);
            
            let matches = if let Some(market_order_budget) = market_order_budget_opt {
                log_info!("Processing as market order with budget: {}", market_order_budget);
              order_book.create_market_order(
        order.market_id, 
        &mut order_raw, 
        market_order_budget,
        order.liquidity_b,
        order.q_yes,  // Pass q_yes from the order
        order.q_no,   // Pass q_no from the order
    )
           } else {
log_info!("Processing order type: {:?}", order.order_type);

// Compute current market price from LMSR state
let b = order.liquidity_b;
let exp_yes = (order.q_yes / b).exp();
let exp_no  = (order.q_no  / b).exp();
let total   = exp_yes + exp_no;
let current_price = match order.outcome {
    Outcome::YES => exp_yes / total,
    Outcome::NO  => exp_no  / total,
    _            => dec!(0.5),
};

let price_condition_met = match order.order_type {
    OrderType::LIMIT => {
        match order.side {
            OrderSide::BUY  => current_price <= order.price,  // Buy when price drops to limit
            OrderSide::SELL => current_price >= order.price,  // Sell when price rises to limit
        }
    },
    OrderType::StopLoss => {
        match order.side {
            OrderSide::BUY  => current_price >= order.price,  // Buy when price rises to stop (breakout)
            OrderSide::SELL => current_price <= order.price,  // Sell when price drops to stop (stop loss)
        }
    },
    _ => false,
};

if !price_condition_met {
    log_info!(
        "{:?} order {} parked — {:?} {:?} | price: {}, current: {}",
        order.order_type, order.id, order.side, order.outcome, order.price, current_price
    );

    // Set OPEN so trigger_eligible_limit_orders picks it up on next price move
    order_raw.status = OrderStatus::OPEN;
    vec![]  // no matches — return empty so post-processing is a no-op
} else {
    log_info!(
        "{:?} order {} condition met — {:?} {:?} | price: {}, current: {}",
        order.order_type, order.id, order.side, order.outcome, order.price, current_price
    );
    order_book.process_order(&mut order_raw, order.liquidity_b, order.q_yes, order.q_no)
}
};
            log_info!("Order processing complete, found {} matches", matches.len());

            // updating current order filled quantity and status
            (matches, order_raw)
        }
    };
    
    log_info!("Updating order {} in database", updated_raw_order.id);
    updated_raw_order
        .update(&app_state.db_pool)
        .await
        .map_err(|e| {
            log_error!("Failed to update order {}: {:#?}", updated_raw_order.id, e);
            format!("Failed to update order: {:#?}", e)
        })?;
    
    log_info!("Order {} successfully updated in database", updated_raw_order.id);

    log_info!("Starting post-order processing tasks for order {}", updated_raw_order.id);
    
    // CHANGE: Replace update_matched_orders with update_lmsr_orders
    let update_matched_order_future =
        update_lmsr_orders(matched_order, app_state.clone(), &updated_raw_order);

    let update_service_state_future = update_service_state(app_state.clone(), &updated_raw_order);

    let (update_matched_orders_result, update_service_state_result) =
        tokio::join!(update_matched_order_future, update_service_state_future);

    if let Err(e) = &update_matched_orders_result {
        log_error!("Error while updating post order for {}: {e:#?}", updated_raw_order.id);
    }
    update_matched_orders_result.map_err(|e| format!("Error while updating post order {e:#?}"))?;

    if let Err(e) = &update_service_state_result {
        log_error!("Error while updating service states for {}: {e:#?}", updated_raw_order.id);
    }
    update_service_state_result
        .map_err(|e| format!("Error while updating service states {e:#?}"))?;

    log_info!("Order {} processing completed successfully", updated_raw_order.id);
    Ok(())
}


#[cfg(test)]
mod test {
    use std::{str::FromStr, time::Duration};

    use futures_util::SinkExt;
    use prost::Message;
    use proto_defs::proto_types::ws_common_types::{
        Channel, OperationType, Payload, WsData, WsMessage,
    };
    use rdkafka::{
        ClientConfig,
        producer::{FutureProducer, FutureRecord},
    };
    use rust_decimal_macros::dec;
    use serde_json::json;
    use tokio_tungstenite::{
        connect_async,
        tungstenite::{Message as WsMessageType, client::IntoClientRequest},
    };
    use utility_helpers::{
    log_error, log_info,};
    use uuid::Uuid;

    #[tokio::test]
    #[ignore = "just ignore"]
    async fn test_kafka_publishing() {
        let rd_kafka: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .set("message.timeout.ms", "10000")
            .create()
            .expect("Failed to create Kafka client");

        let record = FutureRecord::to("price-updates-test")
            .payload("test message 1")
            .key("test_key_1");

        log_info!("Record {record:?}");

        let res = rd_kafka.send(record, Duration::from_secs(0)).await;
        assert!(
            res.is_ok(),
            "Failed to send record to Kafka: {:?}",
            res.err()
        );
    }

    #[tokio::test]
    #[ignore = "just ignore"]
    async fn test_publish_data_to_clickhouse_client() {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:19092")
            .set("message.timeout.ms", "10000")
            .create()
            .expect("Failed to create Kafka client");

        let market_id = Uuid::new_v4().to_string();
        let ts = chrono::Utc::now().to_rfc3339();

        let msg = json!({
            "market_id": market_id,
            "yes_price": 0.4,
            "no_price": 0.6,
            "ts": ts,
        })
        .to_string();

        for _i in 0..10 {
            let record: FutureRecord<'_, String, String> =
                FutureRecord::to("price-updates").payload(&msg);
            let res = producer.send(record, Duration::from_secs(0)).await;
            assert!(
                res.is_ok(),
                "Failed to send record to Kafka: {:?}",
                res.err()
            );
        }
    }

    #[tokio::test]
    #[ignore = "just ignore"]
    async fn test_websocket_message() {
        let websocket_req = format!("ws://localhost:4010/ws")
            .into_client_request()
            .expect("Failed to create WebSocket request");
        let (mut stream, _) = connect_async(websocket_req)
            .await
            .expect("Failed to connect to WebSocket server");

        let real_market_id = Uuid::from_str("67df943a-09a5-4ddb-adeb-11042c37c324")
            .unwrap()
            .to_string();

        let market_data = serde_json::json!({
            "market_id": real_market_id,
            "yes_price": dec!(0.4).to_string(),
            "no_price": dec!(0.6).to_string(),
        })
        .to_string();

        let message = WsMessage {
            id: None,
            payload: Some(Payload {
                ops: OperationType::Post.into(),
                data: Some(WsData {
                    channel: Channel::Priceposter.into(),
                    params: market_data,
                }),
            }),
        };

        let bin_data = message.encode_to_vec();

        if let Err(e) = stream.send(WsMessageType::Binary(bin_data.into())).await {
            log_error!("Failed to send message to WebSocket: {:#?}", e);
        }
    }
}
