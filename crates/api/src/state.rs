use std::sync::Arc;

use backend::{
    services::{
        AssetServiceImpl, DiscoveryServiceImpl, OrganizationServiceImpl, UserServiceImpl,
        VulnerabilityServiceImpl,
    },
    AssetService, DiscoveryJobRepository, DiscoveryService, OrganizationService, UserService,
    VulnerabilityService,
};
use infrastructure::{database::Database, repositories::RepositoryFactory};
use redis::Client as RedisClient;
use shared::{config::Config, errors::Result};
use sqlx::PgPool;

/// Application state shared across all routes
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: PgPool,
    pub redis_client: Option<RedisClient>,
    pub asset_service: Arc<dyn AssetService>,
    pub vulnerability_service: Arc<dyn VulnerabilityService>,
    pub discovery_service: Arc<dyn DiscoveryService>,
    pub discovery_job_repository: Arc<dyn DiscoveryJobRepository>,
    pub user_service: Arc<dyn UserService>,
    pub organization_service: Arc<dyn OrganizationService>,
}

impl AppState {
    pub async fn new(config: &Config) -> Result<Self> {
        // Connect to the database
        let db = Database::new(&config.database_url, 10).await?;
        let db_pool = db.pool;

        // Create repository factory
        let repo_factory = RepositoryFactory::new(db_pool.clone());

        // Connect to Redis if configured
        let redis_client = if let Some(redis_url) = &config.redis_url {
            Some(RedisClient::open(redis_url.as_str()).map_err(|e| {
                shared::errors::AppError::external_service(format!("Redis error: {}", e))
            })?)
        } else {
            None
        };

        // Create repository instances
        let user_repo = repo_factory.user_repository();
        let asset_repo = repo_factory.asset_repository();
        let vulnerability_repo = repo_factory.vulnerability_repository();
        let discovery_asset_repo = repo_factory.asset_repository();
        let discovery_job_repo = repo_factory.discovery_job_repository();
        let organization_repo = repo_factory.organization_repository();

        // Create services
        let user_service: Arc<dyn UserService> =
            Arc::new(UserServiceImpl::new(user_repo, organization_repo.clone()));
        let asset_service: Arc<dyn AssetService> =
            Arc::new(AssetServiceImpl::new(asset_repo.clone()));
        let vulnerability_service: Arc<dyn VulnerabilityService> = Arc::new(
            VulnerabilityServiceImpl::new(vulnerability_repo, asset_repo),
        );
        let discovery_service: Arc<dyn DiscoveryService> = Arc::new(DiscoveryServiceImpl::new(
            discovery_asset_repo,
            discovery_job_repo.clone(),
        ));
        let organization_service: Arc<dyn OrganizationService> =
            Arc::new(OrganizationServiceImpl::new(organization_repo));

        Ok(Self {
            config: config.clone(),
            db_pool,
            redis_client,
            asset_service,
            vulnerability_service,
            discovery_service,
            discovery_job_repository: discovery_job_repo,
            user_service,
            organization_service,
        })
    }
}
