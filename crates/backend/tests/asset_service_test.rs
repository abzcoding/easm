#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use backend::models::Asset;
    use backend::services::AssetServiceImpl;
    use backend::{AssetRepository, AssetService, Error, Result};
    use shared::types::{AssetStatus, AssetType, ID};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tokio::test;
    use uuid::Uuid;
    // A mock repository for testing
    #[derive(Clone)]
    struct MockAssetRepository {
        assets: Arc<Mutex<HashMap<ID, Asset>>>,
    }

    impl MockAssetRepository {
        fn new() -> Self {
            Self {
                assets: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl AssetRepository for MockAssetRepository {
        async fn create_asset(&self, asset: &Asset) -> Result<Asset> {
            let mut assets = self.assets.lock().unwrap();
            let new_asset = asset.clone();
            assets.insert(new_asset.id, new_asset.clone());
            Ok(new_asset)
        }

        async fn get_asset(&self, id: ID) -> Result<Asset> {
            let assets = self.assets.lock().unwrap();
            assets
                .get(&id)
                .cloned()
                .ok_or_else(|| Error::NotFound(format!("Asset with ID {} not found", id)))
        }

        async fn update_asset(&self, asset: &Asset) -> Result<Asset> {
            let mut assets = self.assets.lock().unwrap();

            if !assets.contains_key(&asset.id) {
                return Err(Error::NotFound(format!(
                    "Asset with ID {} not found",
                    asset.id
                )));
            }

            let updated_asset = asset.clone();
            assets.insert(updated_asset.id, updated_asset.clone());
            Ok(updated_asset)
        }

        async fn delete_asset(&self, id: ID) -> Result<bool> {
            let mut assets = self.assets.lock().unwrap();

            if !assets.contains_key(&id) {
                return Err(Error::NotFound(format!("Asset with ID {} not found", id)));
            }

            assets.remove(&id);
            Ok(true)
        }

        async fn list_assets(
            &self,
            organization_id: Option<ID>,
            asset_type: Option<AssetType>,
            status: Option<AssetStatus>,
            limit: usize,
            offset: usize,
        ) -> Result<Vec<Asset>> {
            let assets = self.assets.lock().unwrap();

            let filtered: Vec<Asset> = assets
                .values()
                .filter(|a| {
                    organization_id.is_none_or(|oid| a.organization_id == oid)
                        && asset_type.is_none_or(|at| a.asset_type == at)
                        && status.is_none_or(|s| a.status == s)
                })
                .cloned()
                .collect();

            let paginated = filtered.into_iter().skip(offset).take(limit).collect();

            Ok(paginated)
        }

        async fn count_assets(
            &self,
            organization_id: Option<ID>,
            asset_type: Option<AssetType>,
            status: Option<AssetStatus>,
        ) -> Result<usize> {
            let assets = self.assets.lock().unwrap();

            let count = assets
                .values()
                .filter(|a| {
                    organization_id.is_none_or(|oid| a.organization_id == oid)
                        && asset_type.is_none_or(|at| a.asset_type == at)
                        && status.is_none_or(|s| a.status == s)
                })
                .count();

            Ok(count)
        }
    }

    // Service tests
    #[test]
    async fn test_create_asset() {
        let repository = MockAssetRepository::new();
        let service = AssetServiceImpl::new(Arc::new(repository));

        let org_id = Uuid::new_v4();
        let asset = Asset::new(org_id, AssetType::Domain, "example.com".into(), None);

        let created = service.create_asset(&asset).await.unwrap();
        assert_eq!(created.organization_id, org_id);
        assert_eq!(created.asset_type, AssetType::Domain);
        assert_eq!(created.value, "example.com");
    }

    #[test]
    async fn test_get_asset() {
        let repository = MockAssetRepository::new();
        let service = AssetServiceImpl::new(Arc::new(repository));

        let org_id = Uuid::new_v4();
        let asset = Asset::new(org_id, AssetType::Domain, "example.com".into(), None);

        // First create the asset
        let created = service.create_asset(&asset).await.unwrap();

        // Then try to get it
        let retrieved = service.get_asset(created.id).await.unwrap();

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.value, "example.com");
    }

    #[test]
    async fn test_update_asset() {
        let repository = MockAssetRepository::new();
        let service = AssetServiceImpl::new(Arc::new(repository));

        let org_id = Uuid::new_v4();
        let mut asset = Asset::new(org_id, AssetType::Domain, "example.com".into(), None);

        // First create the asset
        let created = service.create_asset(&asset).await.unwrap();

        // Modify the asset
        asset = created;
        asset.status = AssetStatus::Inactive;

        // Update it
        let updated = service.update_asset(&asset).await.unwrap();

        assert_eq!(updated.id, asset.id);
        assert_eq!(updated.status, AssetStatus::Inactive);
    }

    #[test]
    async fn test_delete_asset() {
        let repository = MockAssetRepository::new();
        let service = AssetServiceImpl::new(Arc::new(repository));

        let org_id = Uuid::new_v4();
        let asset = Asset::new(org_id, AssetType::Domain, "example.com".into(), None);

        // Create the asset
        let created = service.create_asset(&asset).await.unwrap();

        // Delete it
        let result = service.delete_asset(created.id).await.unwrap();
        assert!(result);

        // Verify it's gone
        let err = service.get_asset(created.id).await.unwrap_err();
        match err {
            backend::Error::NotFound(_) => {}
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    async fn test_list_assets() {
        let repository = MockAssetRepository::new();
        let service = AssetServiceImpl::new(Arc::new(repository));

        let org_id = Uuid::new_v4();

        // Create multiple assets
        let assets = vec![
            Asset::new(org_id, AssetType::Domain, "example.com".into(), None),
            Asset::new(org_id, AssetType::Domain, "test.com".into(), None),
            Asset::new(org_id, AssetType::IPAddress, "192.0.2.1".into(), None),
            Asset::new(
                Uuid::new_v4(),
                AssetType::Domain,
                "other-org.com".into(),
                None,
            ),
        ];

        for asset in &assets {
            service.create_asset(asset).await.unwrap();
        }

        // Test filtering by organization
        let results = service
            .list_assets(Some(org_id), None, None, 10, 0)
            .await
            .unwrap();
        assert_eq!(results.len(), 3);

        // Test filtering by asset type
        let results = service
            .list_assets(Some(org_id), Some(AssetType::Domain), None, 10, 0)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);

        // Test pagination
        let results = service
            .list_assets(Some(org_id), None, None, 1, 1)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    async fn test_count_assets() {
        let repository = MockAssetRepository::new();
        let service = AssetServiceImpl::new(Arc::new(repository));

        let org_id = Uuid::new_v4();

        // Create multiple assets
        let assets = vec![
            Asset::new(org_id, AssetType::Domain, "example.com".into(), None),
            Asset::new(org_id, AssetType::Domain, "test.com".into(), None),
            Asset::new(org_id, AssetType::IPAddress, "192.0.2.1".into(), None),
            Asset::new(
                Uuid::new_v4(),
                AssetType::Domain,
                "other-org.com".into(),
                None,
            ),
        ];

        for asset in &assets {
            service.create_asset(asset).await.unwrap();
        }

        // Test counting with filters
        let count = service
            .count_assets(Some(org_id), None, None)
            .await
            .unwrap();
        assert_eq!(count, 3);

        let count = service
            .count_assets(Some(org_id), Some(AssetType::Domain), None)
            .await
            .unwrap();
        assert_eq!(count, 2);

        let count = service.count_assets(None, None, None).await.unwrap();
        assert_eq!(count, 4);
    }
}
