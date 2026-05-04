use axum::{
    Router,
    extract::{State, ws::WebSocketUpgrade},
    routing::any,
};
use std::sync::Arc;
use tracing_subscriber;
use utility_helpers::{log_error, log_info};

use crate::{
    core::handle_connection::handle_connection, nats_handler::nats_handler,
    state::WebSocketAppState,
};

mod core;
mod nats_handler;
mod state;

pub type SafeAppState = Arc<WebSocketAppState>;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    log_info!("Initializing WebSocket service...");

    //
    // 1. Determine runtime port (Render -> PORT | Local -> 4010)
    //
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(4010);

    let addr = format!("0.0.0.0:{}", port);
    log_info!("WebSocket server will bind on {}", addr);


   //
    // 5. Bind TCP listener + serve
    //
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    log_info!("WebSocket server is now listening on {}", addr);


    //
    // 2. Initialize shared application state
    //
    let app_state = Arc::new(WebSocketAppState::new().await?);
    let nats_state = app_state.clone();

    //
    // 3. Build Axum router
    //
    let app = Router::new()
        .route("/", any(|| async { "Hello from WebSocket server!" }))
        .route("/ws", any(socket_handler))
        .with_state(app_state);

    //
    // 4. Spawn background NATS handler
    //
    tokio::spawn(async move {
        if let Err(e) = nats_handler(nats_state).await {
            log_error!("NATS handler crashed: {}", e);
            panic!("NATS handler failure");
        }
    });

 

    axum::serve(listener, app).await?;
    Ok(())
}

async fn socket_handler(
    ws: WebSocketUpgrade,
    State(state): State<SafeAppState>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_connection(socket, state))
}