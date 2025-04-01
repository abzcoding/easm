use shared::errors::{Error, Result};
use sqlx::{migrate::MigrateDatabase, Pool, Postgres};
use tracing::info;

/// The migrator manages database migrations using SQLx
pub struct Migrator {
    pool: Pool<Postgres>,
}

impl Migrator {
    /// Create a new migrator
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> Result<()> {
        info!("Running database migrations");

        // Using a fixed path relative to the workspace root
        // Make sure to run the app from the workspace root
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database(format!("Failed to connect to database: {}", e)))?;

        info!("Database connection successful, applying migrations");

        // Check if the migrations table exists
        let migration_table_exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_schema = 'public' 
                AND table_name = '_sqlx_migrations'
            ) as "exists!"
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            Error::database(format!("Failed to check if migrations table exists: {}", e))
        })?;

        // Create migrations table if it doesn't exist
        if !migration_table_exists {
            info!("Creating migrations table");
            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS _sqlx_migrations (
                    version BIGINT PRIMARY KEY,
                    description TEXT NOT NULL,
                    installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
                    success BOOLEAN NOT NULL,
                    checksum BYTEA NOT NULL,
                    execution_time BIGINT NOT NULL
                );
                "#,
            )
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database(format!("Failed to create migrations table: {}", e)))?;
        }

        // Check if the migration has already been applied
        let migration_applied = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT FROM _sqlx_migrations 
                WHERE version = $1
            ) as "exists!"
            "#,
            20250331180000_i64
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database(format!("Failed to check migration status: {}", e)))?;

        // If the migration is already applied, we're done
        if migration_applied {
            info!("Migration already applied, skipping");
            return Ok(());
        }

        // Load migration SQL directly
        let mut migration_sql =
            include_str!("../../../../migrations/20250331180000_initial_schema.sql").to_string();

        // Make tables creation idempotent by adding IF NOT EXISTS
        migration_sql = migration_sql
            .replace("CREATE TABLE ", "CREATE TABLE IF NOT EXISTS ")
            .replace("CREATE INDEX ", "CREATE INDEX IF NOT EXISTS ");

        info!("Applying initial schema migration");

        // Begin a transaction
        let mut tx = self.pool.begin().await?;

        // Split the SQL file into individual statements and execute each separately
        let statements = migration_sql
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        for statement in statements {
            sqlx::query(statement)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    Error::database(format!(
                        "Failed to apply statement: {} - Error: {}",
                        statement, e
                    ))
                })?;
        }

        // Record the migration in the _sqlx_migrations table
        sqlx::query(
            r#"
            INSERT INTO _sqlx_migrations
            (version, description, success, checksum, execution_time)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (version) DO NOTHING
            "#,
        )
        .bind(20250331180000_i64)
        .bind("initial_schema")
        .bind(true)
        .bind(&[0u8; 32][..]) // Simple checksum placeholder
        .bind(0_i64) // Simple execution time placeholder
        .execute(&mut *tx)
        .await
        .map_err(|e| Error::database(format!("Failed to record migration: {}", e)))?;

        // Commit the transaction
        tx.commit().await?;

        info!("Database migrations complete");
        Ok(())
    }

    /// Create database if it doesn't exist
    pub async fn create_database_if_not_exists(database_url: &str) -> Result<()> {
        let db_exists = Postgres::database_exists(database_url).await?;

        if !db_exists {
            info!("Creating database as it doesn't exist");
            Postgres::create_database(database_url).await?;
            info!("Database created");
        } else {
            info!("Database already exists");
        }

        Ok(())
    }
}
