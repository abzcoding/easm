#[cfg(test)]
mod technology_integration_tests {
    use backend::models::{Asset, Organization, Technology};
    use chrono::Utc;
    use infrastructure::repositories::RepositoryFactory;
    use shared::{config::Config, types::AssetType};
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
    async fn test_technology_repository_operations() {
        let factory = setup_test_db().await;

        // Create repositories
        let org_repo = factory.organization_repository();
        let asset_repo = factory.asset_repository();
        let tech_repo = factory.technology_repository();

        // Create organization
        let org_name = format!("Tech Test Org {}", Uuid::new_v4());
        let org = Organization::new(org_name);
        let org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Create web app asset
        let web_asset = Asset::new(
            org.id,
            AssetType::WebApp,
            "https://techtest.example.com".to_string(),
            None,
        );
        let web_asset = asset_repo
            .create_asset(&web_asset)
            .await
            .expect("Failed to create web asset");

        // Create technologies for the web app
        let now = Utc::now();
        let technologies = vec![
            ("Nginx", "1.18.0", "Web Server"),
            ("PHP", "7.4.3", "Programming Language"),
            ("WordPress", "5.9", "CMS"),
            ("jQuery", "3.5.1", "JavaScript Library"),
        ];

        let mut created_techs = Vec::new();

        for (name, version, category) in technologies {
            let tech = Technology {
                id: Uuid::new_v4(),
                asset_id: web_asset.id,
                name: name.to_string(),
                version: Some(version.to_string()),
                category: Some(category.to_string()),
                created_at: now,
                updated_at: now,
            };

            let created_tech = tech_repo
                .create_technology(&tech)
                .await
                .expect("Failed to create technology");

            created_techs.push(created_tech);
        }

        // Test technology retrieval by asset
        let asset_techs = tech_repo
            .list_technologies(Some(web_asset.id), None, None, 100, 0)
            .await
            .expect("Failed to get technologies by asset");

        assert_eq!(asset_techs.len(), 4);

        // Test filtering by name
        let wordpress_techs = tech_repo
            .list_technologies(None, Some("WordPress".to_string()), None, 100, 0)
            .await
            .expect("Failed to get WordPress technologies");

        assert!(!wordpress_techs.is_empty());
        assert_eq!(wordpress_techs[0].name, "WordPress");
        assert_eq!(wordpress_techs[0].version.as_ref().unwrap(), "5.9");

        // Test filtering by category
        let web_server_techs = tech_repo
            .list_technologies(None, None, Some("Web Server".to_string()), 100, 0)
            .await
            .expect("Failed to get Web Server technologies");

        assert!(!web_server_techs.is_empty());
        assert_eq!(web_server_techs[0].name, "Nginx");

        // Test technology update
        let mut tech_to_update = created_techs[2].clone(); // WordPress
        tech_to_update.version = Some("6.0".to_string());

        let updated_tech = tech_repo
            .update_technology(&tech_to_update)
            .await
            .expect("Failed to update technology");

        assert_eq!(updated_tech.version.as_ref().unwrap(), "6.0");

        // Test technology deletion
        let _ = tech_repo
            .delete_technology(created_techs[3].id) // jQuery
            .await
            .expect("Failed to delete technology");

        let remaining_techs = tech_repo
            .list_technologies(Some(web_asset.id), None, None, 100, 0)
            .await
            .expect("Failed to get remaining technologies");

        assert_eq!(remaining_techs.len(), 3);

        // Clean up
        for tech in created_techs.iter().take(3) {
            let _ = tech_repo
                .delete_technology(tech.id)
                .await
                .expect("Failed to delete technology");
        }

        let _ = asset_repo
            .delete_asset(web_asset.id)
            .await
            .expect("Failed to delete asset");

        let _ = org_repo
            .delete_organization(org.id)
            .await
            .expect("Failed to delete organization");
    }

    #[tokio::test]
    async fn test_technology_detection_workflow() {
        let factory = setup_test_db().await;

        // Create repositories
        let org_repo = factory.organization_repository();
        let asset_repo = factory.asset_repository();
        let port_repo = factory.port_repository();
        let tech_repo = factory.technology_repository();

        // Create organization
        let org_name = format!("Tech Workflow Org {}", Uuid::new_v4());
        let org = Organization::new(org_name);
        let org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Create domain asset
        let domain = format!("tech-{}.example.com", Uuid::new_v4().as_simple());
        let domain_asset = Asset::new(org.id, AssetType::Domain, domain.clone(), None);
        let domain_asset = asset_repo
            .create_asset(&domain_asset)
            .await
            .expect("Failed to create domain asset");

        // Create web app asset
        let web_app_url = format!("https://{}", domain);
        let web_app_asset = Asset::new(
            org.id,
            AssetType::WebApp,
            web_app_url,
            Some(serde_json::json!({
                "domain": domain,
                "headers": {
                    "server": "nginx/1.18.0",
                    "x-powered-by": "PHP/7.4.3"
                }
            })),
        );
        let web_app_asset = asset_repo
            .create_asset(&web_app_asset)
            .await
            .expect("Failed to create web app asset");

        // Create technologies based on headers
        let now = Utc::now();
        let web_server_tech = Technology {
            id: Uuid::new_v4(),
            asset_id: web_app_asset.id,
            name: "Nginx".to_string(),
            version: Some("1.18.0".to_string()),
            category: Some("Web Server".to_string()),
            created_at: now,
            updated_at: now,
        };

        let web_server = tech_repo
            .create_technology(&web_server_tech)
            .await
            .expect("Failed to create web server technology");

        let language_tech = Technology {
            id: Uuid::new_v4(),
            asset_id: web_app_asset.id,
            name: "PHP".to_string(),
            version: Some("7.4.3".to_string()),
            category: Some("Programming Language".to_string()),
            created_at: now,
            updated_at: now,
        };

        let language = tech_repo
            .create_technology(&language_tech)
            .await
            .expect("Failed to create language technology");

        // Simulate content scan finding WordPress
        let cms_tech = Technology {
            id: Uuid::new_v4(),
            asset_id: web_app_asset.id,
            name: "WordPress".to_string(),
            version: Some("5.9".to_string()),
            category: Some("CMS".to_string()),
            created_at: now,
            updated_at: now,
        };

        let cms = tech_repo
            .create_technology(&cms_tech)
            .await
            .expect("Failed to create CMS technology");

        // Query for all technologies on the web app
        let web_app_techs = tech_repo
            .list_technologies(Some(web_app_asset.id), None, None, 100, 0)
            .await
            .expect("Failed to get web app technologies");

        assert_eq!(web_app_techs.len(), 3);

        // Verify that we can find all assets with WordPress
        let wordpress_techs = tech_repo
            .list_technologies(None, Some("WordPress".to_string()), None, 100, 0)
            .await
            .expect("Failed to get WordPress technologies");

        assert!(!wordpress_techs.is_empty());

        let wp_asset_ids: Vec<Uuid> = wordpress_techs.iter().map(|t| t.asset_id).collect();
        assert!(wp_asset_ids.contains(&web_app_asset.id));

        // Clean up
        let _ = tech_repo
            .delete_technology(web_server.id)
            .await
            .expect("Failed to delete web server technology");

        let _ = tech_repo
            .delete_technology(language.id)
            .await
            .expect("Failed to delete language technology");

        let _ = tech_repo
            .delete_technology(cms.id)
            .await
            .expect("Failed to delete CMS technology");

        let _ = asset_repo
            .delete_asset(web_app_asset.id)
            .await
            .expect("Failed to delete web app asset");

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
