use axum::{
    Json, Router,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use std::sync::Arc;

use crate::state::AppState;
use crate::utils::middleware as custom_middleware;
use crate::routes::user::orders::create_market_order::fetch_and_validate_market;
use crate::routes::user::trades::btc_markets::get_btc_markets_handler;

pub mod mpesa; // lowercase, matches the filename
pub mod admin;
pub mod login;
pub mod user;
pub mod sugg;



async fn default_home_route() -> (StatusCode, impl IntoResponse) {
    let welcome_message = json!({
        "message": "Welcome to the Polymarket clone service API!"
    });
    (StatusCode::OK, Json(welcome_message))
}

pub fn router(app_state: AppState) -> Router<AppState> {
    let app_state = Arc::new(app_state.clone());
     let user_routes = user::router().layer(middleware::from_fn_with_state(
        app_state.clone(), // clone here so original Arc stays usable
        custom_middleware::validate_jwt,
    ));

    // let admin_routes = admin::router(); // for now without middleware


let admin_routes = admin::router()
    .layer(axum::middleware::from_fn(custom_middleware::admin_only))        // admin guard
    .layer(axum::middleware::from_fn_with_state(
        app_state.clone(),
       custom_middleware::validate_jwt,                                   // validates JWT
    ));




Router::new()
    .route("/", get(default_home_route))
    .route("/login", post(login::oauth_login))
    .route("/mpesa/callback", post(mpesa::mpesa_callback_handler)) // <-- callback endpoint
    .nest("/user", user_routes)
    .nest("/admin", admin_routes)
    //   .route("/mpesa/b2c", post(mpesa::b2c_result_callback)) // <-- callback endpoint
.route("/suggestions/public", get(sugg::get_public_suggestions_handler))
.route("/quote", post(fetch_and_validate_market))
.route("/btcusdt", get(get_btc_markets_handler))
}