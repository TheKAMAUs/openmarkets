use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RedisKey {
    Market(Uuid),
    User(Uuid),
    Markets(u64, u64, u64),
    MpesaCheckoutId(String),           // existing CheckoutRequestID
    MpesaB2CConversation(String),      // new variant for B2C ConversationID
}

impl fmt::Display for RedisKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RedisKey::Market(uuid) => write!(f, "market:{}", uuid),
            RedisKey::User(uuid) => write!(f, "user:{}", uuid),
            RedisKey::Markets(page_no, page_size, market_status) => {
                write!(f, "markets:{}:{}:{}", page_no, page_size, market_status)
            }
            RedisKey::MpesaCheckoutId(checkout_id) => write!(f, "mpesa_checkout:{}", checkout_id),
            RedisKey::MpesaB2CConversation(conv_id) => write!(f, "mpesa_b2c:{}", conv_id),
        }
    }
}
