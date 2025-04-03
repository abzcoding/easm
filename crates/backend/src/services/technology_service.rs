use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, info};
use uuid::Uuid;

use crate::{
    models::Technology,
    traits::{AssetRepository, TechnologyRepository, TechnologyService},
    Result,
};

pub struct TechnologyServiceImpl {
    repository: Arc<dyn TechnologyRepository>,
    asset_repository: Arc<dyn AssetRepository>,
}

impl TechnologyServiceImpl {
    pub fn new(
        repository: Arc<dyn TechnologyRepository>,
        asset_repository: Arc<dyn AssetRepository>,
    ) -> Self {
        Self {
            repository,
            asset_repository,
        }
    }
}

#[async_trait]
impl TechnologyService for TechnologyServiceImpl {
    async fn create_technology(&self, technology: &Technology) -> Result<Technology> {
        debug!("Creating technology: {:?}", technology);
        self.repository.create_technology(technology).await
    }

    async fn get_technology(&self, id: Uuid) -> Result<Technology> {
        debug!("Getting technology by ID: {}", id);
        self.repository.get_technology(id).await
    }

    async fn update_technology(&self, technology: &Technology) -> Result<Technology> {
        debug!("Updating technology: {:?}", technology);
        self.repository.update_technology(technology).await
    }

    async fn delete_technology(&self, id: Uuid) -> Result<bool> {
        debug!("Deleting technology with ID: {}", id);
        self.repository.delete_technology(id).await
    }

    async fn list_technologies(
        &self,
        asset_id: Option<Uuid>,
        name: Option<String>,
        category: Option<String>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Technology>> {
        debug!(
            "Listing technologies - asset_id: {:?}, name: {:?}, category: {:?}, limit: {}, offset: {}",
            asset_id, name, category, limit, offset
        );
        self.repository
            .list_technologies(asset_id, name, category, limit, offset)
            .await
    }

    async fn count_technologies(
        &self,
        asset_id: Option<Uuid>,
        name: Option<String>,
        category: Option<String>,
    ) -> Result<usize> {
        debug!(
            "Counting technologies - asset_id: {:?}, name: {:?}, category: {:?}",
            asset_id, name, category
        );
        self.repository
            .count_technologies(asset_id, name, category)
            .await
    }

    async fn get_technology_statistics(
        &self,
        organization_id: Uuid,
    ) -> Result<HashMap<String, usize>> {
        info!(
            "Getting technology statistics for organization: {}",
            organization_id
        );

        // Get assets for the organization
        let assets = self
            .asset_repository
            .list_assets(Some(organization_id), None, None, 1000, 0)
            .await?;

        if assets.is_empty() {
            debug!("No assets found for organization: {}", organization_id);
            return Ok(HashMap::new());
        }

        // Get asset IDs
        let asset_ids: Vec<Uuid> = assets.iter().map(|a| a.id).collect();

        let mut stats = HashMap::new();

        // Get technologies by category
        for asset_id in asset_ids {
            let technologies = self
                .repository
                .list_technologies(Some(asset_id), None, None, 1000, 0)
                .await?;

            for tech in technologies {
                if let Some(category) = &tech.category {
                    let count = stats.entry(category.clone()).or_insert(0);
                    *count += 1;
                } else {
                    let count = stats.entry("Uncategorized".to_string()).or_insert(0);
                    *count += 1;
                }
            }
        }

        debug!("Technology statistics: {:?}", stats);
        Ok(stats)
    }
}
