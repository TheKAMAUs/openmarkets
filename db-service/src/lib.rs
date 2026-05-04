use std::error::Error as StdError;

pub mod pagination;
pub mod procedures;
pub mod schema;
pub mod utils;

pub struct DbService;
use utility_helpers::{ log_error, log_info,};


impl DbService {
    pub async fn run_migrations(pg_pool: &sqlx::PgPool) -> Result<(), Box<dyn StdError>> {
        log_info!("🚀 Starting database migrations...");

        // The migrate macro loads migration metadata at compile-time.
        let migrator = sqlx::migrate!("./migrations");

        // Print all migration files BEFORE running
        for m in migrator.iter() {
            log_info!("📄 Pending migration: {}", m.description);
        }

        match migrator.run(pg_pool).await {
            Ok(_) => {
                log_info!("✅ Database migrations completed successfully.");
                Ok(())
            }
            Err(e) => {
                log_error!("❌ Migration failed: {}", e);
                Err(Box::from(format!("Migration failed: {}", e)))
            }
        }
    }
}
