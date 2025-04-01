use backend::traits::{
    AssetRepository, DiscoveryJobRepository, OrganizationRepository, PortRepository,
    TechnologyRepository, UserRepository, VulnerabilityRepository,
};
use sqlx::PgPool;
use std::sync::Arc;

use super::{
    PgAssetRepository, PgDiscoveryJobRepository, PgOrganizationRepository, PgPortRepository,
    PgTechnologyRepository, PgUserRepository, PgVulnerabilityRepository,
};

/// Factory for creating all repositories
#[derive(Clone)]
pub struct RepositoryFactory {
    pool: PgPool,
}

impl RepositoryFactory {
    /// Create a new repository factory
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create an organization repository
    pub fn organization_repository(&self) -> Arc<dyn OrganizationRepository> {
        Arc::new(PgOrganizationRepository::new(self.pool.clone()))
    }

    /// Create a user repository
    pub fn user_repository(&self) -> Arc<dyn UserRepository> {
        Arc::new(PgUserRepository::new(self.pool.clone()))
    }

    /// Create an asset repository
    pub fn asset_repository(&self) -> Arc<dyn AssetRepository> {
        Arc::new(PgAssetRepository::new(self.pool.clone()))
    }

    /// Create a port repository
    pub fn port_repository(&self) -> Arc<dyn PortRepository> {
        Arc::new(PgPortRepository::new(self.pool.clone()))
    }

    /// Create a technology repository
    pub fn technology_repository(&self) -> Arc<dyn TechnologyRepository> {
        Arc::new(PgTechnologyRepository::new(self.pool.clone()))
    }

    /// Create a vulnerability repository
    pub fn vulnerability_repository(&self) -> Arc<dyn VulnerabilityRepository> {
        Arc::new(PgVulnerabilityRepository::new(self.pool.clone()))
    }

    /// Create a discovery job repository
    pub fn discovery_job_repository(&self) -> Arc<dyn DiscoveryJobRepository> {
        Arc::new(PgDiscoveryJobRepository::new(self.pool.clone()))
    }
}
