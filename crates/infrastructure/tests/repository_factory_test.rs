use infrastructure::repositories::RepositoryFactory;
use sqlx::{postgres::PgPoolOptions, PgPool};

// Helper function to create a test database connection
async fn create_test_pool() -> PgPool {
    let db_url = if let Ok(url) = std::env::var("TEST_DATABASE_URL") {
        url
    } else {
        "postgres://postgres:postgres@localhost:5432/easm_test".to_string()
    };

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to create database pool")
}

#[tokio::test]
async fn test_repository_factory_creation() {
    // Create a test pool
    let pool = create_test_pool().await;

    // Create repository factory
    let _factory = RepositoryFactory::new(pool);

    // Just assert that creation succeeds without panicking
}

#[tokio::test]
async fn test_repository_factory_repositories() {
    // Create a test pool
    let pool = create_test_pool().await;

    // Create repository factory
    let factory = RepositoryFactory::new(pool);

    // Verify all repositories can be created
    let _org_repo = factory.organization_repository();
    let _user_repo = factory.user_repository();
    let _asset_repo = factory.asset_repository();
    let _port_repo = factory.port_repository();
    let _technology_repo = factory.technology_repository();
    let _vulnerability_repo = factory.vulnerability_repository();
    let _discovery_job_repo = factory.discovery_job_repository();

    // Simply checking that these functions don't panic is sufficient
    // More detailed repository tests are in their respective test files
}

#[tokio::test]
async fn test_repository_factory_cloning() {
    // Create a test pool
    let pool = create_test_pool().await;

    // Create repository factory
    let factory = RepositoryFactory::new(pool);

    // Clone the factory (should share same pool)
    let factory_clone = factory.clone();

    // The clone operation should succeed without error
    // Indirectly test they share the same pool by creating repositories from both
    let _org_repo1 = factory.organization_repository();
    let _org_repo2 = factory_clone.organization_repository();
}
