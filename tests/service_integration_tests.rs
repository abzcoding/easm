#[cfg(test)]
mod service_integration_tests {
    use backend::{
        models::{Asset, Organization},
        traits::{AssetService, DiscoveryService},
    };
    use infrastructure::repositories::RepositoryFactory;
    use shared::{
        config::Config,
        types::{AssetStatus, AssetType},
    };

    async fn setup_test_db() -> RepositoryFactory {
        // Use environment variables for database connection
        let _ = dotenvy::dotenv();
        let config = Config::from_env().expect("Failed to load config");
        let database_url = config.database_url.clone();

        // Create database connection pool
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        // Create repository factory
        RepositoryFactory::new(pool)
    }

    #[tokio::test]
    async fn test_asset_service_with_repository() {
        // Set up the test database and repositories
        let factory = setup_test_db().await;
        let org_repo = factory.organization_repository();
        let asset_repo = factory.asset_repository();

        // Create a test organization
        let org_name = format!("Test Org Asset Service {}", uuid::Uuid::new_v4());
        let org = Organization::new(org_name.clone());
        let created_org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Create the asset service
        let asset_service = backend::services::AssetServiceImpl::new(asset_repo.clone());

        // Test the asset service
        let asset = Asset::new(
            created_org.id,
            AssetType::Domain,
            "service-example.com".to_string(),
            None,
        );

        // Create an asset
        let created_asset = asset_service
            .create_asset(&asset)
            .await
            .expect("Failed to create asset");

        // Verify asset creation
        assert_eq!(created_asset.organization_id, created_org.id);
        assert_eq!(created_asset.value, "service-example.com");

        // Update the asset
        let mut updated_asset = created_asset.clone();
        updated_asset.status = AssetStatus::Inactive;

        let result = asset_service
            .update_asset(&updated_asset)
            .await
            .expect("Failed to update asset");

        assert_eq!(result.status, AssetStatus::Inactive);

        // Get the asset
        let fetched_asset = asset_service
            .get_asset(created_asset.id)
            .await
            .expect("Failed to get asset");

        assert_eq!(fetched_asset.id, created_asset.id);
        assert_eq!(fetched_asset.status, AssetStatus::Inactive);

        // List assets
        let assets = asset_service
            .list_assets(Some(created_org.id), None, None, 10, 0)
            .await
            .expect("Failed to list assets");

        assert!(assets.len() > 0);

        // Delete the asset
        let deleted = asset_service
            .delete_asset(created_asset.id)
            .await
            .expect("Failed to delete asset");

        assert!(deleted);

        // Clean up
        let _ = org_repo
            .delete_organization(created_org.id)
            .await
            .expect("Failed to delete organization");
    }

    #[tokio::test]
    async fn test_discovery_service_workflow() {
        // Set up the test database and repositories
        let factory = setup_test_db().await;
        let org_repo = factory.organization_repository();
        let _asset_repo = factory.asset_repository(); // Keep with underscore as may be used later

        // Create a test organization
        let org_name = format!("Test Org Discovery {}", uuid::Uuid::new_v4());
        let org = Organization::new(org_name.clone());
        let created_org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Create the discovery service
        let discovery_service =
            backend::services::DiscoveryServiceImpl::new(factory.asset_repository());

        // Test the discovery service for a domain
        let domain = "service-discovery-example.com";
        let job = discovery_service
            .discover_assets(created_org.id, domain, vec![])
            .await
            .expect("Failed to discover assets");

        // Basic verification that we get back a job
        assert_eq!(job.organization_id, created_org.id);
        assert!(job.target.is_some());
        assert_eq!(job.target.unwrap(), domain);

        // Clean up
        let _ = org_repo
            .delete_organization(created_org.id)
            .await
            .expect("Failed to delete organization");
    }
}
