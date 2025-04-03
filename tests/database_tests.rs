#[cfg(test)]
mod integration_tests {
    use backend::models::{Asset, Organization, Vulnerability};
    use infrastructure::repositories::RepositoryFactory;
    use shared::{
        config::Config,
        types::{AssetStatus, AssetType, Severity, VulnerabilityStatus},
    };

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

    #[tokio::test]
    async fn test_organization_crud_operations() {
        let factory = setup_test_db().await;

        // Create repository
        let repo = factory.organization_repository();

        // Create a test organization
        let org_name = format!("Test Org {}", uuid::Uuid::new_v4());
        let org = Organization::new(org_name.clone());

        // Create
        let created_org = repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");
        assert_eq!(created_org.name, org_name);

        // Get
        let retrieved_org = repo
            .get_organization(created_org.id)
            .await
            .expect("Failed to get organization");
        assert_eq!(retrieved_org.id, created_org.id);
        assert_eq!(retrieved_org.name, org_name);

        // Update
        let mut updated_org = retrieved_org.clone();
        updated_org.name = format!("Updated {}", org_name);
        let updated_result = repo
            .update_organization(&updated_org)
            .await
            .expect("Failed to update organization");
        assert_eq!(updated_result.name, updated_org.name);

        // List
        let orgs = repo
            .list_organizations(10, 0)
            .await
            .expect("Failed to list organizations");
        assert!(!orgs.is_empty());

        // Count
        let count = repo
            .count_organizations()
            .await
            .expect("Failed to count organizations");
        assert!(count > 0);

        // Delete
        let deleted = repo
            .delete_organization(created_org.id)
            .await
            .expect("Failed to delete organization");
        assert!(deleted);
    }

    #[tokio::test]
    async fn test_asset_repository() {
        let factory = setup_test_db().await;

        // Create repositories
        let org_repo = factory.organization_repository();
        let asset_repo = factory.asset_repository();

        // First create an organization
        let org = Organization::new("Test Org for Assets".to_string());
        let org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Create an asset
        let asset = Asset::new(
            org.id,
            AssetType::Domain,
            "test.example.com".to_string(),
            None,
        );

        // Create
        let created_asset = asset_repo
            .create_asset(&asset)
            .await
            .expect("Failed to create asset");
        assert_eq!(created_asset.value, "test.example.com");
        assert_eq!(created_asset.asset_type, AssetType::Domain);

        // Get
        let retrieved_asset = asset_repo
            .get_asset(created_asset.id)
            .await
            .expect("Failed to get asset");
        assert_eq!(retrieved_asset.id, created_asset.id);

        // List
        let assets = asset_repo
            .list_assets(
                Some(org.id),
                Some(AssetType::Domain),
                Some(AssetStatus::Active),
                10,
                0,
            )
            .await
            .expect("Failed to list assets");
        assert!(!assets.is_empty());
        assert!(assets.iter().any(|a| a.id == created_asset.id));

        // Delete the asset and organization to clean up
        let _ = asset_repo
            .delete_asset(created_asset.id)
            .await
            .expect("Failed to delete asset");
        let _ = org_repo
            .delete_organization(org.id)
            .await
            .expect("Failed to delete organization");
    }

    #[tokio::test]
    async fn test_vulnerability_repository() {
        let factory = setup_test_db().await;

        // Create repositories
        let org_repo = factory.organization_repository();
        let asset_repo = factory.asset_repository();
        let vuln_repo = factory.vulnerability_repository();

        // First create an organization
        let org = Organization::new("Test Org for Vulns".to_string());
        let org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Create an asset
        let asset = Asset::new(
            org.id,
            AssetType::Domain,
            "vuln.example.com".to_string(),
            None,
        );
        let asset = asset_repo
            .create_asset(&asset)
            .await
            .expect("Failed to create asset");

        // Create a vulnerability
        let vuln = Vulnerability::new(
            asset.id,
            None, // No specific port
            "Test Vulnerability".to_string(),
            Some("This is a test vulnerability".to_string()),
            Severity::Medium,
            None, // No CVE ID
            None, // No evidence
            None, // No remediation
        );

        // Create
        let created_vuln = vuln_repo
            .create_vulnerability(&vuln)
            .await
            .expect("Failed to create vulnerability");
        assert_eq!(created_vuln.title, "Test Vulnerability");
        assert_eq!(created_vuln.severity, Severity::Medium);

        // Get
        let retrieved_vuln = vuln_repo
            .get_vulnerability(created_vuln.id)
            .await
            .expect("Failed to get vulnerability");
        assert_eq!(retrieved_vuln.id, created_vuln.id);

        // List
        let vulns = vuln_repo
            .list_vulnerabilities(
                Some(asset.id),
                None,
                Some(Severity::Medium),
                Some(VulnerabilityStatus::Open),
                10,
                0,
            )
            .await
            .expect("Failed to list vulnerabilities");
        assert!(!vulns.is_empty());

        // Clean up
        let _ = vuln_repo
            .delete_vulnerability(created_vuln.id)
            .await
            .expect("Failed to delete vulnerability");
        let _ = asset_repo
            .delete_asset(asset.id)
            .await
            .expect("Failed to delete asset");
        let _ = org_repo
            .delete_organization(org.id)
            .await
            .expect("Failed to delete organization");
    }

