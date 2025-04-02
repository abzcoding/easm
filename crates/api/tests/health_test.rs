use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::{
    models::{Asset, Vulnerability},
    Result,
};
use http_body_util::BodyExt;
use shared::{
    config::Config,
    types::{AssetStatus, AssetType, JobType, Severity, VulnerabilityStatus, ID},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint() {
    // Create a mock config
    let config = Config {
        database_url: "postgres://postgres:postgres@localhost:5432/easm_test".to_string(),
        redis_url: None,
        host: "127.0.0.1".parse().unwrap(),
        port: 3000,
        jwt_secret: "test_secret".to_string(),
        jwt_expiration: 3600,
        environment: shared::config::Environment::Test,
        log_level: "debug".to_string(),
        max_concurrent_tasks: 5,
    };

    // Create the router
    let router = api::routes::create_router(api::state::AppState {
        config: config.clone(),
        db_pool: sqlx::postgres::PgPool::connect_lazy(&config.database_url).unwrap(),
        redis_client: None,
        asset_service: std::sync::Arc::new(MockAssetService),
        vulnerability_service: std::sync::Arc::new(MockVulnerabilityService),
        discovery_service: std::sync::Arc::new(MockDiscoveryService),
    });

    // Create a request to the health endpoint
    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    // Send the request to the router
    let response = router.oneshot(request).await.unwrap();

    // Check that the response has a 200 OK status
    assert_eq!(response.status(), StatusCode::OK);

    // Check the response body
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check that the response contains the expected data
    assert_eq!(body["status"], "ok");
}

// Mock services for testing
#[derive(Clone)]
struct MockAssetService;

#[async_trait]
impl backend::AssetService for MockAssetService {
    async fn create_asset(&self, _asset: &Asset) -> Result<Asset> {
        unimplemented!()
    }

    async fn get_asset(&self, _id: ID) -> Result<Asset> {
        unimplemented!()
    }

    async fn update_asset(&self, _asset: &Asset) -> Result<Asset> {
        unimplemented!()
    }

    async fn delete_asset(&self, _id: ID) -> Result<bool> {
        unimplemented!()
    }

    async fn list_assets(
        &self,
        _organization_id: Option<ID>,
        _asset_type: Option<AssetType>,
        _status: Option<AssetStatus>,
        _limit: usize,
        _offset: usize,
    ) -> Result<Vec<Asset>> {
        unimplemented!()
    }

    async fn count_assets(
        &self,
        _organization_id: Option<ID>,
        _asset_type: Option<AssetType>,
        _status: Option<AssetStatus>,
    ) -> Result<usize> {
        unimplemented!()
    }
}

#[derive(Clone)]
struct MockVulnerabilityService;

#[async_trait]
impl backend::VulnerabilityService for MockVulnerabilityService {
    async fn create_vulnerability(&self, _vulnerability: &Vulnerability) -> Result<Vulnerability> {
        unimplemented!()
    }

    async fn get_vulnerability(&self, _id: ID) -> Result<Vulnerability> {
        unimplemented!()
    }

    async fn update_vulnerability(&self, _vulnerability: &Vulnerability) -> Result<Vulnerability> {
        unimplemented!()
    }

    async fn delete_vulnerability(&self, _id: ID) -> Result<bool> {
        unimplemented!()
    }

    async fn list_vulnerabilities(
        &self,
        _asset_id: Option<ID>,
        _port_id: Option<ID>,
        _severity: Option<Severity>,
        _status: Option<VulnerabilityStatus>,
        _limit: usize,
        _offset: usize,
    ) -> Result<Vec<Vulnerability>> {
        unimplemented!()
    }

    async fn count_vulnerabilities(
        &self,
        _asset_id: Option<ID>,
        _port_id: Option<ID>,
        _severity: Option<Severity>,
        _status: Option<VulnerabilityStatus>,
    ) -> Result<usize> {
        unimplemented!()
    }
}

#[derive(Clone)]
struct MockDiscoveryService;

#[async_trait]
impl backend::DiscoveryService for MockDiscoveryService {
    async fn discover_assets(
        &self,
        _organization_id: ID,
        _domain: &str,
        _job_types: Vec<JobType>,
    ) -> Result<backend::models::DiscoveryJob> {
        unimplemented!()
    }

    async fn scan_asset(&self, _asset_id: ID) -> Result<Vec<Vulnerability>> {
        unimplemented!()
    }
}
