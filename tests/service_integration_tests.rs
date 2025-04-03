#[cfg(test)]
mod service_integration_tests {
    use backend::{
        models::{Asset, Organization, Vulnerability},
        traits::{AssetService, DiscoveryService, VulnerabilityService},
    };
    use infrastructure::repositories::RepositoryFactory;
    use serde_json::json;
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

    // Helper to create test assets
    async fn create_test_assets(factory: &RepositoryFactory, org_id: uuid::Uuid) -> Vec<Asset> {
        let asset_repo = factory.asset_repository();
        let mut created_assets = Vec::new();

        // Create a domain asset
        let domain = Asset::new(
            org_id,
            AssetType::Domain,
            "test-domain.com".to_string(),
            Some(json!({
                "whois_info": {
                    "registrar": "Test Registrar Inc."
                }
            })),
        );

        let domain_asset = asset_repo
            .create_asset(&domain)
            .await
            .expect("Failed to create domain asset");
        created_assets.push(domain_asset);

        // Create a subdomain asset
        let subdomain = Asset::new(
            org_id,
            AssetType::Domain,
            "sub.test-domain.com".to_string(),
            Some(json!({
                "whois_info": {
                    "registrar": "Test Registrar Inc."
                }
            })),
        );

        let subdomain_asset = asset_repo
            .create_asset(&subdomain)
            .await
            .expect("Failed to create subdomain asset");
        created_assets.push(subdomain_asset);

        // Create an IP address asset
        let ip = Asset::new(
            org_id,
            AssetType::IPAddress,
            "192.168.1.1".to_string(),
            None,
        );

        let ip_asset = asset_repo
            .create_asset(&ip)
            .await
            .expect("Failed to create IP asset");
        created_assets.push(ip_asset);

        // Create a web app asset
        let web_app = Asset::new(
            org_id,
            AssetType::WebApp,
            "https://sub.test-domain.com".to_string(),
            Some(json!({
                "host_info": {
                    "domain": "sub.test-domain.com",
                    "ip_address": "192.168.1.1"
                }
            })),
        );

        let web_app_asset = asset_repo
            .create_asset(&web_app)
            .await
            .expect("Failed to create web app asset");
        created_assets.push(web_app_asset);

        created_assets
    }

    // Helper to create test vulnerabilities
    async fn create_test_vulnerabilities(
        factory: &RepositoryFactory,
        assets: &[Asset],
    ) -> Vec<Vulnerability> {
        let vuln_repo = factory.vulnerability_repository();
        let mut created_vulns = Vec::new();

        // Create a vulnerability for the web app
        let web_app_asset = assets
            .iter()
            .find(|a| a.asset_type == AssetType::WebApp)
            .unwrap();

        let now = chrono::Utc::now();
        let vuln1 = Vulnerability {
            id: uuid::Uuid::new_v4(),
            title: "XSS in web form".to_string(),
            description: Some("Cross-site scripting vulnerability in contact form".to_string()),
            asset_id: web_app_asset.id,
            port_id: None,
            severity: Severity::High,
            cvss_score: Some(7.5),
            cve_id: Some("CVE-2023-12345".to_string()),
            evidence: json!({
                "affected_component": "Contact Form",
                "affected_technology": "JavaScript"
            }),
            status: VulnerabilityStatus::Open,
            remediation: Some("Sanitize user inputs and implement CSP".to_string()),
            first_seen: now,
            last_seen: now,
            resolved_at: None,
            created_at: now,
            updated_at: now,
        };

        let vuln1_created = vuln_repo
            .create_vulnerability(&vuln1)
            .await
            .expect("Failed to create vulnerability 1");
        created_vulns.push(vuln1_created);

        // Create another vulnerability for the domain
        let domain_asset = assets
            .iter()
            .find(|a| a.asset_type == AssetType::Domain && a.value == "test-domain.com")
            .unwrap();

        let vuln2 = Vulnerability {
            id: uuid::Uuid::new_v4(),
            title: "SSL Certificate Expiration".to_string(),
            description: Some("SSL certificate is about to expire".to_string()),
            asset_id: domain_asset.id,
            port_id: None,
            severity: Severity::Medium,
            cvss_score: Some(5.0),
            cve_id: None,
            evidence: json!({
                "affected_component": "SSL Certificate",
                "affected_technology": "TLS"
            }),
            status: VulnerabilityStatus::Open,
            remediation: Some("Renew SSL certificate".to_string()),
            first_seen: now,
            last_seen: now,
            resolved_at: None,
            created_at: now,
            updated_at: now,
        };

        let vuln2_created = vuln_repo
            .create_vulnerability(&vuln2)
            .await
            .expect("Failed to create vulnerability 2");
        created_vulns.push(vuln2_created);

        // Create a similar vulnerability for the subdomain
        let subdomain_asset = assets
            .iter()
            .find(|a| a.asset_type == AssetType::Domain && a.value == "sub.test-domain.com")
            .unwrap();

        let vuln3 = Vulnerability {
            id: uuid::Uuid::new_v4(),
            title: "SSL Certificate Expiration".to_string(),
            description: Some("SSL certificate is about to expire".to_string()),
            asset_id: subdomain_asset.id,
            port_id: None,
            severity: Severity::Medium,
            cvss_score: Some(5.0),
            cve_id: None,
            evidence: json!({
                "affected_component": "SSL Certificate",
                "affected_technology": "TLS"
            }),
            status: VulnerabilityStatus::Open,
            remediation: Some("Renew SSL certificate".to_string()),
            first_seen: now,
            last_seen: now,
            resolved_at: None,
            created_at: now,
            updated_at: now,
        };

        let vuln3_created = vuln_repo
            .create_vulnerability(&vuln3)
            .await
            .expect("Failed to create vulnerability 3");
        created_vulns.push(vuln3_created);

        created_vulns
    }

