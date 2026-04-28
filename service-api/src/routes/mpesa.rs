use std::env;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use axum::{Json, extract::State, response::IntoResponse,  http::StatusCode  };
use serde_json::json;

use serde_json::Value;
use utility_helpers::{ log_error, log_info,};
use base64::{engine::general_purpose, Engine};
use utility_helpers::redis::keys::RedisKey;
use uuid::Uuid;
use db_service::schema::users::User;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

use crate::state::AppState;



#[derive(Debug, Deserialize)]
 struct MpesaAccessTokenResponse {
    pub access_token: String,
    pub expires_in: String,
}

#[derive(Clone)]
pub struct MpesaClient {
    client: Client,
    consumer_key: String,
    consumer_secret: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct B2CResultCallback {
    pub Result: B2CResult,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct B2CResult {
    pub ResultType: i32,
    pub ResultCode: i32,
    pub ResultDesc: String,
    pub OriginatorConversationID: String,
    pub ConversationID: String,
    pub TransactionID: String,
    pub ResultParameters: Option<ResultParams>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResultParams {
    pub ResultParameter: Vec<ResultParameterItem>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResultParameterItem {
    pub Key: String,
    pub Value: serde_json::Value,
}







impl MpesaClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            consumer_key: env::var("MPESA_CONSUMER_KEY")
                .expect("MPESA_CONSUMER_KEY must be set"),
            consumer_secret: env::var("MPESA_CONSUMER_SECRET")
                .expect("MPESA_CONSUMER_SECRET must be set"),
        }
    }

pub async fn get_access_token(&self) -> Result<String, anyhow::Error> {
    let url = "https://sandbox.safaricom.co.ke/oauth/v1/generate?grant_type=client_credentials";

    let auth = general_purpose::STANDARD.encode(format!("{}:{}", self.consumer_key, self.consumer_secret));

    let response = self.client
        .get(url)
        .header("Authorization", format!("Basic {}", auth))
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await?;

    println!("🔍 Raw Token Response [{}]: {}", status, text);

    if !status.is_success() {
        anyhow::bail!("Token request failed: {}", text);
    }

    let parsed: MpesaAccessTokenResponse = serde_json::from_str(&text)?;
    Ok(parsed.access_token)
}
}


pub async fn mpesa_callback_handler(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    log_info!("📩 Received M-Pesa callback: {:?}", payload);

    if let Some(stk_callback) = payload.get("Body").and_then(|b| b.get("stkCallback")) {
        let result_code = stk_callback
            .get("ResultCode")
            .and_then(|v| v.as_i64())
            .unwrap_or(-1);

        let checkout_request_id = stk_callback
            .get("CheckoutRequestID")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Look up user_id from Redis using checkout_request_id
        let user_id: Option<String> = if !checkout_request_id.is_empty() {
            let redis_key = RedisKey::MpesaCheckoutId(checkout_request_id.to_string());

            match state.redis_helper
                .get_or_set_cache(redis_key, || async {
                    // Fallback if not in Redis
                    Err::<String, _>("Not found".into())
                }, Some(600))
                .await
            {
                Ok(uid) => Some(uid),
                Err(e) => {
                    log_error!("❌ Failed to fetch user_id for {}: {}", checkout_request_id, e);
                    None
                }
            }
        } else {
            None
        };

        let account_reference = user_id.clone().unwrap_or_else(|| "unknown".to_string());

        if result_code == 0 {
            // Extract amount from CallbackMetadata
     let amount: Decimal = stk_callback
    .get("CallbackMetadata")
    .and_then(|meta| meta.get("Item"))
    .and_then(|items| items.as_array())
    .and_then(|arr| {
        arr.iter()
            .find_map(|i| {
                if i.get("Name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("") == "Amount"
                {
                    i.get("Value").and_then(|v| v.as_f64())
                } else {
                    None
                }
            })
    })
    .map(|f| Decimal::from_f64(f).unwrap_or(Decimal::ZERO))
    .unwrap_or(Decimal::ZERO);



 if let Some(uid_str) = user_id {
    // Convert user_id string to Uuid
    match Uuid::parse_str(&uid_str) {
        Ok(uid) => {
            // Deposit amount (already a Decimal)
            match User::deposit_funds(&state.pg_pool, uid, amount).await {
                Ok(_) => log_info!(
                    "✅ Successfully updated balance for user_id={} with amount={}",
                    uid,
                    amount
                ),
                Err(e) => log_error!(
                    "❌ Failed to update balance for user_id={}: {}",
                    uid,
                    e
                ),
            }
        }
        Err(_) => log_error!("❌ Invalid user_id UUID: {}", uid_str),
    }
} else {
    log_error!(
        "❌ Could not extract user_id from M-Pesa callback for checkout_request_id={}",
        checkout_request_id
    );
}



            log_info!("✅ Payment successful for account: {}", account_reference);
        } else {
            let desc = stk_callback
                .get("ResultDesc")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            log_error!("❌ Payment failed for {}: {}", account_reference, desc);
        }
    } else {
        log_error!("❌ Invalid callback payload: {:?}", payload);
    }

    // M-Pesa requires 200 OK
    (StatusCode::OK, "OK")
}








// pub async fn b2c_result_callback(
//     State(state): State<AppState>,
//     Json(payload): Json<B2CResultCallback>,
// ) -> Json<serde_json::Value> {
//     let result = payload.Result;

//     let mut transaction_amount = None;
//     let mut working_account = None;
//     let mut utility_account = None;
//     let mut completed_time = None;
//     let mut receiver_name = None;
//     let mut receiver_phone = None;

//     if let Some(params) = result.ResultParameters {
//         for item in params.ResultParameter {
//             match item.Key.as_str() {
//                 "TransactionAmount" => transaction_amount = Some(item.Value.clone()),
//                 "WorkingAccountAvailableFunds" => working_account = Some(item.Value.clone()),
//                 "UtilityAccountAvailableFunds" => utility_account = Some(item.Value.clone()),
//                 "TransactionCompletedDateTime" => completed_time = Some(item.Value.clone()),
//                 "ReceiverPartyPublicName" => receiver_name = Some(item.Value.clone()),
//                 "ReceiverPartyPhone" => receiver_phone = Some(item.Value.clone()),
//                 _ => {}
//             }
//         }
//     }

//     log_info!("🔥 B2C Callback Received");
//     log_info!("ResultCode: {}", result.ResultCode);
//     log_info!("ResultDesc: {}", result.ResultDesc);
//     log_info!("TransactionAmount: {:?}", transaction_amount);
//     log_info!("ReceiverPhone: {:?}", receiver_phone);

//     // ----- Get user_id from Redis using ConversationID -----
//     let user_id: Option<String> = match state
//         .redis_helper
//         .get_or_set_cache(
//             RedisKey::MpesaB2CConversation(result.ConversationID.clone()),
//             || async { Ok(None::<String>) }, // fallback if not found
//             Some(600),                       // cache expiry
//         )
//         .await
//     {
//         Ok(id_opt) => id_opt,
//         Err(e) => {
//             log_error!("❌ Failed to fetch user_id from Redis: {}", e);
//             None
//         }
//     };

//     if let Some(uid_str) = user_id {
//         log_info!("💾 Found user_id {} for ConversationID {}", uid_str, result.ConversationID);

//         // TODO: Convert uid_str to UUID or internal ID if needed
//         // let user_uuid = Uuid::parse_str(&uid_str).unwrap();

//         // ----- Store/update database with user_id -----
//         let record = json!({
//             "user_id": uid_str,
//             "resultType": result.ResultType,
//             "resultCode": result.ResultCode,
//             "resultDesc": result.ResultDesc,
//             "originatorConversationID": result.OriginatorConversationID,
//             "conversationID": result.ConversationID,
//             "transactionID": result.TransactionID,
//             "transactionAmount": transaction_amount,
//             "transactionCompletedDateTime": completed_time,
//             "receiverPartyPublicName": receiver_name,
//             "receiverPartyPhone": receiver_phone,
//             "timestamp": chrono::Utc::now().to_rfc3339(),
//         });

//         // if let Err(e) = state.db.save_b2c_result(&result.ConversationID, record).await {
//         //     tracing::error!("❌ Failed to save B2C result to DB: {}", e);
//         // }
//     } else {
//         tracing::warn!(
//             "⚠️ No user_id found in Redis for ConversationID {}",
//             result.ConversationID
//         );
//     }

//     Json(json!({
//         "status": "B2C callback received"
//     }))
// }



