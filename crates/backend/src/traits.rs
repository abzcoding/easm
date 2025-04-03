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

use std::collections::HashMap;

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
        name: Option<String>,
        category: Option<String>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Technology>>;

    async fn count_technologies(
        &self,
        asset_id: Option<ID>,
        name: Option<String>,
        category: Option<String>,
    ) -> Result<usize>;
}

#[async_trait]
pub trait TechnologyService: Send + Sync + 'static {
    async fn create_technology(&self, technology: &Technology) -> Result<Technology>;

    async fn get_technology(&self, id: ID) -> Result<Technology>;

    async fn update_technology(&self, technology: &Technology) -> Result<Technology>;

    async fn delete_technology(&self, id: ID) -> Result<bool>;

    async fn list_technologies(
        &self,
        asset_id: Option<ID>,
        name: Option<String>,
        category: Option<String>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Technology>>;

    async fn count_technologies(
        &self,
        asset_id: Option<ID>,
        name: Option<String>,
        category: Option<String>,
    ) -> Result<usize>;

    /// Get technology statistics for an organization
    async fn get_technology_statistics(
        &self,
        organization_id: ID,
    ) -> Result<std::collections::HashMap<String, usize>>;
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

    /// Create a relationship between two assets
    async fn create_asset_relationship(
        &self,
        source_asset_id: ID,
        target_asset_id: ID,
        relationship_type: String,
        metadata: Option<serde_json::Value>,
    ) -> Result<bool>;

    /// Delete a relationship between two assets
    async fn delete_asset_relationship(
        &self,
        source_asset_id: ID,
        target_asset_id: ID,
        relationship_type: String,
    ) -> Result<bool>;

    /// Get all related assets for a given asset
    async fn get_related_assets(
        &self,
        asset_id: ID,
        relationship_type: Option<String>,
    ) -> Result<Vec<(Asset, String)>>;

    /// Find potential relationships between assets
    async fn discover_asset_relationships(
        &self,
        organization_id: ID,
    ) -> Result<Vec<(ID, ID, String)>>;
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

    /// Correlate vulnerabilities across assets based on common patterns
    /// Returns a map of vulnerability IDs to related vulnerability IDs
    async fn correlate_vulnerabilities(
        &self,
        organization_id: ID,
        min_severity: Option<Severity>,
    ) -> Result<std::collections::HashMap<ID, Vec<ID>>>;

    /// Find similar vulnerabilities for a specific vulnerability
    async fn find_similar_vulnerabilities(
        &self,
        vulnerability_id: ID,
        limit: usize,
    ) -> Result<Vec<Vulnerability>>;

    /// Update vulnerability status in bulk for easier management
    async fn bulk_update_vulnerability_status(
        &self,
        vulnerability_ids: Vec<ID>,
        status: VulnerabilityStatus,
    ) -> Result<usize>;

    /// Get vulnerability statistics for an organization
    async fn get_vulnerability_statistics(
        &self,
        organization_id: ID,
    ) -> Result<std::collections::HashMap<String, usize>>;
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

/// Service for sending notifications about findings and changes
#[async_trait]
pub trait NotificationService: Send + Sync + 'static {
    /// Send a notification about a new vulnerability
    async fn notify_new_vulnerability(&self, vulnerability: &Vulnerability) -> Result<bool>;

    /// Send a notification about a vulnerability status change
    async fn notify_vulnerability_status_change(
        &self,
        vulnerability: &Vulnerability,
        old_status: VulnerabilityStatus,
    ) -> Result<bool>;

    /// Send a notification about a new critical asset discovery
    async fn notify_new_critical_asset(&self, asset: &Asset) -> Result<bool>;

    /// Send a summary report notification
    async fn send_summary_report(
        &self,
        organization_id: ID,
        period: NotificationPeriod,
    ) -> Result<bool>;

    /// Get notification settings for an organization
    async fn get_notification_settings(&self, organization_id: ID) -> Result<NotificationSettings>;

    /// Update notification settings for an organization
    async fn update_notification_settings(
        &self,
        organization_id: ID,
        settings: &NotificationSettings,
    ) -> Result<NotificationSettings>;
}

/// Period for notification reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPeriod {
    Daily,
    Weekly,
    Monthly,
}

/// Settings for notifications
#[derive(Debug, Clone)]
pub struct NotificationSettings {
    pub organization_id: ID,
    pub email_notifications: bool,
    pub email_recipients: Vec<String>,
    pub webhook_notifications: bool,
    pub webhook_url: Option<String>,
    pub notification_period: NotificationPeriod,
    pub notify_on_new_vulnerability: bool,
    pub notify_on_status_change: bool,
    pub notify_on_new_critical_asset: bool,
    pub minimum_severity_for_notification: Severity,
    pub additional_settings: Option<HashMap<String, serde_json::Value>>,
}
