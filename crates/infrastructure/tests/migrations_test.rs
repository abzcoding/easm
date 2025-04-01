use infrastructure::database::migrations::Migrator;
use shared::errors::Result;
use sqlx::postgres::PgPoolOptions;

// Add mock implementation for database URL environment variable
fn get_test_db_url() -> String {
    // Try to get from env first (allows overriding from CI/CD)
    if let Ok(url) = std::env::var("TEST_DATABASE_URL") {
        return url;
    }

    // Fallback to local test database
    "postgres://postgres:postgres@localhost:5432/easm_test".to_string()
}

#[tokio::test]
async fn test_run_migrations() -> Result<()> {
    let db_url = get_test_db_url();

    // Create pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to create database pool");

    // Create migrator
    let migrator = Migrator::new(pool.clone());

    // Run migrations
    migrator.run_migrations().await?;

    // Verify migrations table exists and has records
    let count = sqlx::query!("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .expect("Failed to query migrations table");

    assert!(count.count.unwrap() > 0, "No migrations were applied");

    Ok(())
}

#[tokio::test]
async fn test_migration_idempotence() -> Result<()> {
    let db_url = get_test_db_url();

    // Create pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to create database pool");

    // Create migrator
    let migrator = Migrator::new(pool.clone());

    // Run migrations first time
    migrator.run_migrations().await?;

    // Get migration count after first run
    let count1 = sqlx::query!("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .expect("Failed to query migrations table");

    // Run migrations again
    migrator.run_migrations().await?;

    // Get migration count after second run
    let count2 = sqlx::query!("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .expect("Failed to query migrations table");

    // Counts should be the same - no new migrations should be applied
    assert_eq!(
        count1.count.unwrap(),
        count2.count.unwrap(),
        "Migration count changed after second run - migrations are not idempotent"
    );

    Ok(())
}
