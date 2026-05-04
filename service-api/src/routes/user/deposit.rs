use crate::state::AppState;
use auth_service::types::SessionTokenClaims;
use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    body::Body,
    response::{IntoResponse, Response},
};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utility_helpers::{log_error, log_info};
use rust_decimal::prelude::Zero;

use std::env;
use base64;

use base64::{Engine, engine::general_purpose};
use utility_helpers::redis::keys::RedisKey;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct StkPushRequest<'a> {
    BusinessShortCode: &'a str,
    Password: String,
    Timestamp: String,
    TransactionType: &'a str,
    Amount: String,
    PartyA: &'a str,
    PartyB: &'a str,
    PhoneNumber: &'a str,
    CallBackURL: &'a str,
    AccountReference: &'a str,
    TransactionDesc: &'a str,
}

#[derive(Deserialize)]
pub struct StkPushResponse {
    pub MerchantRequestID: Option<String>,
    pub CheckoutRequestID: Option<String>,
    pub ResponseCode: Option<String>,
    pub ResponseDescription: Option<String>,
    pub CustomerMessage: Option<String>,
}



#[derive(Deserialize, Serialize, Debug)]
pub struct DepositRequest {
    pub amount: Decimal,
    pub phone_number: String,
    pub account_reference: String,
}


// pub async fn deposit_funds_handler(
//     State(state): State<AppState>,
//     Extension(claims): Extension<SessionTokenClaims>,
//     Json(DepositRequest { amount }): Json<DepositRequest>,
// ) -> Result<Response, (StatusCode, Response)> {
//     let user_id = claims.user_id;

//     let amount_decimal = Decimal::from_f64(amount)// Fixed
// .ok_or_else(|| {
//     (
//         StatusCode::BAD_REQUEST,
//         Json(json!({"error": "Invalid amount provided"})).into_response(),
//     )
// })?;

//     // just deposit balance for now

//     Ok((
//         StatusCode::OK,
//         Json(json!({
//             "message": "Deposit successful",
//             "user_id": user_id,
//             "amount": amount_decimal.to_string()
//         })),
//     )
//         .into_response())
// }






pub async fn deposit_funds_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Json(req): Json<DepositRequest>,
) -> Result<Response, (StatusCode, Response)> {
    let user_id = claims.user_id;

    log_info!("💰 Deposit request received: user_id={}, amount={}, phone={}", 
        user_id, req.amount, req.phone_number);

 // ---- Validate amount ----
// ---- Validate amount ----
let amount = req.amount;

if amount <= Decimal::zero() {
    log_error!("❌ Invalid amount (<= 0): {:?}", amount);
    return Err((
        StatusCode::BAD_REQUEST,
        Json(json!({"error": "Invalid amount provided"})).into_response(),
    ));
}



// ---- Format M-Pesa Timestamp ----
let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

// ---- Read short code and passkey from .env ----
let short_code = env::var("MPESA_SHORT_CODE")
    .expect("MPESA_SHORT_CODE must be set in .env");
let passkey = env::var("MPESA_PASSKEY")
    .expect("MPESA_PASSKEY must be set in .env");
let callback_url = env::var("MPESA_CALLBACK_URL")
    .expect("MPESA_CALLBACK_URL must be set in .env");
// ---- Create STK Password ----
let password_raw = format!("{}{}{}", short_code, passkey, timestamp);
let password = general_purpose::STANDARD.encode(password_raw);

log_info!("STK Callback being sent to Safaricom: {}", callback_url);


    // ---- Fetch Access Token from MpesaClient ----
    let access_token = match state.mpesa_client.get_access_token().await {
        Ok(token) => {
            log_info!("✅ Successfully fetched M-Pesa access token");
            token
        },
        Err(err) => {
            log_error!("❌ Failed to get access token: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to get access token: {}", err)}))
                    .into_response(),
            ));
        }
    };



    // ---- Build STK Push Request ----
    let stk_request = StkPushRequest {
        BusinessShortCode: &short_code,
        Password: password,
        Timestamp: timestamp.clone(),
        TransactionType: "CustomerPayBillOnline",
        Amount: amount.to_string(),
        PartyA: &req.phone_number,
        PartyB: &short_code,
        PhoneNumber: &req.phone_number,
        CallBackURL: &callback_url,
         AccountReference: "Test",  
        TransactionDesc: "Mpesa Daraja STK Push using Rust",
    };

  log_info!("📤 Sending STK Push request for user_id={}", user_id);


    // ---- Send Request ----
    let client = reqwest::Client::new();
    let mpesa_res = client
        .post("https://sandbox.safaricom.co.ke/mpesa/stkpush/v1/processrequest")
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&stk_request)
        .send()
        .await;

    let response_text = match mpesa_res {
        Ok(raw) => raw.text().await.unwrap_or_else(|_| {
            log_error!("❌ Failed to read M-Pesa response body");
            "Failed to read response".to_string()
        }),
        Err(err) => {
            log_error!("❌ STK Push failed: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("STK Push failed: {}", err)})).into_response(),
            ));
        }
    };
log_info!("📡 Raw M-Pesa STK Push response: {}", response_text);

   // ---- Parse JSON from M-Pesa response ----
let parsed_json: serde_json::Value = match serde_json::from_str(&response_text) {
    Ok(v) => v,
    Err(_) => {
        log_error!("❌ Failed to parse M-Pesa JSON response, returning raw text");
        json!({ "raw": response_text })
    }
};

// Assuming `checkout_id` is extracted from M-Pesa response
if let Some(checkout_id) = parsed_json.get("CheckoutRequestID").and_then(|v| v.as_str()) {
    let redis_key = RedisKey::MpesaCheckoutId(checkout_id.to_string());

    // Store CheckoutRequestID → user_id mapping in Redis
    match state.redis_helper
        .get_or_set_cache(redis_key, || async {
            Ok(user_id.to_string()) // the value we want to store
        }, Some(600)) // 10 minutes expiry
        .await
    {
        Ok(stored_user_id) => {
            log_info!("✅ CheckoutRequestID stored in Redis for user_id={}", stored_user_id);
        }
        Err(e) => {
            log_error!("❌ Failed to store CheckoutRequestID in Redis: {}", e);
        }
    }
} else {
    log_error!("❌ CheckoutRequestID not found in M-Pesa response");
}


    log_info!("✅ STK Push request sent successfully for user_id={}", user_id);

    // ---- Final API Response ----
    Ok((
        StatusCode::OK,
        Json(json!({
            "status": true,
            "message": "Request sent. Enter your M-Pesa PIN.",
            "user_id": user_id,
            "amount": amount.to_string(),
            "mpesa_response": parsed_json
        })),
    )
        .into_response())
}

