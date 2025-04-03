use backend::{models::Technology, Result};
use infrastructure::{
    database::migrations::Migrator,
    repositories::RepositoryFactory,
    utils::testing::{create_test_asset, create_test_organization},
};
use shared::types::AssetType;
use sqlx::PgPool;

// Helper function to run migrations before tests
async fn setup_database(pool: &PgPool) -> Result<()> {
    let migrator = Migrator::new(pool.clone());
    migrator.run_migrations().await.unwrap();
    Ok(())
}

#[sqlx::test]
async fn test_technology_repository_basic_operations(pool: PgPool) -> Result<()> {
    // Run migrations first
    setup_database(&pool).await?;

    let factory = RepositoryFactory::new(pool);
    let tech_repo = factory.technology_repository();

    // Create test organization and asset
    let org = create_test_organization(&factory, "Test Organization").await?;
    let asset = create_test_asset(&factory, org.id, AssetType::Domain, "example.com").await?;

    // Create technology
    let technology = Technology::new(
        asset.id,
        "WordPress".to_string(),
        Some("5.8.1".to_string()),
        Some("CMS".to_string()),
    );

    // Test create
    let created_tech = tech_repo.create_technology(&technology).await?;
    assert_eq!(created_tech.name, "WordPress");
    assert_eq!(created_tech.version, Some("5.8.1".to_string()));
    assert_eq!(created_tech.category, Some("CMS".to_string()));

    // Test get
    let fetched_tech = tech_repo.get_technology(created_tech.id).await?;
    assert_eq!(fetched_tech.id, created_tech.id);
    assert_eq!(fetched_tech.name, created_tech.name);

    // Test update
    let mut tech_to_update = fetched_tech.clone();
    tech_to_update.name = "WordPress Updated".to_string();
    tech_to_update.version = Some("5.9.0".to_string());

    let updated_tech = tech_repo.update_technology(&tech_to_update).await?;
    assert_eq!(updated_tech.name, "WordPress Updated");
    assert_eq!(updated_tech.version, Some("5.9.0".to_string()));

    // Test list
    let technologies = tech_repo
        .list_technologies(Some(asset.id), None, None, 10, 0)
        .await?;

    assert_eq!(technologies.len(), 1);
    assert_eq!(technologies[0].id, created_tech.id);

    // Test count
    let count = tech_repo
        .count_technologies(Some(asset.id), None, None)
        .await?;

    assert_eq!(count, 1);

    // Test delete
    let deleted = tech_repo.delete_technology(created_tech.id).await?;
    assert!(deleted);

    // Verify deleted
    let count_after_delete = tech_repo
        .count_technologies(Some(asset.id), None, None)
        .await?;

    assert_eq!(count_after_delete, 0);

    Ok(())
}

#[sqlx::test]
async fn test_technology_repository_filters(pool: PgPool) -> Result<()> {
    // Run migrations first
    setup_database(&pool).await?;

    let factory = RepositoryFactory::new(pool);
    let tech_repo = factory.technology_repository();

    // Create test organization and assets
    let org = create_test_organization(&factory, "Test Organization").await?;
    let asset1 = create_test_asset(&factory, org.id, AssetType::Domain, "example1.com").await?;
    let asset2 = create_test_asset(&factory, org.id, AssetType::Domain, "example2.com").await?;

    // Create multiple technologies
    let technologies = vec![
        Technology::new(
            asset1.id,
            "WordPress".to_string(),
            Some("5.8".to_string()),
            Some("CMS".to_string()),
        ),
        Technology::new(
            asset1.id,
            "PHP".to_string(),
            Some("7.4".to_string()),
            Some("ProgrammingLanguage".to_string()),
        ),
        Technology::new(
            asset1.id,
            "MySQL".to_string(),
            Some("8.0".to_string()),
            Some("Database".to_string()),
        ),
        Technology::new(
            asset2.id,
            "WordPress".to_string(),
            Some("5.9".to_string()),
            Some("CMS".to_string()),
        ),
        Technology::new(
            asset2.id,
            "Nginx".to_string(),
            Some("1.20".to_string()),
            Some("WebServer".to_string()),
        ),
    ];

    // Insert all technologies
    for tech in technologies {
        tech_repo.create_technology(&tech).await?;
    }

    // Test filter by asset_id
    let asset1_techs = tech_repo
        .list_technologies(Some(asset1.id), None, None, 10, 0)
        .await?;

    assert_eq!(asset1_techs.len(), 3);

    // Test pagination
    let paginated_techs = tech_repo.list_technologies(None, None, None, 2, 0).await?;

    assert_eq!(paginated_techs.len(), 2);

    // Second page
    let second_page_techs = tech_repo.list_technologies(None, None, None, 2, 2).await?;

    assert_eq!(second_page_techs.len(), 2);

    // Count all
    let count_all = tech_repo.count_technologies(None, None, None).await?;
    assert_eq!(count_all, 5);

    Ok(())
}
