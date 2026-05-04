use async_nats::{connect, jetstream};
use tokio::sync::RwLock;
use utility_helpers::{log_info, types::EnvVarConfig};
use nkeys::KeyPair;
use async_nats::{ConnectOptions,};


use crate::core::client_manager::SubscriptionAndClientManager;

#[derive(Debug)]
pub struct WebSocketAppState {
    pub client_manager: RwLock<SubscriptionAndClientManager>,
    pub jetstream: jetstream::Context,
}

impl WebSocketAppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        log_info!("Initializing WebSocketAppState...");

        dotenv::dotenv().ok();
        log_info!("Loaded .env file");

        let env_var_config = EnvVarConfig::new()?;
        log_info!("Environment variables loaded: nc_url = {}", env_var_config.nc_url);

     // 🔐 LOAD SECRETS
        let jwt = std::env::var("NATS_USER_JWT")?;
        let seed = std::env::var("NATS_NKEY_SEED")?;

        // optional: for local fallback
        let url = env_var_config.nc_url;

let keypair = KeyPair::from_seed(&seed)?;


        log_info!("Connecting to NATS server at {}", url);
       // 🔥 AUTHENTICATED CONNECTION
let nc = ConnectOptions::with_jwt(jwt, move |nonce| {
    let kp = keypair.clone();

    async move {
        Ok(kp.sign(&nonce).expect("valid seed"))
    }
})
.connect(url)
.await?;

        let jetstream = jetstream::new(nc);
        log_info!("JetStream client initialized");

        let client_manager = RwLock::new(SubscriptionAndClientManager::new());
        log_info!("Client manager initialized");

        log_info!("WebSocketAppState initialization complete");
        Ok(WebSocketAppState {
            jetstream,
            client_manager,
        })
    }
}
