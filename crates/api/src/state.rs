use std::sync::Arc;

use backend::{
    services::{AssetServiceImpl, DiscoveryServiceImpl, VulnerabilityServiceImpl},
    AssetService, DiscoveryService, VulnerabilityService,
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
                shared::errors::Error::external_service(format!("Redis error: {}", e))
            })?)
        } else {
            None
        };

        // Create service instances
        let asset_repo = repo_factory.asset_repository();
        let vulnerability_repo = repo_factory.vulnerability_repository();

        // Create services
        let asset_service = Arc::new(AssetServiceImpl::new(asset_repo.clone()));
        let vulnerability_service = Arc::new(VulnerabilityServiceImpl::new(vulnerability_repo));
        let discovery_service = Arc::new(DiscoveryServiceImpl::new(asset_repo));

        Ok(Self {
            config: config.clone(),
            db_pool,
            redis_client,
            asset_service,
            vulnerability_service,
            discovery_service,
        })
    }
}
