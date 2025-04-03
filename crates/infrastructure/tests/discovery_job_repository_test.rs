#[cfg(test)]
mod tests {
    use backend::models::{DiscoveryJob, JobAssetLink};
    use infrastructure::{
        repositories::factory::RepositoryFactory,
        utils::testing::{create_test_asset, create_test_organization, setup_test_db},
    };
    use shared::types::{AssetType, JobStatus, JobType, ID};

    use serde_json::json;

    // Helper to create a test job
    async fn create_test_job(
        factory: &RepositoryFactory,
        org_id: ID,
        job_type: JobType,
        status: JobStatus,
    ) -> DiscoveryJob {
        let job = DiscoveryJob::new(
            org_id,
            job_type,
            Some("example.com".to_string()), // target
            Some(json!({ "depth": 1 })),     // configuration
        );
        let mut created_job = factory
            .discovery_job_repository()
            .create_job(&job)
            .await
            .expect("Failed to create test job");

        // Update status if needed
        if created_job.status != status {
            created_job.status = status;
            if status == JobStatus::Running {
                created_job.started_at = Some(chrono::Utc::now());
            } else if status == JobStatus::Completed || status == JobStatus::Failed {
                created_job.started_at = Some(chrono::Utc::now() - chrono::Duration::minutes(5));
                created_job.completed_at = Some(chrono::Utc::now());
            }
            created_job = factory
                .discovery_job_repository()
                .update_job(&created_job)
                .await
                .expect("Failed to update job status");
        }
        created_job
    }

    #[tokio::test]
    async fn test_discovery_job_repository_basic_operations() {
        let (db_pool, _container) = setup_test_db().await;
        let factory = RepositoryFactory::new(db_pool);
        let job_repo = factory.discovery_job_repository();

        // Setup: Create org
        let org = create_test_organization(&factory, "Test Org Job Repo")
            .await
            .unwrap();

        // 1. Create Job
        let created_job =
            create_test_job(&factory, org.id, JobType::DnsEnum, JobStatus::Pending).await;
        assert_eq!(created_job.organization_id, org.id);
        assert_eq!(created_job.job_type, JobType::DnsEnum);
        assert_eq!(created_job.status, JobStatus::Pending);

        // 2. Get Job
        let found_job = job_repo
            .get_job(created_job.id)
            .await
            .expect("Failed to get job");
        assert_eq!(found_job.id, created_job.id);

        // 3. Update Job
        let mut job_to_update = found_job.clone();
        job_to_update.status = JobStatus::Completed;
        job_to_update.logs = Some("Completed successfully".to_string());
        job_to_update.completed_at = Some(chrono::Utc::now());
        let updated_job = job_repo
            .update_job(&job_to_update)
            .await
            .expect("Failed to update job");
        assert_eq!(updated_job.id, created_job.id);
        assert_eq!(updated_job.status, JobStatus::Completed);
        assert!(updated_job.completed_at.is_some());

        // Verify update
        let verified_job = job_repo
            .get_job(created_job.id)
            .await
            .expect("Failed to get job after update");
        assert_eq!(verified_job.status, JobStatus::Completed);

        // 4. Delete Job
        let deleted = job_repo
            .delete_job(created_job.id)
            .await
            .expect("Failed to delete job");
        assert!(deleted);

        // Verify deletion
        let not_found_result = job_repo.get_job(created_job.id).await;
        assert!(not_found_result.is_err());
    }

