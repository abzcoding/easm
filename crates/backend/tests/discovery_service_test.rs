#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use backend::models::Vulnerability;
    use backend::models::{Asset, DiscoveryJob, JobAssetLink};
    use backend::{
        AssetRepository, DiscoveryJobRepository, DiscoveryService, Error, Result,
        VulnerabilityRepository,
    };
    use shared::types::{AssetStatus, Severity, VulnerabilityStatus};
    use shared::types::{AssetType, JobStatus, JobType, ID};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tokio::test;
    use uuid::Uuid;

    #[derive(Clone)]
    struct MockDiscoveryJobRepository {
        jobs: Arc<Mutex<HashMap<ID, DiscoveryJob>>>,
        assets: Arc<Mutex<HashMap<ID, Asset>>>,
        links: Arc<Mutex<Vec<JobAssetLink>>>,
    }

    impl MockDiscoveryJobRepository {
        fn new() -> Self {
            Self {
                jobs: Arc::new(Mutex::new(HashMap::new())),
                assets: Arc::new(Mutex::new(HashMap::new())),
                links: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn add_asset(&self, asset: Asset) {
            let mut assets = self.assets.lock().unwrap();
            assets.insert(asset.id, asset);
        }
    }

    #[async_trait]
    impl DiscoveryJobRepository for MockDiscoveryJobRepository {
        async fn create_job(&self, job: &DiscoveryJob) -> Result<DiscoveryJob> {
            let mut jobs = self.jobs.lock().unwrap();
            let new_job = job.clone();
            jobs.insert(new_job.id, new_job.clone());
            Ok(new_job)
        }

        async fn get_job(&self, id: ID) -> Result<DiscoveryJob> {
            let jobs = self.jobs.lock().unwrap();
            jobs.get(&id)
                .cloned()
                .ok_or_else(|| Error::NotFound(format!("DiscoveryJob with ID {} not found", id)))
        }

        async fn update_job(&self, job: &DiscoveryJob) -> Result<DiscoveryJob> {
            let mut jobs = self.jobs.lock().unwrap();
            if !jobs.contains_key(&job.id) {
                return Err(Error::NotFound(format!(
                    "DiscoveryJob with ID {} not found",
                    job.id
                )));
            }
            let updated_job = job.clone();
            jobs.insert(updated_job.id, updated_job.clone());
            Ok(updated_job)
        }

        async fn delete_job(&self, id: ID) -> Result<bool> {
            let mut jobs = self.jobs.lock().unwrap();
            if !jobs.contains_key(&id) {
                return Err(Error::NotFound(format!(
                    "DiscoveryJob with ID {} not found",
                    id
                )));
            }
            jobs.remove(&id);
            Ok(true)
        }

        async fn list_jobs(
            &self,
            organization_id: Option<ID>,
            job_type: Option<JobType>,
            status: Option<JobStatus>,
            limit: usize,
            offset: usize,
        ) -> Result<Vec<DiscoveryJob>> {
            let jobs = self.jobs.lock().unwrap();
            let filtered: Vec<DiscoveryJob> = jobs
                .values()
                .filter(|j| {
                    organization_id.is_none_or(|oid| j.organization_id == oid)
                        && job_type.is_none_or(|jt| j.job_type == jt)
                        && status.is_none_or(|s| j.status == s)
                })
                .cloned()
                .collect();

            let paginated = filtered.into_iter().skip(offset).take(limit).collect();
            Ok(paginated)
        }

        async fn count_jobs(
            &self,
            organization_id: Option<ID>,
            job_type: Option<JobType>,
            status: Option<JobStatus>,
        ) -> Result<usize> {
            let jobs = self.jobs.lock().unwrap();
            let count = jobs
                .values()
                .filter(|j| {
                    organization_id.is_none_or(|oid| j.organization_id == oid)
                        && job_type.is_none_or(|jt| j.job_type == jt)
                        && status.is_none_or(|s| j.status == s)
                })
                .count();
            Ok(count)
        }

        async fn create_job_asset_link(&self, link: &JobAssetLink) -> Result<JobAssetLink> {
            let mut links = self.links.lock().unwrap();
            let new_link = link.clone();
            links.push(new_link.clone());
            Ok(new_link)
        }

        async fn get_job_assets(&self, job_id: ID) -> Result<Vec<Asset>> {
            let links = self.links.lock().unwrap();
            let assets = self.assets.lock().unwrap();

            let asset_ids: Vec<ID> = links
                .iter()
                .filter(|link| link.job_id == job_id)
                .map(|link| link.asset_id)
                .collect();

            let job_assets: Vec<Asset> = asset_ids
                .iter()
                .filter_map(|id| assets.get(id).cloned())
                .collect();

            Ok(job_assets)
        }

        async fn list_jobs_by_status(
            &self,
            status: JobStatus,
            limit: usize,
        ) -> Result<Vec<DiscoveryJob>> {
            let jobs = self.jobs.lock().unwrap();
            let filtered: Vec<DiscoveryJob> = jobs
                .values()
                .filter(|j| j.status == status)
                .take(limit)
                .cloned()
                .collect();

            Ok(filtered)
        }
    }

    #[derive(Clone)]
    struct MockAssetRepository {
        assets: Arc<Mutex<HashMap<ID, Asset>>>,
    }

    impl MockAssetRepository {
        fn new() -> Self {
            Self {
                assets: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl AssetRepository for MockAssetRepository {
        async fn create_asset(&self, asset: &Asset) -> Result<Asset> {
            let mut assets = self.assets.lock().unwrap();
            let new_asset = asset.clone();
            assets.insert(new_asset.id, new_asset.clone());
            Ok(new_asset)
        }

        async fn get_asset(&self, id: ID) -> Result<Asset> {
            let assets = self.assets.lock().unwrap();
            assets
                .get(&id)
                .cloned()
                .ok_or_else(|| Error::NotFound(format!("Asset with ID {} not found", id)))
        }

        async fn update_asset(&self, asset: &Asset) -> Result<Asset> {
            let mut assets = self.assets.lock().unwrap();

            if !assets.contains_key(&asset.id) {
                return Err(Error::NotFound(format!(
                    "Asset with ID {} not found",
                    asset.id
                )));
            }

            let updated_asset = asset.clone();
            assets.insert(updated_asset.id, updated_asset.clone());
            Ok(updated_asset)
        }

        async fn delete_asset(&self, id: ID) -> Result<bool> {
            let mut assets = self.assets.lock().unwrap();

            if !assets.contains_key(&id) {
                return Err(Error::NotFound(format!("Asset with ID {} not found", id)));
            }

            assets.remove(&id);
            Ok(true)
        }

        async fn list_assets(
            &self,
            organization_id: Option<ID>,
            asset_type: Option<AssetType>,
            status: Option<AssetStatus>,
            limit: usize,
            offset: usize,
        ) -> Result<Vec<Asset>> {
            let assets = self.assets.lock().unwrap();

            let filtered: Vec<Asset> = assets
                .values()
                .filter(|a| {
                    organization_id.is_none_or(|oid| a.organization_id == oid)
                        && asset_type.is_none_or(|at| a.asset_type == at)
                        && status.is_none_or(|s| a.status == s)
                })
                .cloned()
                .collect();

            let paginated = filtered.into_iter().skip(offset).take(limit).collect();

            Ok(paginated)
        }

        async fn count_assets(
            &self,
            organization_id: Option<ID>,
            asset_type: Option<AssetType>,
            status: Option<AssetStatus>,
        ) -> Result<usize> {
            let assets = self.assets.lock().unwrap();

            let count = assets
                .values()
                .filter(|a| {
                    organization_id.is_none_or(|oid| a.organization_id == oid)
                        && asset_type.is_none_or(|at| a.asset_type == at)
                        && status.is_none_or(|s| a.status == s)
                })
                .count();

            Ok(count)
        }
    }

    #[derive(Clone)]
    struct MockVulnerabilityRepository {
        vulnerabilities: Arc<Mutex<HashMap<ID, Vulnerability>>>,
    }

    impl MockVulnerabilityRepository {
        fn new() -> Self {
            Self {
                vulnerabilities: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl VulnerabilityRepository for MockVulnerabilityRepository {
        async fn create_vulnerability(
            &self,
            vulnerability: &Vulnerability,
        ) -> Result<Vulnerability> {
            let mut vulnerabilities = self.vulnerabilities.lock().unwrap();
            let new_vulnerability = vulnerability.clone();
            vulnerabilities.insert(new_vulnerability.id, new_vulnerability.clone());
            Ok(new_vulnerability)
        }

        async fn get_vulnerability(&self, id: ID) -> Result<Vulnerability> {
            let vulnerabilities = self.vulnerabilities.lock().unwrap();
            vulnerabilities
                .get(&id)
                .cloned()
                .ok_or_else(|| Error::NotFound(format!("Vulnerability with ID {} not found", id)))
        }

        async fn update_vulnerability(
            &self,
            vulnerability: &Vulnerability,
        ) -> Result<Vulnerability> {
            let mut vulnerabilities = self.vulnerabilities.lock().unwrap();

            if !vulnerabilities.contains_key(&vulnerability.id) {
                return Err(Error::NotFound(format!(
                    "Vulnerability with ID {} not found",
                    vulnerability.id
                )));
            }

            let updated_vulnerability = vulnerability.clone();
            vulnerabilities.insert(updated_vulnerability.id, updated_vulnerability.clone());
            Ok(updated_vulnerability)
        }

        async fn delete_vulnerability(&self, id: ID) -> Result<bool> {
            let mut vulnerabilities = self.vulnerabilities.lock().unwrap();

            if !vulnerabilities.contains_key(&id) {
                return Err(Error::NotFound(format!(
                    "Vulnerability with ID {} not found",
                    id
                )));
            }

            vulnerabilities.remove(&id);
            Ok(true)
        }

        async fn list_vulnerabilities(
            &self,
            asset_id: Option<ID>,
            port_id: Option<ID>,
            severity: Option<Severity>,
            status: Option<VulnerabilityStatus>,
            limit: usize,
            offset: usize,
        ) -> Result<Vec<Vulnerability>> {
            let vulnerabilities = self.vulnerabilities.lock().unwrap();

            let filtered: Vec<Vulnerability> = vulnerabilities
                .values()
                .filter(|v| {
                    asset_id.is_none_or(|aid| v.asset_id == aid)
                        && port_id.is_none_or(|pid| v.port_id.as_ref().is_some_and(|id| *id == pid))
                        && severity.is_none_or(|s| v.severity == s)
                        && status.is_none_or(|s| v.status == s)
                })
                .cloned()
                .collect();

            let paginated = filtered.into_iter().skip(offset).take(limit).collect();

            Ok(paginated)
        }

        async fn count_vulnerabilities(
            &self,
            asset_id: Option<ID>,
            port_id: Option<ID>,
            severity: Option<Severity>,
            status: Option<VulnerabilityStatus>,
        ) -> Result<usize> {
            let vulnerabilities = self.vulnerabilities.lock().unwrap();

            let count = vulnerabilities
                .values()
                .filter(|v| {
                    asset_id.is_none_or(|aid| v.asset_id == aid)
                        && port_id.is_none_or(|pid| v.port_id.as_ref().is_some_and(|id| *id == pid))
                        && severity.is_none_or(|s| v.severity == s)
                        && status.is_none_or(|s| v.status == s)
                })
                .count();

            Ok(count)
        }
    }

    // Simplified Discovery Service implementation for testing
    struct TestDiscoveryService {
        job_repo: MockDiscoveryJobRepository,
        asset_repo: MockAssetRepository,
        vuln_repo: MockVulnerabilityRepository,
    }

    impl TestDiscoveryService {
        fn new(
            job_repo: MockDiscoveryJobRepository,
            asset_repo: MockAssetRepository,
            vuln_repo: MockVulnerabilityRepository,
        ) -> Self {
            Self {
                job_repo,
                asset_repo,
                vuln_repo,
            }
        }
    }

    #[async_trait]
    impl DiscoveryService for TestDiscoveryService {
        async fn discover_assets(
            &self,
            organization_id: ID,
            domain: &str,
            job_types: Vec<JobType>,
        ) -> Result<DiscoveryJob> {
            // Create a new job
            let job = DiscoveryJob::new(
                organization_id,
                job_types.first().copied().unwrap_or(JobType::DnsEnum),
                Some(domain.to_string()),
                None,
            );

            let created_job = self.job_repo.create_job(&job).await?;

            // Create a few assets as if discovered
            let asset_domain =
                Asset::new(organization_id, AssetType::Domain, domain.to_string(), None);
            let asset_domain = self.asset_repo.create_asset(&asset_domain).await?;
            self.job_repo.add_asset(asset_domain.clone());

            // Create a subdomain
            let asset_subdomain = Asset::new(
                organization_id,
                AssetType::Domain,
                format!("www.{}", domain),
                None,
            );
            let asset_subdomain = self.asset_repo.create_asset(&asset_subdomain).await?;
            self.job_repo.add_asset(asset_subdomain.clone());

            // Create an IP address
            let asset_ip = Asset::new(
                organization_id,
                AssetType::IPAddress,
                "192.0.2.1".to_string(),
                None,
            );
            let asset_ip = self.asset_repo.create_asset(&asset_ip).await?;
            self.job_repo.add_asset(asset_ip.clone());

            // Link assets to job
            self.job_repo
                .create_job_asset_link(&JobAssetLink::new(created_job.id, asset_domain.id))
                .await?;
            self.job_repo
                .create_job_asset_link(&JobAssetLink::new(created_job.id, asset_subdomain.id))
                .await?;
            self.job_repo
                .create_job_asset_link(&JobAssetLink::new(created_job.id, asset_ip.id))
                .await?;

            // Update job status
            let mut updated_job = created_job;
            updated_job.status = JobStatus::Completed;
            updated_job.completed_at = Some(chrono::Utc::now());

            let updated_job = self.job_repo.update_job(&updated_job).await?;

            Ok(updated_job)
        }

        async fn scan_asset(&self, asset_id: ID) -> Result<Vec<Vulnerability>> {
            // Get the asset
            let asset = self.asset_repo.get_asset(asset_id).await?;

            // Create some mock vulnerabilities
            let vuln1 = Vulnerability::new(
                asset.id,
                None,
                "Test Vulnerability 1".to_string(),
                Some("This is a test vulnerability for unit tests".to_string()),
                Severity::Medium,
                None,
                None,
                None,
            );

            let vuln2 = Vulnerability::new(
                asset.id,
                None,
                "Test Vulnerability 2".to_string(),
                Some("This is another test vulnerability".to_string()),
                Severity::High,
                None,
                None,
                None,
            );

            let created_vuln1 = self.vuln_repo.create_vulnerability(&vuln1).await?;
            let created_vuln2 = self.vuln_repo.create_vulnerability(&vuln2).await?;

            Ok(vec![created_vuln1, created_vuln2])
        }
    }

    #[test]
    async fn test_discover_assets() {
        let job_repo = MockDiscoveryJobRepository::new();
        let asset_repo = MockAssetRepository::new();
        let vuln_repo = MockVulnerabilityRepository::new();

        let discovery_service =
            TestDiscoveryService::new(job_repo.clone(), asset_repo.clone(), vuln_repo.clone());

        let org_id = Uuid::new_v4();
        let domain = "example.com";

        // Test asset discovery
        let job = discovery_service
            .discover_assets(org_id, domain, vec![JobType::DnsEnum])
            .await
            .unwrap();

        // Verify job was created and completed
        assert_eq!(job.organization_id, org_id);
        assert_eq!(job.job_type, JobType::DnsEnum);
        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.completed_at.is_some());

        // Verify assets were created and linked
        let assets = job_repo.get_job_assets(job.id).await.unwrap();
        assert_eq!(assets.len(), 3);

        // Verify the discovered assets
        let domain_assets: Vec<_> = assets
            .iter()
            .filter(|a| a.asset_type == AssetType::Domain)
            .collect();
        assert_eq!(domain_assets.len(), 2);

        let ip_assets: Vec<_> = assets
            .iter()
            .filter(|a| a.asset_type == AssetType::IPAddress)
            .collect();
        assert_eq!(ip_assets.len(), 1);
    }

    #[test]
    async fn test_scan_asset() {
        let job_repo = MockDiscoveryJobRepository::new();
        let asset_repo = MockAssetRepository::new();
        let vuln_repo = MockVulnerabilityRepository::new();

        let discovery_service =
            TestDiscoveryService::new(job_repo.clone(), asset_repo.clone(), vuln_repo.clone());

        // Create an asset first
        let org_id = Uuid::new_v4();
        let asset = Asset::new(org_id, AssetType::Domain, "example.com".to_string(), None);
        let created_asset = asset_repo.create_asset(&asset).await.unwrap();

        // Test vulnerability scanning
        let vulnerabilities = discovery_service
            .scan_asset(created_asset.id)
            .await
            .unwrap();

        // Verify vulnerabilities were created
        assert_eq!(vulnerabilities.len(), 2);

        // Verify severities
        let high_severity = vulnerabilities
            .iter()
            .find(|v| v.severity == Severity::High)
            .expect("Expected a High severity vulnerability");

        let medium_severity = vulnerabilities
            .iter()
            .find(|v| v.severity == Severity::Medium)
            .expect("Expected a Medium severity vulnerability");

        assert_eq!(high_severity.title, "Test Vulnerability 2");
        assert_eq!(medium_severity.title, "Test Vulnerability 1");
    }
}
