#[cfg(test)]
mod task_integration_tests {
    use anyhow::Result;
    use backend::traits::DiscoveryJobRepository;
    use backend::{
        models::{Asset, DiscoveryJob, Organization},
        traits::AssetService,
    };
    use discovery::results::DiscoveryResult;
    use infrastructure::repositories::{PgDiscoveryJobRepository, RepositoryFactory};
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
        async fn execute(&self, target: &str) -> Result<DiscoveryResult>;
    }

    // Mock discovery task implementation
    struct MockTaskDiscoveryTask;

    #[async_trait::async_trait]
    impl TaskExecutor for MockTaskDiscoveryTask {
        async fn execute(&self, target: &str) -> Result<DiscoveryResult> {
            // Mock implementation that returns synthetic results
            let mut result = DiscoveryResult::new();

            // Add domain results
            result.domains.push(discovery::results::DiscoveredDomain {
                domain_name: format!("task-sub1.{}", target),
                source: "mock_task_discovery".to_string(),
            });

            result.domains.push(discovery::results::DiscoveredDomain {
                domain_name: format!("task-sub2.{}", target),
                source: "mock_task_discovery".to_string(),
            });

            Ok(result)
        }
    }

    // Create a mock job processor for testing
    struct MockJobProcessor {
        job_repo: PgDiscoveryJobRepository,
        asset_service: backend::services::AssetServiceImpl,
    }

    impl MockJobProcessor {
        fn new(
            job_repo: PgDiscoveryJobRepository,
            asset_service: backend::services::AssetServiceImpl,
        ) -> Self {
            Self {
                job_repo,
                asset_service,
            }
        }

        // Simplified version of process_job from JobProcessor
        async fn process_job(&self, job_id: uuid::Uuid) -> Result<(), Box<dyn std::error::Error>> {
            // Get the job
            let mut job = self.job_repo.get_job(job_id).await?;

            // Update job status to in progress
            job.status = DiscoveryJobStatus::Running;
            job = self.job_repo.update_job(&job).await?;

            // Run the discovery task
            let discovery_task = MockTaskDiscoveryTask;
            let results = discovery_task.execute(job.target.as_ref().unwrap()).await?;

            // Create a vector to store the created assets
            let mut created_assets = Vec::new();

            // Process results and create assets
            for domain in results.domains {
                // Create asset for the subdomain
                let asset = Asset::new(
                    job.organization_id,
                    AssetType::Domain,
                    domain.domain_name.clone(),
                    None,
                );

                let created_asset = self.asset_service.create_asset(&asset).await?;
                created_assets.push(created_asset);
            }

            // Complete the job
            job.status = DiscoveryJobStatus::Completed;
            self.job_repo.update_job(&job).await?;

            Ok(())
        }
    }

    #[tokio::test]
    async fn test_job_processor_integration() {
        // Set up the test database and repositories
        let factory = setup_test_db().await;
        let org_repo = factory.organization_repository();

        // Create a test organization
        let org_name = format!("Test Job Processor {}", Uuid::new_v4());
        let org = Organization::new(org_name.clone());
        let created_org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Initialize repositories
        let asset_repo = factory.asset_repository();
        let asset_service = backend::services::AssetServiceImpl::new(asset_repo.clone());
        let job_repo = factory.discovery_job_repository();

        // Create a discovery job
        let domain_name = format!(
            "task-{}.example.com",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        );
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

        // Get concrete repository implementations for direct use
        let direct_job_repo = PgDiscoveryJobRepository::new(factory.pool().clone());

        // Create and run the job processor
        let job_processor = MockJobProcessor::new(direct_job_repo, asset_service);

        // Process the job
        job_processor
            .process_job(created_job.id)
            .await
            .expect("Failed to process job");

        // Verify job status
        let updated_job = job_repo
            .get_job(created_job.id)
            .await
            .expect("Failed to get job");

        assert_eq!(updated_job.status, DiscoveryJobStatus::Completed);

        // Find assets by domain pattern
        let all_assets = asset_repo
            .list_assets(Some(created_org.id), Some(AssetType::Domain), None, 20, 0)
            .await
            .expect("Failed to list assets");

        // Filter assets that match our pattern
        let job_assets: Vec<Asset> = all_assets
            .into_iter()
            .filter(|a| a.value.contains("task-sub") && a.value.contains(&domain_name))
            .collect();

        assert_eq!(job_assets.len(), 2);

        // Verify asset values
        let asset_values: Vec<String> = job_assets.iter().map(|a| a.value.clone()).collect();
        assert!(asset_values.contains(&format!("task-sub1.{}", domain_name)));
        assert!(asset_values.contains(&format!("task-sub2.{}", domain_name)));

        // Clean up - delete assets and organization
        for asset in job_assets {
            let _ = asset_repo
                .delete_asset(asset.id)
                .await
                .expect("Failed to delete asset");
        }

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
