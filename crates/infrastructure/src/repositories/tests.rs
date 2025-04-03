use backend::models::{Asset, Organization, User};
use shared::types::{AssetStatus, AssetType, UserRole};
use sqlx::PgPool;

use super::RepositoryFactory;
use crate::database::migrations::Migrator;

// Helper function to run migrations before tests
async fn setup_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    let migrator = Migrator::new(pool.clone());
    migrator.run_migrations().await.unwrap();
    Ok(())
}

#[sqlx::test]
async fn test_organization_repository(pool: PgPool) -> sqlx::Result<()> {
    // Run migrations first
    setup_database(&pool).await?;

    let factory = RepositoryFactory::new(pool);
    let org_repo = factory.organization_repository();

    // Create organization
    let org = Organization::new("Test Organization".to_string());
    let created_org = org_repo.create_organization(&org).await.unwrap();

    // Verify ID is assigned
    assert_eq!(created_org.name, "Test Organization");

    // Get organization by ID
    let fetched_org = org_repo.get_organization(created_org.id).await.unwrap();
    assert_eq!(fetched_org.id, created_org.id);
    assert_eq!(fetched_org.name, created_org.name);

    // Update organization
    let mut updated_org = fetched_org.clone();
    updated_org.name = "Updated Organization".to_string();
    let updated_org = org_repo.update_organization(&updated_org).await.unwrap();
    assert_eq!(updated_org.name, "Updated Organization");

    // List organizations
    let orgs = org_repo.list_organizations(10, 0).await.unwrap();
    assert!(orgs.len() > 0);

    // Count organizations
    let count = org_repo.count_organizations().await.unwrap();
    assert!(count > 0);

    // Delete organization
    let deleted = org_repo.delete_organization(created_org.id).await.unwrap();
    assert!(deleted);

    Ok(())
}

#[sqlx::test]
async fn test_user_repository(pool: PgPool) -> sqlx::Result<()> {
    // Run migrations first
    setup_database(&pool).await?;

    let factory = RepositoryFactory::new(pool);
    let org_repo = factory.organization_repository();
    let user_repo = factory.user_repository();

    // Create organization
    let org = Organization::new("Test Organization".to_string());
    let created_org = org_repo.create_organization(&org).await.unwrap();

    // Create user
    let user = User::new(
        created_org.id,
        "testuser".to_string(),
        "test@example.com".to_string(),
        "password-hash".to_string(),
        Some(UserRole::Admin),
    );
    let created_user = user_repo.create_user(&user).await.unwrap();

    // Verify ID is assigned
    assert_eq!(created_user.username, "testuser");
    assert_eq!(created_user.email, "test@example.com");
    assert_eq!(created_user.role, UserRole::Admin);

    // Get user by ID
    let fetched_user = user_repo.get_user(created_user.id).await.unwrap();
    assert_eq!(fetched_user.id, created_user.id);
    assert_eq!(fetched_user.username, created_user.username);

    // Get by username
    let user_by_username = user_repo.get_user_by_username("testuser").await.unwrap();
    assert_eq!(user_by_username.unwrap().id, created_user.id);

    // Get by email
    let user_by_email = user_repo
        .get_user_by_email("test@example.com")
        .await
        .unwrap();
    assert_eq!(user_by_email.unwrap().id, created_user.id);

    // Update user
    let mut updated_user = fetched_user.clone();
    updated_user.username = "updateduser".to_string();
    let updated_user = user_repo.update_user(&updated_user).await.unwrap();
    assert_eq!(updated_user.username, "updateduser");

    // List users
    let users = user_repo
        .list_users(Some(created_org.id), None, 10, 0)
        .await
        .unwrap();
    assert!(users.len() > 0);

    // Count users
    let count = user_repo
        .count_users(Some(created_org.id), None)
        .await
        .unwrap();
    assert!(count > 0);

    // Delete user
    let deleted = user_repo.delete_user(created_user.id).await.unwrap();
    assert!(deleted);

    // Cleanup
    org_repo.delete_organization(created_org.id).await.unwrap();

    Ok(())
}

#[sqlx::test]
async fn test_asset_repository(pool: PgPool) -> sqlx::Result<()> {
    // Run migrations first
    setup_database(&pool).await?;

    let factory = RepositoryFactory::new(pool);
    let org_repo = factory.organization_repository();
    let asset_repo = factory.asset_repository();

    // Create organization
    let org = Organization::new("Test Organization".to_string());
    let created_org = org_repo.create_organization(&org).await.unwrap();

    // Create asset
    let asset = Asset::new(
        created_org.id,
        AssetType::Domain,
        "example.com".to_string(),
        None, // attributes
    );
    let created_asset = asset_repo.create_asset(&asset).await.unwrap();

    // Verify ID is assigned
    assert_eq!(created_asset.value, "example.com");
    assert_eq!(created_asset.asset_type, AssetType::Domain);
    assert_eq!(created_asset.status, AssetStatus::Active);

    // Get asset by ID
    let fetched_asset = asset_repo.get_asset(created_asset.id).await.unwrap();
    assert_eq!(fetched_asset.id, created_asset.id);
    assert_eq!(fetched_asset.value, created_asset.value);

    // Update asset
    let mut updated_asset = fetched_asset.clone();
    updated_asset.value = "updated.example.com".to_string();
    let updated_asset = asset_repo.update_asset(&updated_asset).await.unwrap();
    assert_eq!(updated_asset.value, "updated.example.com");

    // List assets
    let assets = asset_repo
        .list_assets(
            Some(created_org.id),
            Some(AssetType::Domain),
            Some(AssetStatus::Active),
            10,
            0,
        )
        .await
        .unwrap();
    assert!(assets.len() > 0);

    // Count assets
    let count = asset_repo
        .count_assets(
            Some(created_org.id),
            Some(AssetType::Domain),
            Some(AssetStatus::Active),
        )
        .await
        .unwrap();
    assert!(count > 0);

    // Delete asset
    let deleted = asset_repo.delete_asset(created_asset.id).await.unwrap();
    assert!(deleted);

    // Cleanup
    org_repo.delete_organization(created_org.id).await.unwrap();

    Ok(())
}
