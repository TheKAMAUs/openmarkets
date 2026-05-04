use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::market::Market;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde_json::json;
use sqlx::types::chrono::{self, DateTime};
use utility_helpers::log_error;
use uuid::Uuid;
use crate::{require_fields_raw_response, state::AppState};

#[derive(serde::Deserialize)]
pub struct CreateMarketRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub logo: Option<Vec<String>>,  // Changed to Vec<String> for multiple logos
    pub liquidity_b: Option<f64>,
    pub market_expiry: Option<String>,
    pub slug: Option<String>,
    pub is_event: Option<bool>,
    pub child_markets: Option<Vec<CreateChildMarketRequest>>,
    pub category: Option<String>,
    pub resolution_criteria: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CreateChildMarketRequest {
    pub name: Option<String>,
    pub question: Option<String>,        // used as description
    pub logo: Option<Vec<String>>,       // Changed to Vec<String> for multiple logos
    pub liquidity: Option<f64>,
    pub market_expiry: Option<String>,
    pub slug: Option<String>,            // optional
    pub category: Option<String>,        // optional
    pub resolution_criteria: Option<String>, // optional
}

// Add market expiry in db


pub async fn create_new_market(
    State(state): State<AppState>,
    Json(payload): Json<CreateMarketRequest>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // Validate required parent fields
    require_fields_raw_response!(payload.name);
    require_fields_raw_response!(payload.description);
    require_fields_raw_response!(payload.logo);
    require_fields_raw_response!(payload.liquidity_b);
    require_fields_raw_response!(payload.market_expiry);

    let liquidity_b = Decimal::from_f64(payload.liquidity_b.unwrap()).ok_or_else(|| {
        log_error!("Invalid liquidity_b value: {}", payload.liquidity_b.unwrap());
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid liquidity_b value" })),
        )
    })?;

  // Parse market expiry
let market_expiry_str = payload.market_expiry.clone().unwrap(); // Clone to keep the original
let rfc3339_str = format!("{}:00Z", market_expiry_str);
let date_time = DateTime::parse_from_rfc3339(&rfc3339_str).map_err(|e| {
    log_error!("Invalid market_expiry format: {} due to {e}", market_expiry_str);
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": "Invalid market_expiry format" })),
    )
})?;

    if date_time < chrono::Utc::now() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Market expiry date cannot be in the past" })),
        ));
    }

    let market_expiry = date_time.naive_utc();
    let is_event = payload.is_event.unwrap_or(false);

    // Create parent market
let parent_market = Market::create_new_market(
    payload.name.unwrap(),                    // 1. name: String
    payload.description.unwrap(),              // 2. description: String
    payload.logo.unwrap(),                     // 3. logo: String
    liquidity_b,                                // 4. liquidity_b: Decimal
    market_expiry,                              // 5. market_expiry: NaiveDateTime
    &state.pg_pool,                             // 6. pg_pool: &PgPool
    None,                                       // 7. parent_id: Option<Uuid> (None for parent)
    is_event,                                   // 8. is_event: bool (true for parent)
    None,                                       // 9. child_market_ids: Option<Vec<Uuid>>
    payload.category.clone(),                   // 10. category: Option<String>
    payload.resolution_criteria.clone(),        // 11. resolution_criteria: Option<String>
    payload.slug.clone(),                       // 12. slug: Option<String>
).await
    .map_err(|e| {
        log_error!("Error creating market: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to create market" })),
        )
    })?;

    // Create child markets if any
    let mut created_children = vec![];
    if let Some(children) = payload.child_markets {
        for child in children {
            if child.name.is_none() || child.question.is_none() {
                continue; // skip invalid child
            }

            let child_liquidity = Decimal::from_f64(child.liquidity.unwrap_or(0.0))
                .unwrap_or(Decimal::ZERO);


    let date_time = DateTime::parse_from_rfc3339(&rfc3339_str).map_err(|e| {
        log_error!("Invalid market_expiry format: {} due to {e}", child.market_expiry.unwrap());
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid market_expiry format" })),
        )
    })?;

    if date_time < chrono::Utc::now() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Market expiry date cannot be in the past" })),
        ));
    }

    let child_market_expiry = date_time.naive_utc();


let child_market = Market::create_new_market(
    child.name.unwrap(),                        // 1. name: String
    child.question.unwrap(),                    // 2. description: String
    child.logo.unwrap_or_default(),             // 3. logo: String
    child_liquidity,                             // 4. liquidity_b: Decimal
    child_market_expiry,                         // 5. market_expiry: NaiveDateTime
    &state.pg_pool,                               // 6. pg_pool: &PgPool
    Some(parent_market.id),                       // 7. parent_id: Option<Uuid>
    false,                                        // 8. is_event: bool (false for child)
    None,                                         // 9. child_market_ids: Option<Vec<Uuid>>
    child.category.clone(),                       // 10. category: Option<String>
    child.resolution_criteria.clone(),            // 11. resolution_criteria: Option<String>
    child.slug.clone(),                           // 12. slug: Option<String>
).await
            .map_err(|e| {
                log_error!("Failed to create child market: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to create child market" })),
                )
            })?;

            created_children.push(child_market);
        }
    }
    // Store child_market_ids in parent row
    if !created_children.is_empty() {
        let child_ids: Vec<Uuid> = created_children.iter().map(|c| c.id).collect();
        Market::update_child_market_ids_field(&parent_market.id, &child_ids, &state.pg_pool)
            .await
            .ok(); // ignore errors for now
    }


    // Build response
    let response = json!({
        "message": "Market created successfully",
        "market": {
            "id": parent_market.id,
            "name": parent_market.name,
            "description": parent_market.description,
            "logo": parent_market.logo,
            "liquidity_b": parent_market.liquidity_b,
            "slug": payload.slug,
            "is_event": is_event,
            "category": payload.category,
            "resolution_criteria": payload.resolution_criteria,
            "child_markets": created_children.iter().map(|c| {
                json!({
                    "id": c.id,
                    "name": c.name,
                    "description": c.description,
                    "logo": c.logo,
                    "liquidity": c.liquidity_b,
                    "slug": c.slug,
                    "category": c.category,
                    "resolution_criteria": c.resolution_criteria,
              
                })
            }).collect::<Vec<_>>()
        }
    });

    Ok((StatusCode::CREATED, Json(response)).into_response())
}