    #[tokio::test]
    async fn test_discovery_job_repository_list_and_filters() {
        let (db_pool, _container) = setup_test_db().await;
        let factory = RepositoryFactory::new(db_pool);
        let job_repo = factory.discovery_job_repository();

        // Setup: Create org and jobs
        let org1 = create_test_organization(&factory, "Test Org Job List 1")
            .await
            .unwrap();
        let org2 = create_test_organization(&factory, "Test Org Job List 2")
            .await
            .unwrap();

        let _job1_o1_dns_pen =
            create_test_job(&factory, org1.id, JobType::DnsEnum, JobStatus::Pending).await;
        let job2_o1_scan_run =
            create_test_job(&factory, org1.id, JobType::PortScan, JobStatus::Running).await;
        let job3_o1_dns_com =
            create_test_job(&factory, org1.id, JobType::DnsEnum, JobStatus::Completed).await;
        let _job4_o2_scan_pen =
            create_test_job(&factory, org2.id, JobType::PortScan, JobStatus::Pending).await;

        let limit = 10;
        let offset = 0;

        // List all for Org 1
        let jobs_org1 = job_repo
            .list_jobs(Some(org1.id), None, None, limit, offset)
            .await
            .expect("Failed to list jobs for org 1");
        assert_eq!(jobs_org1.len(), 3);

        // List DnsEnum type for Org 1
        let dns_jobs_org1 = job_repo
            .list_jobs(Some(org1.id), Some(JobType::DnsEnum), None, limit, offset)
            .await
            .expect("Failed to list DNS jobs for org 1");
        assert_eq!(dns_jobs_org1.len(), 2);

        // List Running status for Org 1
        let running_jobs_org1 = job_repo
            .list_jobs(Some(org1.id), None, Some(JobStatus::Running), limit, offset)
            .await
            .expect("Failed to list running jobs for org 1");
        assert_eq!(running_jobs_org1.len(), 1);
        assert_eq!(running_jobs_org1[0].id, job2_o1_scan_run.id);

        // List Completed DnsEnum for Org 1
        let completed_dns_jobs_org1 = job_repo
            .list_jobs(
                Some(org1.id),
                Some(JobType::DnsEnum),
                Some(JobStatus::Completed),
                limit,
                offset,
            )
            .await
            .expect("Failed to list completed DNS jobs for org 1");
        assert_eq!(completed_dns_jobs_org1.len(), 1);
        assert_eq!(completed_dns_jobs_org1[0].id, job3_o1_dns_com.id);

        // Count all for Org 1
        let count_org1 = job_repo
            .count_jobs(Some(org1.id), None, None)
            .await
            .expect("Failed to count jobs for org 1");
        assert_eq!(count_org1, 3);

        // Count Pending for Org 1
        let count_pending_org1 = job_repo
            .count_jobs(Some(org1.id), None, Some(JobStatus::Pending))
            .await
            .expect("Failed to count pending jobs for org 1");
        assert_eq!(count_pending_org1, 1);

        // Count PortScan for Org 1
        let count_scan_org1 = job_repo
            .count_jobs(Some(org1.id), Some(JobType::PortScan), None)
            .await
            .expect("Failed to count scan jobs for org 1");
        assert_eq!(count_scan_org1, 1);

        // Count all jobs
        let count_all = job_repo
            .count_jobs(None, None, None)
            .await
            .expect("Failed to count all jobs");
        assert_eq!(count_all, 4);
    }

    #[tokio::test]
    async fn test_discovery_job_asset_links() {
        let (db_pool, _container) = setup_test_db().await;
        let factory = RepositoryFactory::new(db_pool);
        let job_repo = factory.discovery_job_repository();
        let _asset_repo = factory.asset_repository();

        // Setup
        let org = create_test_organization(&factory, "Test Org Job Asset Link")
            .await
            .unwrap();
        let asset1 = create_test_asset(&factory, org.id, AssetType::Domain, "link1.com")
            .await
            .unwrap();
        let asset2 = create_test_asset(&factory, org.id, AssetType::IPAddress, "2.2.2.2")
            .await
            .unwrap();
        let job = create_test_job(&factory, org.id, JobType::WebCrawl, JobStatus::Completed).await;

        // Create links
        let link1 = JobAssetLink {
            job_id: job.id,
            asset_id: asset1.id,
        };
        let link2 = JobAssetLink {
            job_id: job.id,
            asset_id: asset2.id,
        };

        let created_link1 = job_repo
            .create_job_asset_link(&link1)
            .await
            .expect("Failed to create link 1");
        assert_eq!(created_link1.job_id, job.id);
        assert_eq!(created_link1.asset_id, asset1.id);

        let created_link2 = job_repo
            .create_job_asset_link(&link2)
            .await
            .expect("Failed to create link 2");
        assert_eq!(created_link2.job_id, job.id);
        assert_eq!(created_link2.asset_id, asset2.id);

        // Get assets for job
        let linked_assets = job_repo
            .get_job_assets(job.id)
            .await
            .expect("Failed to get job assets");
        assert_eq!(linked_assets.len(), 2);
        let linked_asset_ids: Vec<ID> = linked_assets.iter().map(|a| a.id).collect();
        assert!(linked_asset_ids.contains(&asset1.id));
        assert!(linked_asset_ids.contains(&asset2.id));

        // Test duplicate link creation fails (should be handled by DB constraint)
        let duplicate_link_result = job_repo.create_job_asset_link(&link1).await;
        assert!(
            duplicate_link_result.is_err(),
            "Creating duplicate link should fail"
        );

        // Test getting assets for a job with no links
        let job_no_links =
            create_test_job(&factory, org.id, JobType::CertScan, JobStatus::Completed).await;
        let no_linked_assets = job_repo
            .get_job_assets(job_no_links.id)
            .await
            .expect("Failed to get assets for job with no links");
        assert!(no_linked_assets.is_empty());
    }
}
