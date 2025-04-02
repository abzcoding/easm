use api::run;
use shared::config::Config;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from environment variables
    let config = Config::from_env()?;

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    info!("Starting API server on {}:{}", config.host, config.port);

    // Run the server
    run(config).await?;

    Ok(())
}
