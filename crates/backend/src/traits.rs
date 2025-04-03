use async_trait::async_trait;
use shared::types::{
    AssetStatus, AssetType, JobStatus, JobType, PortStatus, Protocol, Severity, UserRole,
    VulnerabilityStatus, ID,
};

use crate::{
    models::{
        Asset, DiscoveryJob, JobAssetLink, Organization, Port, Technology, User, Vulnerability,
    },
    Result,
};

#[async_trait]
pub trait OrganizationRepository: Send + Sync + 'static {
    async fn create_organization(&self, organization: &Organization) -> Result<Organization>;

    async fn get_organization(&self, id: ID) -> Result<Organization>;

    async fn update_organization(&self, organization: &Organization) -> Result<Organization>;

    async fn delete_organization(&self, id: ID) -> Result<bool>;

    async fn list_organizations(&self, limit: usize, offset: usize) -> Result<Vec<Organization>>;

    async fn count_organizations(&self) -> Result<usize>;
}

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn create_user(&self, user: &User) -> Result<User>;

    async fn get_user(&self, id: ID) -> Result<User>;

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;

    async fn update_user(&self, user: &User) -> Result<User>;

    async fn delete_user(&self, id: ID) -> Result<bool>;

    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;

    async fn list_users(
        &self,
        organization_id: Option<ID>,
        role: Option<UserRole>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<User>>;

    async fn count_users(
        &self,
        organization_id: Option<ID>,
        role: Option<UserRole>,
    ) -> Result<usize>;

    /// Atomically checks if email exists and creates user if it doesn't
    async fn atomic_register_user(&self, email: &str, user: &User) -> Result<User>;
}

#[async_trait]
pub trait UserService: Send + Sync + 'static {
    async fn register_user(
        &self,
        organization_id: &uuid::Uuid,
        email: &str,
        password: &str,
    ) -> Result<User>;

    async fn login_user(&self, email: &str, password: &str) -> Result<User>;
}

#[async_trait]
pub trait AssetRepository: Send + Sync + 'static {
    async fn create_asset(&self, asset: &Asset) -> Result<Asset>;

    async fn get_asset(&self, id: ID) -> Result<Asset>;

    async fn update_asset(&self, asset: &Asset) -> Result<Asset>;

    async fn delete_asset(&self, id: ID) -> Result<bool>;

    async fn list_assets(
        &self,
        organization_id: Option<ID>,
        asset_type: Option<AssetType>,
        status: Option<AssetStatus>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Asset>>;

    async fn count_assets(
        &self,
        organization_id: Option<ID>,
        asset_type: Option<AssetType>,
        status: Option<AssetStatus>,
    ) -> Result<usize>;
}

#[async_trait]
pub trait PortRepository: Send + Sync + 'static {
    async fn create_port(&self, port: &Port) -> Result<Port>;

    async fn get_port(&self, id: ID) -> Result<Port>;

    async fn update_port(&self, port: &Port) -> Result<Port>;

    async fn delete_port(&self, id: ID) -> Result<bool>;

    async fn list_ports(
        &self,
        asset_id: Option<ID>,
        port_number: Option<i32>,
        protocol: Option<Protocol>,
        status: Option<PortStatus>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Port>>;

    async fn count_ports(
        &self,
        asset_id: Option<ID>,
        port_number: Option<i32>,
        protocol: Option<Protocol>,
        status: Option<PortStatus>,
    ) -> Result<usize>;
}

#[async_trait]
pub trait TechnologyRepository: Send + Sync + 'static {
    async fn create_technology(&self, technology: &Technology) -> Result<Technology>;

    async fn get_technology(&self, id: ID) -> Result<Technology>;

    async fn update_technology(&self, technology: &Technology) -> Result<Technology>;

    async fn delete_technology(&self, id: ID) -> Result<bool>;

    async fn list_technologies(
        &self,
        asset_id: Option<ID>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Technology>>;

    async fn count_technologies(&self, asset_id: Option<ID>) -> Result<usize>;
}

