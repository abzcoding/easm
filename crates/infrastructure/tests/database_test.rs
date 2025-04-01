use infrastructure::database::Database;
use shared::errors::Result;

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
async fn test_database_connection() -> Result<()> {
    let db_url = get_test_db_url();
    let db = Database::new(&db_url, 5).await?;

    // Test connection check
    db.check().await?;

    Ok(())
}

#[tokio::test]
async fn test_database_max_connections() -> Result<()> {
    let db_url = get_test_db_url();

    // Create with small connection pool
    let db = Database::new(&db_url, 2).await?;

    // Test connection check
    db.check().await?;

    // Verify pool has expected max size
    assert_eq!(db.pool.size(), 1);

    Ok(())
}

#[tokio::test]
async fn test_database_invalid_url() {
    let result = Database::new("postgres://invalid:invalid@localhost:5432/nonexistent", 5).await;

    // Should return an error
    assert!(result.is_err());
}
