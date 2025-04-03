use anyhow::Result;
use infrastructure::database::Database;
use shared::config::Config;
use std::time::Duration;
use tokio::time::sleep;

mod job_processor;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();

    tracing::info!("Starting tasks worker...");

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded successfully.");

    // Initialize database pool
    let db = Database::new(&config.database_url, 5).await?;
    tracing::info!("Database pool initialized.");

    // Main worker loop
    loop {
        tracing::debug!("Checking for pending jobs...");
        match job_processor::process_pending_jobs(&db.pool).await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("Processed {} jobs.", count);
                } else {
                    tracing::debug!("No pending jobs found.");
                }
            }
            Err(e) => {
                tracing::error!("Error processing jobs: {}", e);
            }
        }

        // Sleep for a configurable interval before checking again
        // TODO: Make interval configurable
        sleep(Duration::from_secs(30)).await;
    }
}
