use anyhow::Result;
use backend::models::{Asset, DiscoveryJob};
use backend::services::{AssetServiceImpl, DiscoveryServiceImpl};
use backend::traits::AssetService;
use chrono::Utc;
use discovery::dns;
use discovery::port_scan;
use discovery::results::DiscoveryResult;
use infrastructure::repositories::factory::RepositoryFactory;
use shared::types::{AssetStatus, AssetType, JobStatus, JobType};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Process pending discovery jobs
/// Returns the number of jobs processed
pub async fn process_pending_jobs(pool: &PgPool) -> Result<usize> {
    let repo_factory = Arc::new(RepositoryFactory::new(pool.clone()));

    // Create services with the appropriate repositories
    let asset_repository = repo_factory.asset_repository();
    let discovery_job_repository = repo_factory.discovery_job_repository();

    let asset_service = AssetServiceImpl::new(asset_repository.clone());
    let discovery_service =
        DiscoveryServiceImpl::new(asset_repository.clone(), discovery_job_repository);

    // Log that we're checking for jobs
    tracing::info!("Checking for pending jobs...");

    // Fetch pending jobs
    let pending_jobs = discovery_service
        .get_jobs_by_status(JobStatus::Pending, 10)
        .await?;

    if pending_jobs.is_empty() {
        tracing::info!("No pending jobs found");
        return Ok(0);
    }

    tracing::info!("Found {} pending jobs", pending_jobs.len());

    let mut processed = 0;

    // Process each job
    for mut job in pending_jobs {
        tracing::info!("Processing job: {} ({:?})", job.id, job.job_type);

        // Update job status to running
        job.status = JobStatus::Running;
        job.started_at = Some(Utc::now());
        job = discovery_service.update_job(&job).await?;

        // Process the job based on type
        let result = match job.job_type {
            JobType::DnsEnum => {
                if let Some(target) = &job.target {
                    tracing::info!("Running DNS enumeration for {}", target);
                    process_dns_enumeration(&asset_service, &job, target).await
                } else {
                    Err(anyhow::anyhow!(
                        "No target specified for DNS enumeration job"
                    ))
                }
            }
            JobType::PortScan => {
                if let Some(target) = &job.target {
                    tracing::info!("Running port scan for {}", target);
                    process_port_scan(&asset_service, &job, target).await
                } else {
                    Err(anyhow::anyhow!("No target specified for port scan job"))
                }
            }
            JobType::WebCrawl => {
                tracing::warn!("Web crawl jobs not implemented yet");
                Err(anyhow::anyhow!("Web crawl jobs not implemented yet"))
            }
            JobType::CertScan => {
                tracing::warn!("Certificate transparency jobs not implemented yet");
                Err(anyhow::anyhow!(
                    "Certificate transparency jobs not implemented yet"
                ))
            }
            JobType::VulnScan => {
                tracing::warn!("Vulnerability scan jobs not implemented yet");
                Err(anyhow::anyhow!(
                    "Vulnerability scan jobs not implemented yet"
                ))
            }
        };

        // Update job status based on result
        job.completed_at = Some(Utc::now());
        job.status = match &result {
            Ok(_) => {
                tracing::info!("Job {} completed successfully", job.id);
                JobStatus::Completed
            }
            Err(e) => {
                tracing::error!("Job {} failed: {}", job.id, e);
                // Add error to logs
                job.logs = Some(format!("Error: {}", e));
                JobStatus::Failed
            }
        };

        // Update the job
        discovery_service.update_job(&job).await?;

        if result.is_ok() {
            processed += 1;
        }
    }

    tracing::info!("Successfully processed {} jobs", processed);
    Ok(processed)
}

/// Process DNS enumeration discovery
async fn process_dns_enumeration(
    asset_service: &impl AssetService,
    job: &DiscoveryJob,
    target: &str,
) -> Result<()> {
    // Use the DNS enumerator
    let dns_enumerator = dns::DnsEnumerator::new().await?;
    let results = dns_enumerator.enumerate(target).await?;

    // Process the results
    process_discovery_results(asset_service, job.organization_id, results).await
}

