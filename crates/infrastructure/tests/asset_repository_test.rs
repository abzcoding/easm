#[cfg(test)]
mod tests {
    use backend::models::Asset;
    use infrastructure::{
        repositories::factory::RepositoryFactory,
        utils::testing::{create_test_asset, create_test_organization, setup_test_db},
    };
    use shared::types::{AssetStatus, AssetType, PaginationParams};

    #[tokio::test]
    async fn test_asset_repository_basic_operations() {
        let (db_pool, _container) = setup_test_db().await;
        let factory = RepositoryFactory::new(db_pool);
        let asset_repo = factory.asset_repository();
        let _org_repo = factory.organization_repository();

        // Setup: Create an organization
        let org = create_test_organization(&factory, "Test Org Asset Repo")
            .await
            .unwrap();

        // 1. Create Asset
        let new_asset = Asset::new(org.id, AssetType::Domain, "example.com".to_string(), None);
        let created_asset = asset_repo
            .create_asset(&new_asset)
            .await
            .expect("Failed to create asset");
        assert_eq!(created_asset.value, "example.com");
        assert_eq!(created_asset.asset_type, AssetType::Domain);
        assert_eq!(created_asset.organization_id, org.id);

        // 2. Find Asset by ID
        let found_asset = asset_repo
            .get_asset(created_asset.id)
            .await
            .expect("Failed to find asset by ID");
        assert_eq!(found_asset.id, created_asset.id);
        assert_eq!(found_asset.value, "example.com");

        // 3. Update Asset
        let mut asset_to_update = found_asset.clone();
        asset_to_update.status = AssetStatus::Inactive;
        let updated_asset = asset_repo
            .update_asset(&asset_to_update)
            .await
            .expect("Failed to update asset");
        assert_eq!(updated_asset.id, created_asset.id);
        assert_eq!(updated_asset.status, AssetStatus::Inactive);

        // Verify update
        let verified_asset = asset_repo
            .get_asset(created_asset.id)
            .await
            .expect("Failed to find asset after update");
        assert_eq!(verified_asset.status, AssetStatus::Inactive);

        // 4. Delete Asset
        let deleted = asset_repo
            .delete_asset(created_asset.id)
            .await
            .expect("Failed to delete asset");
        assert!(deleted);

        // Verify deletion
        let not_found_result = asset_repo.get_asset(created_asset.id).await;
        assert!(not_found_result.is_err());
        // Ideally, check if the error is Error::NotFound
    }

    #[tokio::test]
    async fn test_asset_repository_list_and_filters() {
        let (db_pool, _container) = setup_test_db().await;
        let factory = RepositoryFactory::new(db_pool.clone());
        let asset_repo = factory.asset_repository();
        let _org_repo = factory.organization_repository();

        // Setup: Create organizations and assets
        let org1 = create_test_organization(&factory, "Org 1 Assets")
            .await
            .unwrap();
        let org2 = create_test_organization(&factory, "Org 2 Assets")
            .await
            .unwrap();

        let asset1_org1 = create_test_asset(&factory, org1.id, AssetType::Domain, "site1.org1.com")
            .await
            .unwrap();
        let mut asset2_org1 =
            create_test_asset(&factory, org1.id, AssetType::IPAddress, "192.168.1.1")
                .await
                .unwrap();
        asset2_org1.status = AssetStatus::Inactive;
        asset_repo.update_asset(&asset2_org1).await.unwrap();

        let _asset1_org2 =
            create_test_asset(&factory, org2.id, AssetType::Domain, "site1.org2.com")
                .await
                .unwrap();

        // List all for Org 1
        let pagination = PaginationParams {
            page: 1,
            page_size: 10,
        };
        let assets_org1 = asset_repo
            .list_assets(
                Some(org1.id),
                None,
                None,
                pagination.page_size as usize,
                pagination.offset() as usize,
            )
            .await
            .expect("Failed to list assets for Org 1");
        assert_eq!(assets_org1.len(), 2);

        // List active for Org 1
        let active_assets_org1 = asset_repo
            .list_assets(
                Some(org1.id),
                None,
                Some(AssetStatus::Active),
                pagination.page_size as usize,
                pagination.offset() as usize,
            )
            .await
            .expect("Failed to list active assets for Org 1");
        assert_eq!(active_assets_org1.len(), 1);
        assert_eq!(active_assets_org1[0].id, asset1_org1.id);
        assert_eq!(active_assets_org1[0].status, AssetStatus::Active);

        // List Domain type for Org 1
        let domain_assets_org1 = asset_repo
            .list_assets(
                Some(org1.id),
                Some(AssetType::Domain),
                None,
                pagination.page_size as usize,
                pagination.offset() as usize,
            )
            .await
            .expect("Failed to list domain assets for Org 1");
        assert_eq!(domain_assets_org1.len(), 1);
        assert_eq!(domain_assets_org1[0].id, asset1_org1.id);
        assert_eq!(domain_assets_org1[0].asset_type, AssetType::Domain);

        // Count assets for Org 1
        let count_org1 = asset_repo
            .count_assets(Some(org1.id), None, None)
            .await
            .expect("Failed to count assets for Org 1");
        assert_eq!(count_org1, 2);

        // Count active for Org 1
        let count_active_org1 = asset_repo
            .count_assets(Some(org1.id), None, Some(AssetStatus::Active))
            .await
            .expect("Failed to count active assets for Org 1");
        assert_eq!(count_active_org1, 1);

        // Count Domain type for Org 1
        let count_domain_org1 = asset_repo
            .count_assets(Some(org1.id), Some(AssetType::Domain), None)
            .await
            .expect("Failed to count domain assets for Org 1");
        assert_eq!(count_domain_org1, 1);
    }

    #[tokio::test]
    async fn test_asset_repository_uniqueness_constraint() {
        let (db_pool, _container) = setup_test_db().await;
        let factory = RepositoryFactory::new(db_pool.clone());
        let asset_repo = factory.asset_repository();
        let _org_repo = factory.organization_repository();

        // Setup
        let org = create_test_organization(&factory, "Unique Asset Org")
            .await
            .unwrap();
        let _asset1 = create_test_asset(&factory, org.id, AssetType::Domain, "unique.com")
            .await
            .unwrap();

        // Attempt to create duplicate
        let duplicate_asset = Asset::new(org.id, AssetType::Domain, "unique.com".to_string(), None);
        let result = asset_repo.create_asset(&duplicate_asset).await;

        assert!(result.is_err());
        // Ideally, check for a specific database duplicate key error type
    }
}