#[async_trait]
pub trait VulnerabilityRepository: Send + Sync + 'static {
    async fn create_vulnerability(&self, vulnerability: &Vulnerability) -> Result<Vulnerability>;

    async fn get_vulnerability(&self, id: ID) -> Result<Vulnerability>;

    async fn update_vulnerability(&self, vulnerability: &Vulnerability) -> Result<Vulnerability>;

    async fn delete_vulnerability(&self, id: ID) -> Result<bool>;

    async fn list_vulnerabilities(
        &self,
        asset_id: Option<ID>,
        port_id: Option<ID>,
        severity: Option<Severity>,
        status: Option<VulnerabilityStatus>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Vulnerability>>;

    async fn count_vulnerabilities(
        &self,
        asset_id: Option<ID>,
        port_id: Option<ID>,
        severity: Option<Severity>,
        status: Option<VulnerabilityStatus>,
    ) -> Result<usize>;
}

#[async_trait]
pub trait DiscoveryJobRepository: Send + Sync + 'static {
    async fn create_job(&self, job: &DiscoveryJob) -> Result<DiscoveryJob>;

    async fn get_job(&self, id: ID) -> Result<DiscoveryJob>;

    async fn update_job(&self, job: &DiscoveryJob) -> Result<DiscoveryJob>;

    async fn delete_job(&self, id: ID) -> Result<bool>;

    async fn list_jobs(
        &self,
        organization_id: Option<ID>,
        job_type: Option<JobType>,
        status: Option<JobStatus>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<DiscoveryJob>>;

    async fn count_jobs(
        &self,
        organization_id: Option<ID>,
        job_type: Option<JobType>,
        status: Option<JobStatus>,
    ) -> Result<usize>;

    async fn create_job_asset_link(&self, link: &JobAssetLink) -> Result<JobAssetLink>;

    async fn get_job_assets(&self, job_id: ID) -> Result<Vec<Asset>>;
}

#[async_trait]
pub trait DiscoveryService: Send + Sync + 'static {
    async fn discover_assets(
        &self,
        organization_id: ID,
        domain: &str,
        job_types: Vec<JobType>,
    ) -> Result<DiscoveryJob>;

    async fn scan_asset(&self, asset_id: ID) -> Result<Vec<Vulnerability>>;
}

#[async_trait]
pub trait AssetService: Send + Sync + 'static {
    async fn create_asset(&self, asset: &Asset) -> Result<Asset>;

    async fn get_asset(&self, id: ID) -> Result<Asset>;

    async fn update_asset(&self, asset: &Asset) -> Result<Asset>;

    async fn delete_asset(&self, id: ID) -> Result<bool>;

    async fn list_assets(
        &self,
        organization_id: Option<ID>,
        asset_type: Option<AssetType>,
        status: Option<AssetStatus>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Asset>>;

    async fn count_assets(
        &self,
        organization_id: Option<ID>,
        asset_type: Option<AssetType>,
        status: Option<AssetStatus>,
    ) -> Result<usize>;
}

#[async_trait]
pub trait VulnerabilityService: Send + Sync + 'static {
    async fn create_vulnerability(&self, vulnerability: &Vulnerability) -> Result<Vulnerability>;

    async fn get_vulnerability(&self, id: ID) -> Result<Vulnerability>;

    async fn update_vulnerability(&self, vulnerability: &Vulnerability) -> Result<Vulnerability>;

    async fn delete_vulnerability(&self, id: ID) -> Result<bool>;

    async fn list_vulnerabilities(
        &self,
        asset_id: Option<ID>,
        port_id: Option<ID>,
        severity: Option<Severity>,
        status: Option<VulnerabilityStatus>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Vulnerability>>;

    async fn count_vulnerabilities(
        &self,
        asset_id: Option<ID>,
        port_id: Option<ID>,
        severity: Option<Severity>,
        status: Option<VulnerabilityStatus>,
    ) -> Result<usize>;
}

#[async_trait]
pub trait OrganizationService: Send + Sync + 'static {
    async fn create_organization(&self, organization: &Organization) -> Result<Organization>;
    async fn get_organization(&self, id: ID) -> Result<Organization>;
    async fn update_organization(&self, organization: &Organization) -> Result<Organization>;
    async fn delete_organization(&self, id: ID) -> Result<bool>;
    async fn list_organizations(&self, limit: usize, offset: usize) -> Result<Vec<Organization>>;
    async fn count_organizations(&self) -> Result<usize>;
}