/// Process port scan discovery
async fn process_port_scan(
    asset_service: &impl AssetService,
    job: &DiscoveryJob,
    target: &str,
) -> Result<()> {
    // Resolve the target to IP addresses if it's a domain
    let dns_enumerator = dns::DnsEnumerator::new().await?;
    let ips = dns_enumerator.resolve(target).await?;

    if ips.is_empty() {
        return Err(anyhow::anyhow!(
            "No IP addresses found for target: {}",
            target
        ));
    }

    // Run port scan on each IP
    let mut all_results = DiscoveryResult::new();

    for ip in ips {
        let scanner = port_scan::PortScanner::new();
        let results = scanner.scan_ip(&ip.to_string(), None).await?;
        all_results.merge(results);
    }

    // Process the results
    process_discovery_results(asset_service, job.organization_id, all_results).await
}

/// Process discovery results and create assets
async fn process_discovery_results(
    asset_service: &impl AssetService,
    org_id: Uuid,
    results: DiscoveryResult,
) -> Result<()> {
    // Process domains
    for domain in results.domains {
        let asset = Asset {
            id: Uuid::new_v4(),
            organization_id: org_id,
            asset_type: AssetType::Domain,
            value: domain.domain_name,
            status: AssetStatus::Active,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attributes: serde_json::json!({ "source": domain.source }),
        };

        // Create or update the asset
        match asset_service.create_asset(&asset).await {
            Ok(_) => tracing::debug!("Created domain asset: {}", asset.value),
            Err(e) => {
                tracing::warn!("Failed to create domain asset: {}: {}", asset.value, e);
                return Err(anyhow::anyhow!(
                    "Failed to create domain asset: {}: {}",
                    asset.value,
                    e
                ));
            }
        }
    }

    // Process IP addresses
    for ip in results.ip_addresses {
        let asset = Asset {
            id: Uuid::new_v4(),
            organization_id: org_id,
            asset_type: AssetType::IPAddress,
            value: ip.ip_address.to_string(),
            status: AssetStatus::Active,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attributes: serde_json::json!({ "source": ip.source }),
        };

        // Create or update the asset
        match asset_service.create_asset(&asset).await {
            Ok(_) => tracing::debug!("Created IP asset: {}", asset.value),
            Err(e) => {
                tracing::warn!("Failed to create IP asset: {}: {}", asset.value, e);
                return Err(anyhow::anyhow!(
                    "Failed to create IP asset: {}: {}",
                    asset.value,
                    e
                ));
            }
        }
    }

    // Process ports
    for port in results.ports {
        // First, ensure the IP asset exists
        let ip_asset = Asset {
            id: Uuid::new_v4(),
            organization_id: org_id,
            asset_type: AssetType::IPAddress,
            value: port.ip_address.to_string(),
            status: AssetStatus::Active,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attributes: serde_json::json!({ "source": port.source.clone() }),
        };

        let ip_asset = match asset_service.create_asset(&ip_asset).await {
            Ok(asset) => asset,
            Err(e) => {
                tracing::warn!(
                    "Failed to create IP asset for port: {}: {}",
                    ip_asset.value,
                    e
                );
                return Err(anyhow::anyhow!(
                    "Failed to create IP asset for port: {}: {}",
                    ip_asset.value,
                    e
                ));
            }
        };

        // Now add port information to the asset attributes
        // Since there's no direct add_port_to_asset method, we'll update the asset's attributes
        let mut updated_asset = ip_asset.clone();
        let mut attributes = match updated_asset.attributes.as_object() {
            Some(obj) => obj.clone(),
            None => serde_json::Map::new(),
        };

        // Add or update ports array in attributes
        let mut ports = match attributes.get("ports").and_then(|p| p.as_array()) {
            Some(existing_ports) => existing_ports.clone(),
            None => Vec::new(),
        };

        // Add this port to the ports list
        ports.push(serde_json::json!({
            "port": port.port,
            "protocol": port.protocol,
            "service": port.service_name,
            "banner": port.banner,
            "status": port.status
        }));

        attributes.insert("ports".to_string(), serde_json::Value::Array(ports));
        updated_asset.attributes = serde_json::Value::Object(attributes);

        // Update the asset with new attributes
        match asset_service.update_asset(&updated_asset).await {
            Ok(_) => tracing::debug!(
                "Added port {} ({}) to IP {}",
                port.port,
                port.protocol,
                ip_asset.value
            ),
            Err(e) => {
                tracing::warn!(
                    "Failed to add port {} to IP {}: {}",
                    port.port,
                    ip_asset.value,
                    e
                );
                return Err(anyhow::anyhow!(
                    "Failed to add port {} to IP {}: {}",
                    port.port,
                    ip_asset.value,
                    e
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*; // Import items from parent module (job_processor)
    use backend::{
        errors as backend_error, // Alias to avoid conflict with anyhow::Error
        traits::AssetRepository,
        Result as BackendResult, // Use the Result alias from backend
    };
    use discovery::results::{DiscoveredDomain, DiscoveryResult};
    use mockall::{mock, predicate::*};
    use std::sync::Arc;

    mock! {
        pub AssetRepository {}

        #[async_trait::async_trait]
        impl AssetRepository for AssetRepository {
            // Use BackendResult and backend::Error
            async fn create_asset(&self, asset: &Asset) -> BackendResult<Asset>;
            async fn get_asset(&self, id: Uuid) -> BackendResult<Asset>;
            async fn update_asset(&self, asset: &Asset) -> BackendResult<Asset>;
            async fn delete_asset(&self, id: Uuid) -> BackendResult<bool>;
            async fn list_assets(
                &self,
                organization_id: Option<Uuid>,
                asset_type: Option<AssetType>,
                status: Option<AssetStatus>,
                limit: usize,
                offset: usize,
            ) -> BackendResult<Vec<Asset>>;
            async fn count_assets(
                &self,
                organization_id: Option<Uuid>,
                asset_type: Option<AssetType>,
                status: Option<AssetStatus>,
            ) -> BackendResult<usize>;
        }
    }

    // Helper to create a simple discovery result for testing
    async fn create_test_discovery_result(org_id: Uuid) -> Result<()> {
        // Create a mock discovery result with a single domain
        let mut results = DiscoveryResult::new();
        results.domains.push(DiscoveredDomain {
            domain_name: "example.com".to_string(),
            source: "test".to_string(),
        });

        // Set up the mock repository
        let mut mock_repo = MockAssetRepository::new();
        mock_repo
            .expect_create_asset()
            .withf(move |asset: &Asset| {
                asset.organization_id == org_id && asset.value == "example.com"
            })
            .times(1)
            .returning(|asset| Ok(asset.clone()));

        let asset_service = AssetServiceImpl::new(Arc::new(mock_repo));

        // Process the results
        process_discovery_results(&asset_service, org_id, results).await
    }

    #[tokio::test]
    async fn test_process_discovery_result_success() {
        let org_id = Uuid::new_v4();
        let result = create_test_discovery_result(org_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_discovery_result_failure() {
        let org_id = Uuid::new_v4();

        // Create a simple discovery result
        let mut results = DiscoveryResult::new();
        results.domains.push(DiscoveredDomain {
            domain_name: "example.com".to_string(),
            source: "test".to_string(),
        });

        // Set up the mock repository to return an error
        let mut mock_repo = MockAssetRepository::new();
        mock_repo
            .expect_create_asset()
            .withf(move |asset: &Asset| {
                asset.organization_id == org_id && asset.value == "example.com"
            })
            .times(1)
            .returning(|_| Err(backend_error::Error::Database("Mock DB error".to_string())));

        let asset_service = AssetServiceImpl::new(Arc::new(mock_repo));
        let result = process_discovery_results(&asset_service, org_id, results).await;

        assert!(result.is_err());
        let err_string = result.unwrap_err().to_string();
        assert!(err_string.contains("Failed to create domain asset"));
        // Check that the underlying Database error message is included
        assert!(err_string.contains("Mock DB error"));
    }
}
