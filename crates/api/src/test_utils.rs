use async_trait::async_trait;

use backend::{
    models::{Asset, Organization, User, Vulnerability},
    Result,
};
use shared::config::Config;
use shared::types::{
    AssetStatus, AssetType, JobStatus, JobType, Severity, VulnerabilityStatus, ID,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct MockAssetService;

#[derive(Clone)]
pub struct MockUserService;

#[derive(Clone)]
pub struct MockOrganizationService;

#[async_trait]
impl backend::UserService for MockUserService {
    async fn register_user(
        &self,
        _organization_id: &Uuid,
        _email: &str,
        _password: &str,
    ) -> Result<User> {
        unimplemented!()
    }

    async fn login_user(&self, _email: &str, _password: &str) -> Result<User> {
        unimplemented!()
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
    ) -> Result<backend::models::DiscoveryJob> {
        // Return a mock discovery job
        let now = chrono::Utc::now();
        Ok(backend::models::DiscoveryJob {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            job_type: JobType::DnsEnum,
            status: JobStatus::Completed,
            target: Some("example.com".to_string()),
            configuration: serde_json::json!({
                "domain": "example.com"
            }),
            logs: None,
            created_at: now,
            updated_at: now,
            started_at: Some(now),
            completed_at: Some(now),
        })
    }

    async fn scan_asset(&self, _asset_id: ID) -> Result<Vec<Vulnerability>> {
        // Return a list of mock vulnerabilities
        let now = chrono::Utc::now();
        Ok(vec![Vulnerability {
            id: Uuid::new_v4(),
            asset_id: _asset_id,
            port_id: Some(Uuid::new_v4()),
            title: "Found Vulnerability 1".to_string(),
            description: Some("A vulnerability found during scanning".to_string()),
            severity: Severity::High,
            status: VulnerabilityStatus::Open,
            cve_id: Some("CVE-2023-5678".to_string()),
            cvss_score: Some(8.2),
            evidence: serde_json::json!({
                "exploit_available": true,
                "references": ["https://example.com/vuln-scan1"]
            }),
            remediation: Some("Apply security patch".to_string()),
            first_seen: now,
            last_seen: now,
            resolved_at: None,
            created_at: now,
            updated_at: now,
        }])
    }
}
pub fn create_test_app_state() -> crate::state::AppState {
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

    crate::state::AppState {
        config: config.clone(),
        db_pool: sqlx::postgres::PgPool::connect_lazy(&config.database_url).unwrap(),
        redis_client: None,
        asset_service: std::sync::Arc::new(MockAssetService),
        vulnerability_service: std::sync::Arc::new(MockVulnerabilityService),
        discovery_service: std::sync::Arc::new(MockDiscoveryService),
        user_service: std::sync::Arc::new(MockUserService),
        organization_service: std::sync::Arc::new(MockOrganizationService),
    }
}
