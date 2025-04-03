use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

use crate::{handlers::auth_handler::AuthResponseDto, state::AppState};
use backend::{
    models::{Asset, Organization, User, Vulnerability},
    Result,
};
use shared::{
    config::Config,
    types::{
        AssetStatus, AssetType, JobStatus, JobType, Severity, UserRole, VulnerabilityStatus, ID,
    },
};
use uuid::Uuid;

#[derive(Clone)]
pub struct MockAssetService;

#[derive(Clone)]
pub struct MockUserService;

#[derive(Clone)]
pub struct MockOrganizationService;

#[derive(Clone)]
pub struct MockDiscoveryService;

#[async_trait]
impl backend::UserService for MockUserService {
    async fn register_user(
        &self,
        organization_id: &Uuid,
        email: &str,
        _password: &str,
    ) -> Result<User> {
        // In tests, we'll consider any organization ID valid
        // and we'll ignore email uniqueness

        // Return a mock user
        let now = chrono::Utc::now();
        Ok(User {
            id: Uuid::new_v4(),
            organization_id: *organization_id,
            username: email.split('@').next().unwrap_or("testuser").to_string(),
            email: email.to_string(),
            password_hash: "hashed_password".to_string(), // Mock hash
            role: UserRole::Analyst,                      // Default role
            created_at: now,
            updated_at: now,
        })
    }

    async fn login_user(&self, email: &str, _password: &str) -> Result<User> {
        // Return a mock user, assuming login is successful for the test email
        let now = chrono::Utc::now();
        Ok(User {
            id: Uuid::new_v4(), // Use a consistent ID if needed across tests, or generate new
            organization_id: Uuid::new_v4(), // Mock organization ID
            username: email.split('@').next().unwrap_or("testuser").to_string(),
            email: email.to_string(),
            password_hash: "hashed_password".to_string(),
            role: UserRole::Analyst,
            created_at: now,
            updated_at: now,
        })
    }
}

#[async_trait]
impl backend::OrganizationService for MockOrganizationService {
    async fn create_organization(&self, _organization: &Organization) -> Result<Organization> {
        unimplemented!()
    }

    async fn get_organization(&self, _id: ID) -> Result<Organization> {
        unimplemented!()
    }
    async fn update_organization(&self, _organization: &Organization) -> Result<Organization> {
        unimplemented!()
    }

    async fn delete_organization(&self, _id: ID) -> Result<bool> {
        unimplemented!()
    }

