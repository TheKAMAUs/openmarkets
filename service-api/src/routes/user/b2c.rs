use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,Extension
};
use auth_service::types::SessionTokenClaims;

use serde::{Deserialize, Serialize};
use serde_json::json;
use utility_helpers::{ log_error, log_info,};
use utility_helpers::redis::keys::RedisKey;
use rust_decimal::Decimal;

use rust_decimal::prelude::Zero;

use crate::state::AppState;


#[derive(Debug, Deserialize)]
pub struct B2CRequest {
    pub phoneNumber: String,
    pub amount:Decimal,
}


#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub status: bool,
    pub message: String,
    pub data: Option<T>,
}

pub async fn b2c_withdraw_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Json(payload): Json<B2CRequest>,
) -> impl IntoResponse {
    log_info!("🔥 Incoming B2C Withdrawal Request: {:?}", payload);

 if payload.phoneNumber.is_empty() || payload.amount <= Decimal::zero() {
    log_error!("❌ Missing phoneNumber or amount");
    return (
        StatusCode::BAD_REQUEST,
        Json(ApiResponse::<serde_json::Value> {
            status: false,
            message: "Phone number and amount are required".to_string(),
            data: None,
        }),
    );
}


    log_info!("📌 Phone: {}, Amount: {}", payload.phoneNumber, payload.amount);
    log_info!("🔑 Fetching Access Token...");

    let access_token = match state.mpesa_client.get_access_token().await {
        Ok(token) => {
            log_info!("✅ Access Token Received");
            token
        }
        Err(e) => {
            log_error!("❌ ACCESS TOKEN FAILURE: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<serde_json::Value> {
                    status: false,
                    message: "Failed to get access token".to_string(),
                    data: None,
                }),
            );
        }
    };

    // =====================================
    //       B2C CONFIG
    // =====================================
    let security_credential = "RC6E9WDxXR4b9X2c6z3gp0oC5Th==";
    let url = "https://sandbox.safaricom.co.ke/mpesa/b2c/v1/paymentrequest";

    log_info!("📤 Sending B2C Payment Request to Safaricom...");
    log_info!("➡️ URL: {}", url);

    let client = reqwest::Client::new();

    let request_payload = json!({
        "OriginatorConversationID": "600997_Test_32et3241ed8yu",
        "InitiatorName": "testapi",
        "SecurityCredential": security_credential,
        "CommandID": "BusinessPayment",
        "Amount": payload.amount,
        "PartyA": "600992",
        "PartyB": payload.phoneNumber,
        "Remarks": "remarked",
        "QueueTimeOutURL": "https://yourdomain.com/b2c/queue",
        "ResultURL": "https://b884-197-248-221-165.ngrok-free.app/mpesa/b2c",
        "Occasion": "ChristmasPay"
    });

    match client
        .post(url)
        .bearer_auth(access_token)
        .json(&request_payload)
        .send()
        .await
    {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(data) => {
                log_info!("✅ B2C SUCCESS RESPONSE: {:?}", data);

                // Extract ConversationID from Safaricom response
                if let Some(conv_id) = data.get("ConversationID").and_then(|v| v.as_str()) {
                    let user_id = claims.user_id.to_string(); // store the authenticated user_id
                    let redis_key = RedisKey::MpesaB2CConversation(conv_id.to_string());

                    // Save user_id in Redis keyed by ConversationID
                    if let Err(e) = state
                        .redis_helper
                        .get_or_set_cache(redis_key, || async { Ok(user_id.clone()) }, Some(600))
                        .await
                    {
                        log_error!("❌ Failed to store ConversationID in Redis: {}", e);
                    } else {
                        log_info!("💾 Stored ConversationID {} for user_id {} in Redis", conv_id, user_id);
                    }
                }

                (
                    StatusCode::OK,
                    Json(ApiResponse {
                        status: true,
                        message: "Withdrawal request sent successfully".to_string(),
                        data: Some(data),
                    }),
                )
            }
            Err(e) => {
                log_error!("❌ Failed parsing Safaricom response: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<serde_json::Value> {
                        status: false,
                        message: "Failed to parse Safaricom response".to_string(),
                        data: None,
                    }),
                )
            }
        },
        Err(e) => {
            log_error!("❌ B2C SAFARICOM ERROR: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<serde_json::Value> {
                    status: false,
                    message: "B2C Request failed".to_string(),
                    data: None,
                }),
            )
        }
    }
}
