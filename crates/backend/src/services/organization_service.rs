use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    errors::Result,
    models::Organization,
    traits::{OrganizationRepository, OrganizationService},
};

pub struct OrganizationServiceImpl {
    repo: Arc<dyn OrganizationRepository>,
}

impl OrganizationServiceImpl {
    pub fn new(repo: Arc<dyn OrganizationRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl OrganizationService for OrganizationServiceImpl {
    async fn create_organization(&self, organization: &Organization) -> Result<Organization> {
        // Add validation if needed
        self.repo.create_organization(organization).await
    }

    async fn get_organization(&self, id: Uuid) -> Result<Organization> {
        self.repo.get_organization(id).await
    }

    async fn update_organization(&self, organization: &Organization) -> Result<Organization> {
        // Add validation if needed
        self.repo.update_organization(organization).await
    }

    async fn delete_organization(&self, id: Uuid) -> Result<bool> {
        self.repo.delete_organization(id).await
    }

    async fn list_organizations(&self, limit: usize, offset: usize) -> Result<Vec<Organization>> {
        self.repo.list_organizations(limit, offset).await
    }

    async fn count_organizations(&self) -> Result<usize> {
        self.repo.count_organizations().await
    }
}
