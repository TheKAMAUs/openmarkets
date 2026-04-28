use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use state::AppState;
use utility_helpers::log_info;
use tracing_subscriber::EnvFilter;

use db_service::schema::users::User;
use db_service::schema::market::Market;

use rust_decimal::Decimal;
use sqlx::types::chrono::NaiveDate;
use chrono::Utc;
use chrono::Duration;

mod routes;
mod state;
mod utils;

pub mod bloom_f;

const PORT: u16 = 8080;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
     tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

    let addr = format!("[::]:{}", PORT);

    let state = AppState::new().await?;
    // state.run_migrations().await?;





 let demo_market = Market::create_new_market(
    "Demo Market".into(),                        // 1. name: String
    "Automatically created market".into(),       // 2. description: String
    vec!["https://example.com/logo.png".to_string()],
    Decimal::from(1000),                          // 4. liquidity_b: Decimal
    Utc::now().naive_utc() + Duration::days(30),  // 5. market_expiry: NaiveDateTime
    &state.pg_pool,                                // 6. pg_pool: &PgPool
    None,                                          // 7. parent_id: Option<Uuid> (None for parent)
    false,                                         // 8. is_event: bool (true for parent)
    None,                                          // 9. child_market_ids: Option<Vec<Uuid>>
    Some("Demo".into()),                           // 10. category: Option<String>
    Some("Manual resolution".into()),              // 11. resolution_criteria: Option<String>
    Some("demo-market".into()),                    // 12. slug: Option<String>
).await?;

    log_info!("Startup market created: {}", demo_market.id);



    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    let app = Router::new()
        .merge(routes::router(state.clone()))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    log_info!("service-api is listening on http://localhost:{}", PORT);

    axum::serve(listener, app).await.unwrap();
    Ok(())
}
