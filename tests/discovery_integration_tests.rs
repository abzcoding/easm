#[cfg(test)]
mod discovery_integration_tests {
    use backend::models::{Asset, DiscoveryJob, Organization};
    use discovery::results::DiscoveryResult;
    use infrastructure::repositories::RepositoryFactory;
    use shared::{
        config::Config,
        types::{AssetType, JobStatus as DiscoveryJobStatus, JobType as DiscoveryType},
    };
    use uuid::Uuid;

    async fn setup_test_db() -> RepositoryFactory {
        // Use environment variables for database connection
        let _ = dotenvy::dotenv();
        let config = Config::from_env().expect("Failed to load config");
        let database_url = config.database_url.clone();

        // Create database connection pool
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        // Create repository factory
        RepositoryFactory::new(pool)
    }

    // Define a trait for task execution
    #[async_trait::async_trait]
    trait TaskExecutor {
        async fn execute(&self, target: &str) -> anyhow::Result<DiscoveryResult>;
    }

    // Mock DNS discovery implementation
    struct MockDnsDiscoveryTask;

    #[async_trait::async_trait]
    impl TaskExecutor for MockDnsDiscoveryTask {
        async fn execute(&self, target: &str) -> anyhow::Result<DiscoveryResult> {
            // Mock implementation that returns synthetic subdomains
            let mut result = DiscoveryResult::new();

            // Add a few synthetic subdomains as discovery results
            for i in 1..=3 {
                result.domains.push(discovery::results::DiscoveredDomain {
                    domain_name: format!("sub{}.{}", i, target),
                    source: "mock_dns_discovery".to_string(),
                });
            }

            Ok(result)
        }
    }

    #[tokio::test]
    async fn test_discovery_task_integration() {
        // Set up the test database and repositories
        let factory = setup_test_db().await;
        let org_repo = factory.organization_repository();

        // Create a test organization
        let org_name = format!("Test Discovery Integration {}", Uuid::new_v4());
        let org = Organization::new(org_name.clone());
        let created_org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Initialize repositories
        let asset_repo = factory.asset_repository();
        let job_repo = factory.discovery_job_repository();

        // Create initial domain asset
        let domain_name = format!(
            "discovery-{}.example.com",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        );
        let domain_asset = Asset::new(created_org.id, AssetType::Domain, domain_name.clone(), None);

        let created_domain = asset_repo
            .create_asset(&domain_asset)
            .await
            .expect("Failed to create domain asset");

        // Create a discovery job
        let job = DiscoveryJob::new(
            created_org.id,
            DiscoveryType::DnsEnum,
            Some(domain_name.clone()),
            None,
        );

        let created_job = job_repo
            .create_job(&job)
            .await
            .expect("Failed to create discovery job");

        // Update job to in-progress
        let mut updated_job = created_job.clone();
        updated_job.status = DiscoveryJobStatus::Running;
        let _ = job_repo
            .update_job(&updated_job)
            .await
            .expect("Failed to update job status");

        // Execute mock discovery task
        let discovery_task = MockDnsDiscoveryTask;
        let discovery_results = discovery_task
            .execute(&domain_name)
            .await
            .expect("Task execution failed");

        assert_eq!(discovery_results.domains.len(), 3);

        // Create a vector to store created assets for cleanup
        let mut created_assets = Vec::new();

        // Process discovery results and create assets
        for domain in discovery_results.domains {
            // Create asset for subdomain
            let asset = Asset::new(
                created_org.id,
                AssetType::Domain,
                domain.domain_name.clone(),
                None,
            );

            let created_asset = asset_repo
                .create_asset(&asset)
                .await
                .expect("Failed to create subdomain asset");

            created_assets.push(created_asset);
        }

        // Complete the job
        let mut completed_job = updated_job.clone();
        completed_job.status = DiscoveryJobStatus::Completed;
        let completed_job = job_repo
            .update_job(&completed_job)
            .await
            .expect("Failed to complete job");

        assert_eq!(completed_job.status, DiscoveryJobStatus::Completed);

        // Clean up - delete assets and organization
        for asset in created_assets {
            let _ = asset_repo
                .delete_asset(asset.id)
                .await
                .expect("Failed to delete asset");
        }

        let _ = asset_repo
            .delete_asset(created_domain.id)
            .await
            .expect("Failed to delete domain asset");

        let _ = job_repo
            .delete_job(created_job.id)
            .await
            .expect("Failed to delete job");

        let _ = org_repo
            .delete_organization(created_org.id)
            .await
            .expect("Failed to delete organization");
    }
}