    #[tokio::test]
    async fn test_asset_service_with_repository() {
        // Set up the test database and repositories
        let factory = setup_test_db().await;
        let org_repo = factory.organization_repository();
        let asset_repo = factory.asset_repository();

        // Create a test organization
        let org_name = format!("Test Org Asset Service {}", uuid::Uuid::new_v4());
        let org = Organization::new(org_name.clone());
        let created_org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Create the asset service
        let asset_service = backend::services::AssetServiceImpl::new(asset_repo.clone());

        // Test the asset service
        let asset = Asset::new(
            created_org.id,
            AssetType::Domain,
            "service-example.com".to_string(),
            None,
        );

        // Create an asset
        let created_asset = asset_service
            .create_asset(&asset)
            .await
            .expect("Failed to create asset");

        // Verify asset creation
        assert_eq!(created_asset.organization_id, created_org.id);
        assert_eq!(created_asset.value, "service-example.com");

        // Update the asset
        let mut updated_asset = created_asset.clone();
        updated_asset.status = AssetStatus::Inactive;

        let result = asset_service
            .update_asset(&updated_asset)
            .await
            .expect("Failed to update asset");

        assert_eq!(result.status, AssetStatus::Inactive);

        // Get the asset
        let fetched_asset = asset_service
            .get_asset(created_asset.id)
            .await
            .expect("Failed to get asset");

        assert_eq!(fetched_asset.id, created_asset.id);
        assert_eq!(fetched_asset.status, AssetStatus::Inactive);

        // List assets
        let assets = asset_service
            .list_assets(Some(created_org.id), None, None, 10, 0)
            .await
            .expect("Failed to list assets");

        assert!(!assets.is_empty());

        // Delete the asset
        let deleted = asset_service
            .delete_asset(created_asset.id)
            .await
            .expect("Failed to delete asset");

        assert!(deleted);

        // Clean up
        let _ = org_repo
            .delete_organization(created_org.id)
            .await
            .expect("Failed to delete organization");
    }

    #[tokio::test]
    async fn test_asset_relationship_discovery() {
        // Set up the test database and repositories
        let factory = setup_test_db().await;
        let org_repo = factory.organization_repository();

        // Create a test organization
        let org_name = format!("Test Org Relationships {}", uuid::Uuid::new_v4());
        let org = Organization::new(org_name.clone());
        let created_org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Create test assets
        let assets = create_test_assets(&factory, created_org.id).await;

        // Create the asset service
        let asset_service = backend::services::AssetServiceImpl::new(factory.asset_repository());

        // Test relationship discovery
        let relationships = asset_service
            .discover_asset_relationships(created_org.id)
            .await
            .expect("Failed to discover asset relationships");

        // We should have at least a subdomain relationship and a hosted-on relationship
        assert!(!relationships.is_empty());

        // Verify specific relationships exist
        let has_subdomain_rel = relationships
            .iter()
            .any(|(source_id, target_id, rel_type)| {
                let subdomain = assets
                    .iter()
                    .find(|a| a.value == "sub.test-domain.com")
                    .unwrap();
                let domain = assets
                    .iter()
                    .find(|a| a.value == "test-domain.com")
                    .unwrap();

                *source_id == subdomain.id
                    && *target_id == domain.id
                    && rel_type.to_lowercase().contains("subdomain")
            });

        let has_hosted_on_rel = relationships
            .iter()
            .any(|(source_id, target_id, rel_type)| {
                let web_app = assets
                    .iter()
                    .find(|a| a.asset_type == AssetType::WebApp)
                    .unwrap();
                let ip = assets
                    .iter()
                    .find(|a| a.asset_type == AssetType::IPAddress)
                    .unwrap();

                *source_id == web_app.id
                    && *target_id == ip.id
                    && rel_type.to_lowercase().contains("hosted")
            });

        assert!(has_subdomain_rel, "No subdomain relationship found");
        assert!(has_hosted_on_rel, "No hosted-on relationship found");

        // Test creating and querying relationships manually
        let web_app = assets
            .iter()
            .find(|a| a.asset_type == AssetType::WebApp)
            .unwrap();
        let domain = assets
            .iter()
            .find(|a| a.value == "test-domain.com")
            .unwrap();

        let rel_created = asset_service
            .create_asset_relationship(web_app.id, domain.id, "ManagedBy".to_string(), None)
            .await
            .expect("Failed to create relationship");

        assert!(rel_created);

        // Query related assets
        let related_assets = asset_service
            .get_related_assets(web_app.id, None)
            .await
            .expect("Failed to get related assets");

        assert!(!related_assets.is_empty());

        // Verify our manually created relationship exists
        let has_managed_by_rel = related_assets
            .iter()
            .any(|(asset, rel_type)| asset.id == domain.id && rel_type == "ManagedBy");

        assert!(
            has_managed_by_rel,
            "Manually created relationship not found"
        );

        // Clean up
        for asset in assets {
            let _ = factory
                .asset_repository()
                .delete_asset(asset.id)
                .await
                .expect("Failed to delete asset");
        }

        let _ = org_repo
            .delete_organization(created_org.id)
            .await
            .expect("Failed to delete organization");
    }

