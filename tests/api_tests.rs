#[cfg(test)]
mod api_integration_tests {
    use api::{routes::create_router, state::AppState};
    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use backend::models::{Asset, Vulnerability};
    use http_body_util::BodyExt;
    use infrastructure::repositories::RepositoryFactory;
    use shared::{
        config::Config,
        types::{AssetType, Severity},
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    /// Sets up a test database and returns a repository factory
    async fn setup_test_db() -> RepositoryFactory {
        // Load environment variables
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

    /// Sets up the test application state
    async fn setup_test_app_state() -> AppState {
        let config = Config::from_env().expect("Failed to load config");
        AppState::new(&config)
            .await
            .expect("Failed to create app state")
    }

    /// Creates a test router
    async fn setup_test_router() -> axum::Router {
        let state = setup_test_app_state().await;
        create_router(state)
    }

    #[tokio::test]
    async fn test_health_check() {
        // Arrange
        let app = setup_test_router().await;

        // Act
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .method(http::Method::GET)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(body["status"], "ok");
    }

    #[tokio::test]
    async fn test_asset_crud_operations() {
        // Arrange
        let factory = setup_test_db().await;
        let org_repo = factory.organization_repository();

        // Create a test organization
        let org_name = format!("Test API Org {}", Uuid::new_v4());
        let org = backend::models::Organization::new(org_name);
        let org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Setup the router
        let app = setup_test_router().await;

        // Test - Create Asset
        let asset_value = format!("api-test-{}.example.com", Uuid::new_v4());
        let asset_id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let create_asset_json = serde_json::json!({
            "id": asset_id,
            "organization_id": org.id,
            "asset_type": "DOMAIN",
            "value": asset_value,
            "status": "ACTIVE",
            "first_seen": now,
            "last_seen": now,
            "created_at": now,
            "updated_at": now,
            "attributes": {}
        });

        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/assets")
                    .method(http::Method::POST)
                    .header("Content-Type", "application/json")
                    .body(Body::from(create_asset_json.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = create_response.status();
        let body_bytes = create_response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();

        if status != StatusCode::CREATED {
            let body_str = String::from_utf8_lossy(&body_bytes);
            println!("DEBUG - Asset Creation Error: {} - {}", status, body_str);
            panic!("Expected status 201 but got {}", status);
        }

        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("DEBUG - Asset Response Body: {}", body_str);
        let asset: Asset = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(asset.value, asset_value);
        assert_eq!(asset.asset_type, AssetType::Domain);

        // Test - Get Asset
        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/assets/{}", asset.id))
                    .method(http::Method::GET)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(get_response.status(), StatusCode::OK);

        let body = get_response.into_body().collect().await.unwrap().to_bytes();
        let retrieved_asset: Asset = serde_json::from_slice(&body).unwrap();

        assert_eq!(retrieved_asset.id, asset.id);
        assert_eq!(retrieved_asset.value, asset_value);

        // Test - Update Asset
        let updated_value = format!("updated-{}", asset_value);
        let now_update = chrono::Utc::now();
        let update_asset_json = serde_json::json!({
            "id": asset.id,
            "organization_id": org.id,
            "asset_type": "DOMAIN",
            "value": updated_value,
            "status": "ACTIVE",
            "first_seen": asset.first_seen,
            "last_seen": asset.last_seen,
            "created_at": asset.created_at,
            "updated_at": now_update,
            "attributes": asset.attributes
        });

        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/assets/{}", asset.id))
                    .method(http::Method::PUT)
                    .header("Content-Type", "application/json")
                    .body(Body::from(update_asset_json.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(update_response.status(), StatusCode::OK);

        let body = update_response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
        let updated_asset: Asset = serde_json::from_slice(&body).unwrap();

        assert_eq!(updated_asset.value, updated_value);

        // Test - List Assets
        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/assets?organization_id={}", org.id))
                    .method(http::Method::GET)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(list_response.status(), StatusCode::OK);

        let body_bytes = list_response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("DEBUG - Asset List Response Body: {}", body_str);

        // Parse as generic JSON first
        let json_value: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        // Extract the assets array
        let assets: Vec<Asset> = if json_value.is_array() {
            serde_json::from_value(json_value).unwrap()
        } else if let Some(assets_value) = json_value.get("assets") {
            // If there's an "assets" field, extract it
            serde_json::from_value(assets_value.clone()).unwrap()
        } else {
            // If it's a single item, create a vector with one item
            let asset: Asset = serde_json::from_value(json_value).unwrap();
            vec![asset]
        };

        assert!(!assets.is_empty());
        if !assets.is_empty() {
            let found = assets.iter().any(|a| a.id == asset.id);
            println!("DEBUG - Asset ID {} found in response: {}", asset.id, found);
            assert!(
                found,
                "Expected asset with ID {} to be in the response",
                asset.id
            );
        }

        // Test - Delete Asset
        let delete_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/assets/{}", asset.id))
                    .method(http::Method::DELETE)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

        // Cleanup - Delete organization
        let _ = org_repo
            .delete_organization(org.id)
            .await
            .expect("Failed to delete organization");
    }

    #[tokio::test]
    async fn test_vulnerability_crud_operations() {
        // Arrange
        let factory = setup_test_db().await;
        let org_repo = factory.organization_repository();
        let asset_repo = factory.asset_repository();

        // Create a test organization
        let org_name = format!("Test API Org for Vulns {}", Uuid::new_v4());
        let org = backend::models::Organization::new(org_name);
        let org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        // Create a test asset
        let asset = Asset::new(
            org.id,
            AssetType::Domain,
            format!("vuln-api-test-{}.example.com", Uuid::new_v4()),
            None,
        );
        let asset = asset_repo
            .create_asset(&asset)
            .await
            .expect("Failed to create asset");

        // Setup the router
        let app = setup_test_router().await;

        // Test - Create Vulnerability
        let vuln_title = format!("Test Vulnerability {}", Uuid::new_v4());
        let vuln_id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let create_vuln_json = serde_json::json!({
            "id": vuln_id,
            "asset_id": asset.id,
            "title": vuln_title,
            "description": "This is a test vulnerability",
            "severity": "MEDIUM",
            "status": "OPEN",
            "evidence": "Test evidence data",
            "first_seen": now,
            "last_seen": now,
            "created_at": now,
            "updated_at": now
        });

        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/vulnerabilities")
                    .method(http::Method::POST)
                    .header("Content-Type", "application/json")
                    .body(Body::from(create_vuln_json.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = create_response.status();
        let body_bytes = create_response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();

        if status != StatusCode::CREATED {
            let body_str = String::from_utf8_lossy(&body_bytes);
            println!(
                "DEBUG - Vulnerability Creation Error: {} - {}",
                status, body_str
            );
            panic!("Expected status 201 but got {}", status);
        }

        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("DEBUG - Vulnerability Response Body: {}", body_str);
        let vuln: Vulnerability = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(vuln.title, vuln_title);
        assert_eq!(vuln.severity, Severity::Medium);

        // Test - Get Vulnerability
        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/vulnerabilities/{}", vuln.id))
                    .method(http::Method::GET)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(get_response.status(), StatusCode::OK);

        let body = get_response.into_body().collect().await.unwrap().to_bytes();
        let retrieved_vuln: Vulnerability = serde_json::from_slice(&body).unwrap();

        assert_eq!(retrieved_vuln.id, vuln.id);
        assert_eq!(retrieved_vuln.title, vuln_title);

        // Test - Update Vulnerability
        let updated_title = format!("Updated {}", vuln_title);
        let now_update = chrono::Utc::now();
        let update_vuln_json = serde_json::json!({
            "id": vuln.id,
            "asset_id": asset.id,
            "title": updated_title,
            "description": "This is an updated test vulnerability",
            "severity": "HIGH",
            "status": "CLOSED",
            "evidence": vuln.evidence,
            "first_seen": vuln.first_seen,
            "last_seen": vuln.last_seen,
            "created_at": vuln.created_at,
            "updated_at": now_update
        });

        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/vulnerabilities/{}", vuln.id))
                    .method(http::Method::PUT)
                    .header("Content-Type", "application/json")
                    .body(Body::from(update_vuln_json.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = update_response.status();
        let body_bytes = update_response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("DEBUG - Vulnerability Update Response Body: {}", body_str);

        if status != StatusCode::OK {
            println!(
                "DEBUG - Vulnerability Update Error: {} - {}",
                status, body_str
            );
            assert_eq!(status, StatusCode::OK);
        }

        let updated_vuln: Vulnerability = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(updated_vuln.title, updated_title);
        assert_eq!(updated_vuln.severity, Severity::High);

        // Test - List Vulnerabilities
        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/vulnerabilities?asset_id={}", asset.id))
                    .method(http::Method::GET)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(list_response.status(), StatusCode::OK);

        let body_bytes = list_response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("DEBUG - Vulnerability List Response Body: {}", body_str);

        // Parse as generic JSON first
        let json_value: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        // Extract the vulnerabilities array
        let vulns: Vec<Vulnerability> = if json_value.is_array() {
            serde_json::from_value(json_value).unwrap()
        } else if let Some(vulns_value) = json_value.get("vulnerabilities") {
            // If there's a "vulnerabilities" field, extract it
            serde_json::from_value(vulns_value.clone()).unwrap()
        } else {
            // If it's a single item, create a vector with one item
            let vuln: Vulnerability = serde_json::from_value(json_value).unwrap();
            vec![vuln]
        };

        assert!(!vulns.is_empty());
        if !vulns.is_empty() {
            let found = vulns.iter().any(|v| v.id == vuln.id);
            println!("DEBUG - Vuln ID {} found in response: {}", vuln.id, found);
            assert!(
                found,
                "Expected vulnerability with ID {} to be in the response",
                vuln.id
            );
        }

        // Test - Delete Vulnerability
        let delete_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/vulnerabilities/{}", vuln.id))
                    .method(http::Method::DELETE)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

        // Cleanup - Delete asset and organization
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
    async fn test_error_handling() {
        // Arrange
        let app = setup_test_router().await;

        // Test - Not found endpoint
        let not_found_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/nonexistent-endpoint")
                    .method(http::Method::GET)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(not_found_response.status(), StatusCode::NOT_FOUND);

        // Test - Not found resource
        let invalid_id = Uuid::new_v4();
        let not_found_resource_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/assets/{}", invalid_id))
                    .method(http::Method::GET)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(not_found_resource_response.status(), StatusCode::NOT_FOUND);

        // Test - Bad request
        let bad_json = r#"{"invalid_json": "#;
        let bad_request_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/assets")
                    .method(http::Method::POST)
                    .header("Content-Type", "application/json")
                    .body(Body::from(bad_json))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(bad_request_response.status(), StatusCode::BAD_REQUEST);
    }
}
