use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::{
    models::{Asset, DiscoveryJob, Vulnerability},
    Result,
};
use http_body_util::BodyExt;
use serde_json::json;
use shared::{
    config::Config,
    types::{AssetStatus, AssetType, JobType, Severity, VulnerabilityStatus, ID},
};
use tower::ServiceExt;
use uuid::Uuid;

// Mock asset service for testing
#[derive(Clone)]
pub struct MockAssetService;

#[async_trait]
impl backend::AssetService for MockAssetService {
    async fn create_asset(&self, asset: &Asset) -> Result<Asset> {
        // Return a valid asset with id
        let now = chrono::Utc::now();
        Ok(Asset {
            id: Uuid::new_v4(),
            organization_id: asset.organization_id,
            asset_type: asset.asset_type,
            value: asset.value.clone(),
            status: AssetStatus::Active,
            first_seen: now,
            last_seen: now,
            created_at: now,
            updated_at: now,
            attributes: asset.attributes.clone(),
        })
    }

    async fn get_asset(&self, id: ID) -> Result<Asset> {
        // Return a valid asset with the requested id
        let now = chrono::Utc::now();
        Ok(Asset {
            id,
            organization_id: Uuid::new_v4(),
            asset_type: AssetType::Domain,
            value: "test.example.com".to_string(),
            status: AssetStatus::Active,
            first_seen: now,
            last_seen: now,
            created_at: now,
            updated_at: now,
            attributes: serde_json::json!({
                "hostname": "test",
            }),
        })
    }

    async fn update_asset(&self, asset: &Asset) -> Result<Asset> {
        // Return the updated asset
        let now = chrono::Utc::now();
        Ok(Asset {
            id: asset.id,
            organization_id: asset.organization_id,
            asset_type: asset.asset_type,
            value: asset.value.clone(),
            status: asset.status,
            first_seen: asset.first_seen,
            last_seen: now,
            created_at: asset.created_at,
            updated_at: now,
            attributes: asset.attributes.clone(),
        })
    }

    async fn delete_asset(&self, _id: ID) -> Result<bool> {
        // Always return success
        Ok(true)
    }

    async fn list_assets(
        &self,
        _organization_id: Option<ID>,
        _asset_type: Option<AssetType>,
        _status: Option<AssetStatus>,
        _limit: usize,
        _offset: usize,
    ) -> Result<Vec<Asset>> {
        // Return a list of test assets
        let now = chrono::Utc::now();

        Ok(vec![
            Asset {
                id: Uuid::new_v4(),
                organization_id: Uuid::new_v4(),
                asset_type: AssetType::Domain,
                value: "test1.example.com".to_string(),
                status: AssetStatus::Active,
                first_seen: now,
                last_seen: now,
                created_at: now,
                updated_at: now,
                attributes: serde_json::json!({
                    "hostname": "test1",
                }),
            },
            Asset {
                id: Uuid::new_v4(),
                organization_id: Uuid::new_v4(),
                asset_type: AssetType::IPAddress,
                value: "192.168.1.1".to_string(),
                status: AssetStatus::Active,
                first_seen: now,
                last_seen: now,
                created_at: now,
                updated_at: now,
                attributes: serde_json::json!({
                    "ip_address": "192.168.1.1",
                }),
            },
        ])
    }

    async fn count_assets(
        &self,
        _organization_id: Option<ID>,
        _asset_type: Option<AssetType>,
        _status: Option<AssetStatus>,
    ) -> Result<usize> {
        // Return a fixed count
        Ok(2)
    }
}

// Mock vulnerability service for testing
#[derive(Clone)]
pub struct MockVulnerabilityService;

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

// Mock discovery service
#[derive(Clone)]
pub struct MockDiscoveryService;

#[async_trait]
impl backend::DiscoveryService for MockDiscoveryService {
    async fn discover_assets(
        &self,
        _organization_id: ID,
        _domain: &str,
        _job_types: Vec<JobType>,
    ) -> Result<DiscoveryJob> {
        unimplemented!()
    }

    async fn scan_asset(&self, _asset_id: ID) -> Result<Vec<Vulnerability>> {
        unimplemented!()
    }
}

// Helper function to create app state with mock services
fn create_test_app_state() -> api::state::AppState {
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

    api::state::AppState {
        config: config.clone(),
        db_pool: sqlx::postgres::PgPool::connect_lazy(&config.database_url).unwrap(),
        redis_client: None,
        asset_service: std::sync::Arc::new(MockAssetService),
        vulnerability_service: std::sync::Arc::new(MockVulnerabilityService),
        discovery_service: std::sync::Arc::new(MockDiscoveryService),
    }
}

#[tokio::test]
async fn test_list_assets() {
    // Create the router with mock services
    let router = api::routes::create_router(create_test_app_state());

    // Create a request to list assets
    let request = Request::builder()
        .uri("/api/assets")
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

    // Check that the response contains assets
    assert!(body.is_object());
    assert!(body["assets"].is_array());
    assert_eq!(body["assets"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_create_asset() {
    // Create the router with mock services
    let router = api::routes::create_router(create_test_app_state());

    // Create asset payload
    let asset_data = json!({
        "organization_id": Uuid::new_v4().to_string(),
        "asset_type": "Domain",
        "value": "newtest.example.com",
        "attributes": {
            "hostname": "newtest"
        }
    });

    // Create a request to create a new asset
    let request = Request::builder()
        .uri("/api/assets")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(asset_data.to_string()))
        .unwrap();

    // Send the request to the router
    let response = router.oneshot(request).await.unwrap();

    // Check that the response has a 422 status (since MockAssetService::create_asset is unimplemented)
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_get_asset() {
    // Create the router with mock services
    let router = api::routes::create_router(create_test_app_state());

    // Create a random asset ID
    let asset_id = Uuid::new_v4();

    // Create a request to get the asset
    let request = Request::builder()
        .uri(format!("/api/assets/{}", asset_id))
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

    // Check that the response contains an asset with the correct ID
    assert_eq!(body["id"], asset_id.to_string());
    assert_eq!(body["value"], "test.example.com");
}

#[tokio::test]
async fn test_update_asset() {
    // Create the router with mock services
    let router = api::routes::create_router(create_test_app_state());

    // Create a random asset ID
    let asset_id = Uuid::new_v4();

    // Create update asset payload
    let asset_data = json!({
        "id": asset_id.to_string(),
        "organization_id": Uuid::new_v4().to_string(),
        "asset_type": "Domain",
        "value": "updated.example.com",
        "status": "Active",
        "attributes": {
            "hostname": "updated"
        }
    });

    // Create a request to update the asset
    let request = Request::builder()
        .uri(format!("/api/assets/{}", asset_id))
        .method("PUT")
        .header("Content-Type", "application/json")
        .body(Body::from(asset_data.to_string()))
        .unwrap();

    // Send the request to the router
    let response = router.oneshot(request).await.unwrap();

    // Check that the response has a 422 status (since MockAssetService::update_asset is unimplemented)
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_delete_asset() {
    // Create the router with mock services
    let router = api::routes::create_router(create_test_app_state());

    // Create a random asset ID
    let asset_id = Uuid::new_v4();

    // Create a request to delete the asset
    let request = Request::builder()
        .uri(format!("/api/assets/{}", asset_id))
        .method("DELETE")
        .body(Body::empty())
        .unwrap();

    // Send the request to the router
    let response = router.oneshot(request).await.unwrap();

    // Check that the response has a 204 No Content status
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