    #[tokio::test]
    async fn test_vulnerability_correlation() {
        // Set up the test database and repositories
        let factory = setup_test_db().await;
        let org_repo = factory.organization_repository();

        // Create a test organization
        let org_name = format!("Test Org Vulnerabilities {}", uuid::Uuid::new_v4());
        let org = Organization::new(org_name.clone());
        let created_org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Create test assets
        let assets = create_test_assets(&factory, created_org.id).await;

        // Create test vulnerabilities
        let vulnerabilities = create_test_vulnerabilities(&factory, &assets).await;

        // Create the vulnerability service
        let vuln_service = backend::services::VulnerabilityServiceImpl::new(
            factory.vulnerability_repository(),
            factory.asset_repository(),
        );

        // Test vulnerability correlation
        let correlations = vuln_service
            .correlate_vulnerabilities(created_org.id, None)
            .await
            .expect("Failed to correlate vulnerabilities");

        // We should have correlations since we created two similar SSL vulnerabilities
        assert!(!correlations.is_empty());

        // Find our SSL vulnerabilities to check correlation
        let ssl_vulns: Vec<&Vulnerability> = vulnerabilities
            .iter()
            .filter(|v| v.title.contains("SSL Certificate"))
            .collect();

        assert_eq!(ssl_vulns.len(), 2, "Expected 2 SSL vulnerabilities");

        // Check that they are correlated
        let vuln1_has_correlation = correlations.contains_key(&ssl_vulns[0].id)
            && correlations
                .get(&ssl_vulns[0].id)
                .unwrap()
                .contains(&ssl_vulns[1].id);

        let vuln2_has_correlation = correlations.contains_key(&ssl_vulns[1].id)
            && correlations
                .get(&ssl_vulns[1].id)
                .unwrap()
                .contains(&ssl_vulns[0].id);

        assert!(
            vuln1_has_correlation || vuln2_has_correlation,
            "SSL vulnerabilities not properly correlated"
        );

        // Test finding similar vulnerabilities
        let similar_vulns = vuln_service
            .find_similar_vulnerabilities(ssl_vulns[0].id, 10)
            .await
            .expect("Failed to find similar vulnerabilities");

        assert!(
            !similar_vulns.is_empty(),
            "No similar vulnerabilities found"
        );
        assert!(
            similar_vulns.iter().any(|v| v.id == ssl_vulns[1].id),
            "Expected SSL vulnerability in similar results"
        );

        // Clean up
        for vuln in vulnerabilities {
            let _ = factory
                .vulnerability_repository()
                .delete_vulnerability(vuln.id)
                .await
                .expect("Failed to delete vulnerability");
        }

        for asset in assets {
            let _ = factory
                .asset_repository()
                .delete_asset(asset.id)
                .await
                .expect("Failed to delete asset");
        }

        let _ = org_repo
            .delete_organization(created_org.id)
            .await
            .expect("Failed to delete organization");
    }

    #[tokio::test]
    async fn test_discovery_service_workflow() {
        // Set up the test database and repositories
        let factory = setup_test_db().await;
        let org_repo = factory.organization_repository();
        let _asset_repo = factory.asset_repository(); // Keep with underscore as may be used later

        // Create a test organization
        let org_name = format!("Test Org Discovery {}", uuid::Uuid::new_v4());
        let org = Organization::new(org_name.clone());
        let created_org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Create the discovery service
        let discovery_service =
            backend::services::DiscoveryServiceImpl::new(factory.asset_repository());

        // Test the discovery service for a domain
        let domain = "service-discovery-example.com";
        let job = discovery_service
            .discover_assets(created_org.id, domain, vec![])
            .await
            .expect("Failed to discover assets");

        // Basic verification that we get back a job
        assert_eq!(job.organization_id, created_org.id);
        assert!(job.target.is_some());
        assert_eq!(job.target.unwrap(), domain);

        // Clean up
        let _ = org_repo
            .delete_organization(created_org.id)
            .await
            .expect("Failed to delete organization");
    }
}
