pub mod migrations;

use shared::errors::{Error, Result};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

/// Database connection pool singleton
#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self> {
        info!("Connecting to database: {}", database_url);

        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await
            .map_err(|e| Error::database(format!("Failed to connect to database: {}", e)))?;

        info!("Connected to database");

        Ok(Self { pool })
    }

    /// Check database connection
    pub async fn check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database(format!("Database check failed: {}", e)))?;

        Ok(())
    }
}