    #[tokio::test]
    async fn test_end_to_end_flow() {
        let factory = setup_test_db().await;

        // Create repositories
        let org_repo = factory.organization_repository();
        let asset_repo = factory.asset_repository();
        let vuln_repo = factory.vulnerability_repository();

        // 1. Create an organization
        let org_name = format!("E2E Test Org {}", uuid::Uuid::new_v4());
        let org = Organization::new(org_name);
        let org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // 2. Create multiple assets
        let domain_asset = Asset::new(
            org.id,
            AssetType::Domain,
            "e2e.example.com".to_string(),
            None,
        );
        let domain_asset = asset_repo
            .create_asset(&domain_asset)
            .await
            .expect("Failed to create domain asset");

        let ip_asset = Asset::new(
            org.id,
            AssetType::IPAddress,
            "192.168.1.1".to_string(),
            None,
        );
        let ip_asset = asset_repo
            .create_asset(&ip_asset)
            .await
            .expect("Failed to create IP asset");

        // 3. Add vulnerabilities to assets
        let domain_vuln = Vulnerability::new(
            domain_asset.id,
            None,
            "Domain Vulnerability".to_string(),
            Some("This is a domain vulnerability".to_string()),
            Severity::High,
            None,                                            // No CVE ID
            None,                                            // No evidence
            Some("Update domain configuration".to_string()), // Remediation
        );
        let domain_vuln = vuln_repo
            .create_vulnerability(&domain_vuln)
            .await
            .expect("Failed to create domain vulnerability");

        let ip_vuln = Vulnerability::new(
            ip_asset.id,
            None,
            "IP Vulnerability".to_string(),
            Some("This is an IP vulnerability".to_string()),
            Severity::Low,
            None,                                      // No CVE ID
            None,                                      // No evidence
            Some("Update firewall rules".to_string()), // Remediation
        );
        let ip_vuln = vuln_repo
            .create_vulnerability(&ip_vuln)
            .await
            .expect("Failed to create IP vulnerability");

        // 4. Verify organization assets can be listed
        let assets = asset_repo
            .list_assets(Some(org.id), None, None, 10, 0)
            .await
            .expect("Failed to list organization assets");
        assert_eq!(assets.len(), 2);

        // 5. Verify asset vulnerabilities can be queried
        let domain_vulns = vuln_repo
            .list_vulnerabilities(Some(domain_asset.id), None, None, None, 10, 0)
            .await
            .expect("Failed to list domain vulnerabilities");
        assert_eq!(domain_vulns.len(), 1);
        assert_eq!(domain_vulns[0].id, domain_vuln.id);

        // 6. Verify filtering works correctly
        let high_vulns = vuln_repo
            .list_vulnerabilities(None, None, Some(Severity::High), None, 10, 0)
            .await
            .expect("Failed to list high vulnerabilities");
        assert!(!high_vulns.is_empty());
        assert!(high_vulns.iter().any(|v| v.id == domain_vuln.id));

        // Clean up
        let _ = vuln_repo
            .delete_vulnerability(domain_vuln.id)
            .await
            .expect("Failed to delete domain vulnerability");
        let _ = vuln_repo
            .delete_vulnerability(ip_vuln.id)
            .await
            .expect("Failed to delete IP vulnerability");
        let _ = asset_repo
            .delete_asset(domain_asset.id)
            .await
            .expect("Failed to delete domain asset");
        let _ = asset_repo
            .delete_asset(ip_asset.id)
            .await
            .expect("Failed to delete IP asset");
        let _ = org_repo
            .delete_organization(org.id)
            .await
            .expect("Failed to delete organization");
    }
}
