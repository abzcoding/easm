#[cfg(test)]
mod port_integration_tests {
    use backend::models::{Asset, Organization, Port};
    use infrastructure::repositories::RepositoryFactory;
    use shared::{
        config::Config,
        types::{AssetType, PortStatus, Protocol},
    };
    use uuid::Uuid;

    async fn setup_test_db() -> RepositoryFactory {
        let _ = dotenvy::dotenv();
        let config = Config::from_env().expect("Failed to load config");
        let database_url = config.database_url.clone();

        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        RepositoryFactory::new(pool)
    }

    #[tokio::test]
    async fn test_port_repository_operations() {
        let factory = setup_test_db().await;

        // Create repositories
        let org_repo = factory.organization_repository();
        let asset_repo = factory.asset_repository();
        let port_repo = factory.port_repository();

        // Create organization
        let org_name = format!("Port Test Org {}", Uuid::new_v4());
        let org = Organization::new(org_name);
        let org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Create IP address asset
        let ip_asset = Asset::new(
            org.id,
            AssetType::IPAddress,
            "192.168.1.10".to_string(),
            None,
        );
        let ip_asset = asset_repo
            .create_asset(&ip_asset)
            .await
            .expect("Failed to create IP asset");

        // Create ports for the IP
        let ports = vec![
            (80, Protocol::TCP, "http", "Apache/2.4.41"),
            (443, Protocol::TCP, "https", "nginx/1.18.0"),
            (22, Protocol::TCP, "ssh", "OpenSSH_8.2p1"),
            (53, Protocol::UDP, "domain", ""),
        ];

        let mut created_ports = Vec::new();
        for (port_number, protocol, service_name, banner) in ports {
            let port = Port {
                id: Uuid::new_v4(),
                asset_id: ip_asset.id,
                port_number,
                protocol,
                service_name: Some(service_name.to_string()),
                banner: if banner.is_empty() {
                    None
                } else {
                    Some(banner.to_string())
                },
                status: PortStatus::Open,
                first_seen: chrono::Utc::now(),
                last_seen: chrono::Utc::now(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            let created_port = port_repo
                .create_port(&port)
                .await
                .expect("Failed to create port");

            created_ports.push(created_port);
        }

        // Test port retrieval by asset
        let asset_ports = port_repo
            .list_ports(Some(ip_asset.id), None, None, None, 10, 0)
            .await
            .expect("Failed to get ports by asset");

        assert_eq!(asset_ports.len(), 4);

        // Test filtering by protocol
        let tcp_ports = port_repo
            .list_ports(Some(ip_asset.id), None, Some(Protocol::TCP), None, 10, 0)
            .await
            .expect("Failed to get TCP ports");

        assert_eq!(tcp_ports.len(), 3);

        // Test port update
        let mut port_to_update = created_ports[0].clone();
        port_to_update.status = PortStatus::Closed;

        let updated_port = port_repo
            .update_port(&port_to_update)
            .await
            .expect("Failed to update port");

        assert_eq!(updated_port.status, PortStatus::Closed);

        // Test port deletion
        let _ = port_repo
            .delete_port(created_ports[3].id)
            .await
            .expect("Failed to delete port");

        let remaining_ports = port_repo
            .list_ports(Some(ip_asset.id), None, None, None, 10, 0)
            .await
            .expect("Failed to get remaining ports");

        assert_eq!(remaining_ports.len(), 3);

        // Clean up
        for port in created_ports.iter().take(3) {
            let _ = port_repo
                .delete_port(port.id)
                .await
                .expect("Failed to delete port");
        }

        let _ = asset_repo
            .delete_asset(ip_asset.id)
            .await
            .expect("Failed to delete asset");

        let _ = org_repo
            .delete_organization(org.id)
            .await
            .expect("Failed to delete organization");
    }

    #[tokio::test]
    async fn test_port_scan_asset_relation() {
        let factory = setup_test_db().await;

        // Create repositories
        let org_repo = factory.organization_repository();
        let asset_repo = factory.asset_repository();
        let port_repo = factory.port_repository();

        // Create organization
        let org_name = format!("Port Relation Org {}", Uuid::new_v4());
        let org = Organization::new(org_name);
        let org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Create domain asset
        let domain = format!("domain-{}.example.com", Uuid::new_v4().as_simple());
        let domain_asset = Asset::new(org.id, AssetType::Domain, domain.clone(), None);
        let domain_asset = asset_repo
            .create_asset(&domain_asset)
            .await
            .expect("Failed to create domain asset");

        // Create IP address asset
        let ip_address = "10.0.0.1";
        let ip_asset = Asset::new(
            org.id,
            AssetType::IPAddress,
            ip_address.to_string(),
            Some(serde_json::json!({
                "resolved_from": domain
            })),
        );
        let ip_asset = asset_repo
            .create_asset(&ip_asset)
            .await
            .expect("Failed to create IP asset");

        // Create web app asset
        let web_app_url = format!("https://{}", domain);
        let web_app_asset = Asset::new(
            org.id,
            AssetType::WebApp,
            web_app_url,
            Some(serde_json::json!({
                "domain": domain,
                "ip_address": ip_address
            })),
        );
        let web_app_asset = asset_repo
            .create_asset(&web_app_asset)
            .await
            .expect("Failed to create web app asset");

        // Create ports for the IP
        let web_port = Port {
            id: Uuid::new_v4(),
            asset_id: ip_asset.id,
            port_number: 443,
            protocol: Protocol::TCP,
            service_name: Some("https".to_string()),
            banner: Some("nginx/1.18.0".to_string()),
            status: PortStatus::Open,
            first_seen: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let created_port = port_repo
            .create_port(&web_port)
            .await
            .expect("Failed to create port");

        // Verify relationships
        let domain_assets = asset_repo
            .list_assets(Some(org.id), Some(AssetType::Domain), None, 10, 0)
            .await
            .expect("Failed to list domain assets");

        let ip_assets = asset_repo
            .list_assets(Some(org.id), Some(AssetType::IPAddress), None, 10, 0)
            .await
            .expect("Failed to list IP assets");

        let web_app_assets = asset_repo
            .list_assets(Some(org.id), Some(AssetType::WebApp), None, 10, 0)
            .await
            .expect("Failed to list web app assets");

        assert_eq!(domain_assets.len(), 1);
        assert_eq!(ip_assets.len(), 1);
        assert_eq!(web_app_assets.len(), 1);

        // Verify port relationship
        let ip_ports = port_repo
            .list_ports(Some(ip_asset.id), None, None, None, 10, 0)
            .await
            .expect("Failed to get ports by asset");

        assert_eq!(ip_ports.len(), 1);
        assert_eq!(ip_ports[0].port_number, 443);
        assert_eq!(ip_ports[0].service_name.as_ref().unwrap(), "https");

        // Clean up
        let _ = port_repo
            .delete_port(created_port.id)
            .await
            .expect("Failed to delete port");

        let _ = asset_repo
            .delete_asset(web_app_asset.id)
            .await
            .expect("Failed to delete web app asset");

        let _ = asset_repo
            .delete_asset(ip_asset.id)
            .await
            .expect("Failed to delete IP asset");

        let _ = asset_repo
            .delete_asset(domain_asset.id)
            .await
            .expect("Failed to delete domain asset");

        let _ = org_repo
            .delete_organization(org.id)
            .await
            .expect("Failed to delete organization");
    }
}
