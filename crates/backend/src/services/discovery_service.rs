use crate::models::JobAssetLink;
use async_trait::async_trait;
use chrono::Utc;
use shared::types::{AssetType, JobStatus, JobType, ID};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::{
    models::{Asset, DiscoveryJob, Vulnerability},
    traits::{AssetRepository, DiscoveryJobRepository, DiscoveryService},
    Result,
};

pub struct DiscoveryServiceImpl {
    asset_repository: Arc<dyn AssetRepository>,
    discovery_job_repository: Arc<dyn DiscoveryJobRepository>,
}

impl DiscoveryServiceImpl {
    pub fn new(
        asset_repository: Arc<dyn AssetRepository>,
        discovery_job_repository: Arc<dyn DiscoveryJobRepository>,
    ) -> Self {
        Self {
            asset_repository,
            discovery_job_repository,
        }
    }

    // Get jobs by status
    pub async fn get_jobs_by_status(
        &self,
        status: JobStatus,
        limit: usize,
    ) -> Result<Vec<DiscoveryJob>> {
        self.discovery_job_repository
            .list_jobs_by_status(status, limit)
            .await
    }

    // Update job
    pub async fn update_job(&self, job: &DiscoveryJob) -> Result<DiscoveryJob> {
        self.discovery_job_repository.update_job(job).await
    }

    // Create a new job
    pub async fn create_job(
        &self,
        organization_id: Uuid,
        job_type: JobType,
        target: Option<String>,
        configuration: Option<serde_json::Value>,
    ) -> Result<DiscoveryJob> {
        let job = DiscoveryJob {
            id: Uuid::new_v4(),
            organization_id,
            job_type,
            status: JobStatus::Pending,
            target,
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            logs: None,
            configuration: configuration.unwrap_or(serde_json::Value::Null),
        };

        self.discovery_job_repository.create_job(&job).await
    }

    async fn discover_domains(&self, organization_id: ID, domain: &str) -> Result<Vec<Asset>> {
        info!("Discovering domains for: {}", domain);

        // This would be implemented with actual domain discovery logic
        // For now, just return a dummy domain asset
        let asset = Asset::new(organization_id, AssetType::Domain, domain.to_string(), None);

        let asset = self.asset_repository.create_asset(&asset).await?;

        Ok(vec![asset])
    }

    async fn discover_subdomains(&self, organization_id: ID, domain: &str) -> Result<Vec<Asset>> {
        info!("Discovering subdomains for: {}", domain);

        // This would be implemented with actual subdomain enumeration logic
        // For now, just return some dummy subdomain assets
        let subdomains = vec![
            format!("www.{}", domain),
            format!("api.{}", domain),
            format!("mail.{}", domain),
        ];

        let mut results = Vec::new();

        // Get the parent domain asset if it exists
        let _parent_assets = self
            .asset_repository
            .list_assets(Some(organization_id), Some(AssetType::Domain), None, 1, 0)
            .await?;

        // Create and save subdomain assets
        for subdomain in subdomains {
            let asset = Asset::new(
                organization_id,
                AssetType::Domain, // Using Domain type since Subdomain doesn't exist
                subdomain,
                None,
            );

            let asset = self.asset_repository.create_asset(&asset).await?;
            results.push(asset);
        }

        Ok(results)
    }

    /// Helper method to discover ports and services
    async fn discover_ports(&self, organization_id: ID, asset_id: ID) -> Result<Vec<Asset>> {
        info!("Discovering ports for asset ID: {}", asset_id);

        // This would be implemented with actual port scanning logic
        // For now, just return some dummy port assets
        let ports = vec![80, 443, 22];
        let asset = self.asset_repository.get_asset(asset_id).await?;

        let mut results = Vec::new();

        // Create and save port assets
        for port in ports {
            let asset_name = format!("{}:{}", asset.value, port);
            let service_name = match port {
                80 => "HTTP",
                443 => "HTTPS",
                22 => "SSH",
                _ => "Unknown",
            };

            // For now, we'll use WebApp type for ports since there's no Port type
            let port_asset = Asset::new(
                organization_id,
                AssetType::WebApp, // Using WebApp since Port doesn't exist
                asset_name,
                None,
            );

            let port_asset = self.asset_repository.create_asset(&port_asset).await?;

            // For services, we'll also use WebApp type
            let service_asset = Asset::new(
                organization_id,
                AssetType::WebApp, // Using WebApp since Service doesn't exist
                format!("{} ({})", service_name, asset.value),
                None,
            );

            let service_asset = self.asset_repository.create_asset(&service_asset).await?;

            results.push(port_asset);
            results.push(service_asset);
        }

        Ok(results)
    }
}

