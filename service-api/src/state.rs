use std::error::Error as StdError;

use async_nats::{
    connect,
    jetstream::{self, Context},
};

use nkeys::KeyPair;
use async_nats::{ConnectOptions,};

use auth_service::AuthService;
use db_service::DbService;
use utility_helpers::{ log_error, log_info, redis::RedisHelper, types::EnvVarConfig};

use crate::routes::mpesa::MpesaClient;
use crate::bloom_f::BloomFilterWrapper;
use anyhow::Result;

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: sqlx::PgPool,
    pub auth_service: AuthService,
    pub jetstream: Context,
    pub order_stream: async_nats::jetstream::stream::Stream, // ✅ ADD THIS
    pub bloom_filter: BloomFilterWrapper,
    pub redis_helper: RedisHelper,
    pub mpesa_client: MpesaClient,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn StdError>> {
        dotenv::dotenv().ok();

        let env_var_config = EnvVarConfig::new()?;
     // 🔐 LOAD SECRETS
        let jwt = std::env::var("NATS_USER_JWT")?;
        let seed = std::env::var("NATS_NKEY_SEED")?;

        // optional: for local fallback
        let url = env_var_config.nc_url;

let keypair = KeyPair::from_seed(&seed)?;


        log_info!("Connecting to NATS server at {}", url);
       // 🔥 AUTHENTICATED CONNECTION
let ns = ConnectOptions::with_jwt(jwt, move |nonce| {
    let kp = keypair.clone();

    async move {
        Ok(kp.sign(&nonce).expect("valid seed"))
    }
})

.connect(url)
.await?;

        let jetstream = jetstream::new(ns);


let order_stream = jetstream
    .get_or_create_stream(jetstream::stream::Config {
        name: "ORDER".into(),
        subjects: vec!["order.>".into()],

        // required for cloud NATS
        max_bytes: 500 * 1024 * 1024,

        ..Default::default()
    })
    .await?;



        let pg_pool = sqlx::PgPool::connect(&env_var_config.database_url).await?;
        let auth_service = AuthService::new(pg_pool.clone())?;

        let bloom_filter = BloomFilterWrapper::new(&pg_pool).await?;
        let redis_helper = RedisHelper::new(
            &env_var_config.redis_url,
            60 * 60, // default cache expiration 60 sec * 60 sec = 1 hour
        )
        .await?;
        // ---- Initialize M-Pesa Client ----
        let mpesa_client = MpesaClient::new();

    let state = AppState {
    pg_pool,
    auth_service,
    jetstream,
    order_stream, // ✅ ADD THIS
    bloom_filter,
    redis_helper,
    mpesa_client,
};
        Ok(state)
    }

   pub async fn run_migrations(&self) -> Result<(), Box<dyn StdError>> {
    log_info!("🚀 Starting database migrations from DbService...");

    match DbService::run_migrations(&self.pg_pool).await {
        Ok(_) => {
            log_info!("✅ Database migrations completed successfully (DbService).");
            Ok(())
        }
        Err(e) => {
            log_error!("❌ Migration failed inside DbService: {}", e);
            Err(Box::from(format!("Migration failed: {}", e)))
        }
    }
}

}