use serde_json::json;
// use shared::errors::Result;
use backend::{Error, Result};
use sqlx::{Pool, Postgres};
use testcontainers::{core::IntoContainerPort, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres as PostgresContainer;

use crate::repositories::RepositoryFactory;

/// Create a new repository factory for tests with a connection to the test database
pub fn create_test_repo_factory(pool: Pool<Postgres>) -> RepositoryFactory {
    RepositoryFactory::new(pool)
}

/// Helper function to apply fixtures to test database
#[allow(dead_code)]
pub async fn apply_fixture(pool: &Pool<Postgres>, fixture_name: &str) -> Result<()> {
    // Assumes fixture files are in tests/fixtures directory
    let fixture_path = format!("tests/fixtures/{}.sql", fixture_name);
    let fixture_content = match std::fs::read_to_string(fixture_path) {
        Ok(it) => it,
        Err(err) => return Err(Error::Internal(err.to_string())),
    };

    sqlx::query(&fixture_content).execute(pool).await?;

    Ok(())
}

/// Create a test organization
#[allow(dead_code)]
pub async fn create_test_organization(
    factory: &RepositoryFactory,
    name: &str,
) -> Result<backend::models::Organization> {
    let org = backend::models::Organization::new(name.to_string());
    let org_repo = factory.organization_repository();
    org_repo.create_organization(&org).await
}

/// Create a test user
#[allow(dead_code)]
pub async fn create_test_user(
    factory: &RepositoryFactory,
    org_id: shared::types::ID,
    username: &str,
    email: &str,
    role: shared::types::UserRole,
) -> Result<backend::models::User> {
    let user = backend::models::User::new(
        org_id,
        username.to_string(),
        email.to_string(),
        "password-hash".to_string(),
        Some(role),
    );
    let user_repo = factory.user_repository();
    user_repo.create_user(&user).await
}

/// Create a test asset
#[allow(dead_code)]
pub async fn create_test_asset(
    factory: &RepositoryFactory,
    org_id: shared::types::ID,
    asset_type: shared::types::AssetType,
    value: &str,
) -> Result<backend::models::Asset> {
    let attributes = json!({});
    let asset =
        backend::models::Asset::new(org_id, asset_type, value.to_string(), Some(attributes));
    let asset_repo = factory.asset_repository();
    asset_repo.create_asset(&asset).await
}

/// Sets up a test database using testcontainers and returns the connection pool
/// and the container instance (which must be kept in scope).
#[allow(dead_code)]
pub async fn setup_test_db() -> (
    Pool<Postgres>,
    testcontainers::ContainerAsync<PostgresContainer>,
) {
    let container = PostgresContainer::default()
        .start()
        .await
        .expect("Failed to start postgres container");
    let port = container
        .get_host_port_ipv4(5432.tcp())
        .await
        .expect("Failed to get container port");
    let connection_string = format!("postgres://postgres:postgres@localhost:{}/postgres", port);

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations on test database");

    (pool, container)
}