    async fn list_organizations(&self, _limit: usize, _offset: usize) -> Result<Vec<Organization>> {
        unimplemented!()
    }
    async fn count_organizations(&self) -> Result<usize> {
        unimplemented!()
    }
}

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
    async fn create_vulnerability(&self, vulnerability: &Vulnerability) -> Result<Vulnerability> {
        // Return a valid vulnerability with id
        let now = chrono::Utc::now();
        Ok(Vulnerability {
            id: Uuid::new_v4(),
            asset_id: vulnerability.asset_id,
            port_id: vulnerability.port_id,
            title: vulnerability.title.clone(),
            description: vulnerability.description.clone(),
            severity: vulnerability.severity,
            status: VulnerabilityStatus::Open,
            cve_id: vulnerability.cve_id.clone(),
            cvss_score: vulnerability.cvss_score,
            evidence: vulnerability.evidence.clone(),
            remediation: vulnerability.remediation.clone(),
            first_seen: now,
            last_seen: now,
            resolved_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    async fn get_vulnerability(&self, id: ID) -> Result<Vulnerability> {
        // Return a valid vulnerability with the requested id
        let now = chrono::Utc::now();
        Ok(Vulnerability {
            id,
            asset_id: Uuid::new_v4(),
            port_id: Some(Uuid::new_v4()),
            title: "Test Vulnerability".to_string(),
            description: Some("A test vulnerability".to_string()),
            severity: Severity::High,
            status: VulnerabilityStatus::Open,
            cve_id: Some("CVE-2023-1234".to_string()),
            cvss_score: Some(7.5),
            evidence: serde_json::json!({
                "exploit_available": true,
                "references": ["https://example.com/vuln1"]
            }),
            remediation: Some("Apply security patch".to_string()),
            first_seen: now,
            last_seen: now,
            resolved_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    async fn update_vulnerability(&self, vulnerability: &Vulnerability) -> Result<Vulnerability> {
        // Return the updated vulnerability
        let now = chrono::Utc::now();
        Ok(Vulnerability {
            id: vulnerability.id,
            asset_id: vulnerability.asset_id,
            port_id: vulnerability.port_id,
            title: vulnerability.title.clone(),
            description: vulnerability.description.clone(),
            severity: vulnerability.severity,
            status: vulnerability.status,
            cve_id: vulnerability.cve_id.clone(),
            cvss_score: vulnerability.cvss_score,
            evidence: vulnerability.evidence.clone(),
            remediation: vulnerability.remediation.clone(),
            first_seen: vulnerability.first_seen,
            last_seen: now,
            resolved_at: vulnerability.resolved_at,
            created_at: vulnerability.created_at,
            updated_at: now,
        })
    }

    async fn delete_vulnerability(&self, _id: ID) -> Result<bool> {
        // Always return success
        Ok(true)
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
        // Return a list of test vulnerabilities
        let now = chrono::Utc::now();
        Ok(vec![
            Vulnerability {
                id: Uuid::new_v4(),
                asset_id: Uuid::new_v4(),
                port_id: Some(Uuid::new_v4()),
                title: "Test Vulnerability 1".to_string(),
                description: Some("A test vulnerability".to_string()),
                severity: Severity::High,
                status: VulnerabilityStatus::Open,
                cve_id: Some("CVE-2023-1234".to_string()),
                cvss_score: Some(7.5),
                evidence: serde_json::json!({
                    "exploit_available": true,
                    "references": ["https://example.com/vuln1"]
                }),
                remediation: Some("Apply security patch".to_string()),
                first_seen: now,
                last_seen: now,
                resolved_at: None,
                created_at: now,
                updated_at: now,
            },
            Vulnerability {
                id: Uuid::new_v4(),
                asset_id: Uuid::new_v4(),
                port_id: None,
                title: "Test Vulnerability 2".to_string(),
                description: Some("Another test vulnerability".to_string()),
                severity: Severity::Medium,
                status: VulnerabilityStatus::Open,
                cve_id: None,
                cvss_score: None,
                evidence: serde_json::json!({
                    "exploit_available": false,
                    "references": []
                }),
                remediation: None,
                first_seen: now,
                last_seen: now,
                resolved_at: None,
                created_at: now,
                updated_at: now,
            },
        ])
    }

    async fn count_vulnerabilities(
        &self,
        _asset_id: Option<ID>,
        _port_id: Option<ID>,
        _severity: Option<Severity>,
        _status: Option<VulnerabilityStatus>,
    ) -> Result<usize> {
        // Return a fixed count
        Ok(2)
    }

    async fn correlate_vulnerabilities(
        &self,
        _organization_id: ID,
        _min_severity: Option<Severity>,
    ) -> Result<std::collections::HashMap<ID, Vec<ID>>> {
        // Return mock correlation data
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        let mut correlations = std::collections::HashMap::new();
        correlations.insert(id1, vec![id2]);
        correlations.insert(id2, vec![id1, id3]);
        correlations.insert(id3, vec![id2]);

        Ok(correlations)
    }

    async fn find_similar_vulnerabilities(
        &self,
        _vulnerability_id: ID,
        _limit: usize,
    ) -> Result<Vec<Vulnerability>> {
        // Return a list of mock similar vulnerabilities
        let now = chrono::Utc::now();
        let asset_id = Uuid::new_v4();

        Ok(vec![Vulnerability {
            id: Uuid::new_v4(),
            asset_id,
            port_id: Some(Uuid::new_v4()),
            title: "Similar SQL Injection".to_string(),
            description: Some("A similar SQL injection vulnerability".to_string()),
            severity: Severity::High,
            status: VulnerabilityStatus::Open,
            cve_id: Some("CVE-2022-1234".to_string()),
            cvss_score: Some(8.5),
            evidence: serde_json::json!({
                "request": "GET /search?q=1' OR 1=1",
                "response": "Database error"
            }),
            remediation: Some("Parameterize SQL queries".to_string()),
            first_seen: now,
            last_seen: now,
            resolved_at: None,
            created_at: now,
            updated_at: now,
        }])
    }
}

#[async_trait]
impl backend::DiscoveryService for MockDiscoveryService {
    async fn discover_assets(
        &self,
        organization_id: ID,
        domain: &str,
        _job_types: Vec<JobType>,
    ) -> Result<backend::models::DiscoveryJob> {
        let now = chrono::Utc::now();
        Ok(backend::models::DiscoveryJob {
            id: Uuid::new_v4(),
            organization_id,
            job_type: JobType::DnsEnum,
            status: JobStatus::Completed,
            target: Some(domain.to_string()),
            configuration: serde_json::json!({
                "domain": domain
            }),
            logs: None,
            created_at: now,
            updated_at: now,
            started_at: Some(now),
            completed_at: Some(now),
        })
    }

    async fn scan_asset(&self, asset_id: ID) -> Result<Vec<Vulnerability>> {
        let now = chrono::Utc::now();
        Ok(vec![Vulnerability {
            id: Uuid::new_v4(),
            asset_id,
            port_id: Some(Uuid::new_v4()),
            title: "Mock Vulnerability".to_string(),
            description: Some("Found during mock scan".to_string()),
            severity: Severity::Medium,
            status: VulnerabilityStatus::Open,
            cve_id: Some("CVE-2024-MOCK".to_string()),
            cvss_score: Some(5.0),
            evidence: serde_json::json!({ "details": "Mock scan evidence" }),
            remediation: Some("Apply mock patch".to_string()),
            first_seen: now,
            last_seen: now,
            resolved_at: None,
            created_at: now,
            updated_at: now,
        }])
    }
}

// Helper function to create test state
pub fn create_test_app_state() -> AppState {
    // Load config first to get DB URL if needed for lazy pool
    let config = Config::from_env().expect("Failed to load config for test state");
    // Create a lazy PgPool (it won't connect until first use, which it won't in mocks)
    let db_pool = sqlx::postgres::PgPool::connect_lazy(&config.database_url)
        .expect("Failed to create lazy DB pool for tests");

    AppState {
        config,
        db_pool,            // db_pool is still part of AppState
        redis_client: None, // Assuming redis is optional
        // Assign mock services directly to the state fields
        asset_service: std::sync::Arc::new(MockAssetService),
        vulnerability_service: std::sync::Arc::new(MockVulnerabilityService),
        organization_service: std::sync::Arc::new(MockOrganizationService),
        discovery_service: std::sync::Arc::new(MockDiscoveryService),
        user_service: std::sync::Arc::new(MockUserService),
        // Remove repository_factory field
    }
}

// Helper function to authenticate a test user and get a token
pub async fn authenticate_test_user(router: &Router) -> String {
    let org_id = Uuid::new_v4();
    let user_email = format!("testuser_{}@example.com", Uuid::new_v4());
    let user_password = "password123".to_string();

    // --- Register User ---
    let register_payload = json!({
        "organization_id": org_id.to_string(),
        "email": user_email,
        "password": user_password
    });

    let register_request = Request::builder()
        .uri("/api/auth/register")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(register_payload.to_string()))
        .unwrap();

    let register_response = router.clone().oneshot(register_request).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::CREATED);

    // --- Login User ---
    let login_payload = json!({
        "email": user_email,
        "password": user_password
    });

    let login_request = Request::builder()
        .uri("/api/auth/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(login_payload.to_string()))
        .unwrap();

    let login_response = router.clone().oneshot(login_request).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    // Extract token from login response
    let body = login_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let auth_response: AuthResponseDto = serde_json::from_slice(&body).unwrap();

    auth_response.token
}
