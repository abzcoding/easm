use async_trait::async_trait;
use shared::types::{AssetStatus, AssetType, ID};
use tracing::{debug, info};

use crate::{
    models::Asset,
    traits::{AssetRepository, AssetService},
    Result,
};

pub struct AssetServiceImpl<R: AssetRepository> {
    repository: R,
}

impl<R: AssetRepository> AssetServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: AssetRepository> AssetService for AssetServiceImpl<R> {
    async fn create_asset(&self, asset: &Asset) -> Result<Asset> {
        info!("Creating new asset: {}", asset.value);
        let result = self.repository.create_asset(asset).await?;
        debug!("Asset created with ID: {}", result.id);
        Ok(result)
    }

    async fn get_asset(&self, id: ID) -> Result<Asset> {
        debug!("Getting asset with ID: {}", id);
        self.repository.get_asset(id).await
    }

    async fn update_asset(&self, asset: &Asset) -> Result<Asset> {
        debug!("Updating asset with ID: {}", asset.id);
        self.repository.update_asset(asset).await
    }

    async fn delete_asset(&self, id: ID) -> Result<bool> {
        info!("Deleting asset with ID: {}", id);
        self.repository.delete_asset(id).await
    }

    async fn list_assets(
        &self,
        organization_id: Option<ID>,
        asset_type: Option<AssetType>,
        status: Option<AssetStatus>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Asset>> {
        debug!(
            "Listing assets with filters - org: {:?}, type: {:?}, status: {:?}, limit: {}, offset: {}",
            organization_id, asset_type, status, limit, offset
        );
        self.repository
            .list_assets(organization_id, asset_type, status, limit, offset)
            .await
    }

    async fn count_assets(
        &self,
        organization_id: Option<ID>,
        asset_type: Option<AssetType>,
        status: Option<AssetStatus>,
    ) -> Result<usize> {
        debug!(
            "Counting assets with filters - org: {:?}, type: {:?}, status: {:?}",
            organization_id, asset_type, status
        );
        self.repository
            .count_assets(organization_id, asset_type, status)
            .await
    }
}