#[async_trait]
impl DiscoveryService for DiscoveryServiceImpl {
    async fn discover_assets(
        &self,
        organization_id: ID,
        domain: &str,
        job_types: Vec<JobType>,
    ) -> Result<DiscoveryJob> {
        info!("Starting asset discovery for domain: {}", domain);

        // Create a job record for this discovery request
        let job = self
            .create_job(
                organization_id,
                JobType::DnsEnum,
                Some(domain.to_string()),
                None,
            )
            .await?;

        let mut discovered_assets = Vec::new();

        // Domain discovery - map JobType to our internal DiscoveryMethod
        if job_types.contains(&JobType::DnsEnum) || job_types.is_empty() {
            let assets = self.discover_domains(organization_id, domain).await?;
            discovered_assets.extend(assets);
        }

        // Subdomain enumeration
        if job_types.contains(&JobType::DnsEnum) || job_types.is_empty() {
            let assets = self.discover_subdomains(organization_id, domain).await?;
            discovered_assets.extend(assets);
        }

        // Port scanning for each discovered asset
        if job_types.contains(&JobType::PortScan) || job_types.is_empty() {
            let assets_to_scan = discovered_assets.clone();

            for asset in assets_to_scan {
                if asset.asset_type == AssetType::Domain {
                    let port_assets = self.discover_ports(organization_id, asset.id).await?;
                    discovered_assets.extend(port_assets);
                }
            }
        }

        info!("Discovered {} assets", discovered_assets.len());

        // Update the job status
        let mut updated_job = job.clone();
        updated_job.status = JobStatus::Completed;
        updated_job.completed_at = Some(Utc::now());
        let updated_job = self
            .discovery_job_repository
            .update_job(&updated_job)
            .await?;

        Ok(updated_job)
    }

    async fn scan_asset(&self, asset_id: ID) -> Result<Vec<Vulnerability>> {
        info!("Scanning asset with ID: {} for vulnerabilities", asset_id);

        // This would be implemented with actual vulnerability scanning logic
        // For now, just return an empty list of vulnerabilities
        Ok(Vec::new())
    }
}

// Basic tests for DiscoveryServiceImpl
#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{AssetRepository, DiscoveryJobRepository};
    use mockall::mock;
    use mockall::predicate::*;
    use shared::types::{AssetStatus, AssetType, JobStatus, JobType};
    use uuid::Uuid;

    mock! {
        AssetRepo {}
        #[async_trait]
        impl AssetRepository for AssetRepo {
            async fn create_asset(&self, asset: &Asset) -> Result<Asset>;
            async fn get_asset(&self, id: Uuid) -> Result<Asset>;
            async fn update_asset(&self, asset: &Asset) -> Result<Asset>;
            async fn delete_asset(&self, id: Uuid) -> Result<bool>;
            async fn list_assets(
                &self,
                organization_id: Option<Uuid>,
                asset_type: Option<AssetType>,
                status: Option<AssetStatus>,
                limit: usize,
                offset: usize,
            ) -> Result<Vec<Asset>>;
            async fn count_assets(
                &self,
                organization_id: Option<Uuid>,
                asset_type: Option<AssetType>,
                status: Option<AssetStatus>,
            ) -> Result<usize>;
        }
    }

    mock! {
        DiscoveryJobRepo {}
        #[async_trait]
        impl DiscoveryJobRepository for DiscoveryJobRepo {
            async fn create_job(&self, job: &DiscoveryJob) -> Result<DiscoveryJob>;
            async fn get_job(&self, id: Uuid) -> Result<DiscoveryJob>;
            async fn update_job(&self, job: &DiscoveryJob) -> Result<DiscoveryJob>;
            async fn delete_job(&self, id: Uuid) -> Result<bool>;
            async fn list_jobs(
                &self,
                organization_id: Option<Uuid>,
                job_type: Option<JobType>,
                status: Option<JobStatus>,
                limit: usize,
                offset: usize,
            ) -> Result<Vec<DiscoveryJob>>;
            async fn count_jobs(
                &self,
                organization_id: Option<Uuid>,
                job_type: Option<JobType>,
                status: Option<JobStatus>,
            ) -> Result<usize>;
            async fn list_jobs_by_status(
                &self,
                status: JobStatus,
                limit: usize,
            ) -> Result<Vec<DiscoveryJob>>;
            async fn create_job_asset_link(&self, link: &JobAssetLink) -> Result<JobAssetLink>;
            async fn get_job_assets(&self, job_id: Uuid) -> Result<Vec<Asset>>;
        }
    }

    #[tokio::test]
    async fn test_get_jobs_by_status() {
        let mock_asset_repo = MockAssetRepo::new();
        let mut mock_job_repo = MockDiscoveryJobRepo::new();

        let expected_jobs = vec![DiscoveryJob::new(
            Uuid::new_v4(),
            JobType::DnsEnum,
            Some("example.com".to_string()),
            None,
        )];

        mock_job_repo
            .expect_list_jobs_by_status()
            .with(eq(JobStatus::Pending), eq(10))
            .times(1)
            .returning(move |_, _| Ok(expected_jobs.clone()));

        let service = DiscoveryServiceImpl::new(Arc::new(mock_asset_repo), Arc::new(mock_job_repo));

        let jobs = service
            .get_jobs_by_status(JobStatus::Pending, 10)
            .await
            .unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].job_type, JobType::DnsEnum);
    }
}
