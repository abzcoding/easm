use api::test_utils::*;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

// Mock asset service for testing
#[derive(Clone)]
pub struct MockAssetService;

#[tokio::test]
async fn test_list_assets() {
    // Create the router with mock services
    let router = api::routes::create_router(create_test_app_state());
    let token = authenticate_test_user(&router).await;

    // Create a request to list assets
    let request = Request::builder()
        .uri("/api/assets")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    // Send the request to the router
    let response = router.oneshot(request).await.unwrap();

    // Check that the response has a 200 OK status
    assert_eq!(response.status(), StatusCode::OK);

    // Check the response body
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check that the response contains assets
    assert!(body.is_object());
    assert!(body["assets"].is_array());
    assert_eq!(body["assets"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_create_asset() {
    // Create the router with mock services
    let router = api::routes::create_router(create_test_app_state());
    let token = authenticate_test_user(&router).await;

    // Create asset payload
    let asset_data = json!({
        "organization_id": Uuid::new_v4().to_string(),
        "asset_type": "Domain",
        "value": "newtest.example.com",
        "attributes": {
            "hostname": "newtest"
        }
    });

    // Create a request to create a new asset
    let request = Request::builder()
        .uri("/api/assets")
        .method("POST")
        .header("Content-Type", "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(asset_data.to_string()))
        .unwrap();

    // Send the request to the router
    let response = router.oneshot(request).await.unwrap();

    // Check that the response has a 422 status (since MockAssetService::create_asset is unimplemented)
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_get_asset() {
    // Create the router with mock services
    let router = api::routes::create_router(create_test_app_state());
    let token = authenticate_test_user(&router).await;

    // Create a random asset ID
    let asset_id = Uuid::new_v4();

    // Create a request to get the asset
    let request = Request::builder()
        .uri(format!("/api/assets/{}", asset_id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    // Send the request to the router
    let response = router.oneshot(request).await.unwrap();

    // Check that the response has a 200 OK status
    assert_eq!(response.status(), StatusCode::OK);

    // Check the response body
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check that the response contains an asset with the correct ID
    assert_eq!(body["id"], asset_id.to_string());
    assert_eq!(body["value"], "test.example.com");
}

#[tokio::test]
async fn test_update_asset() {
    // Create the router with mock services
    let router = api::routes::create_router(create_test_app_state());
    let token = authenticate_test_user(&router).await;

    // Create a random asset ID
    let asset_id = Uuid::new_v4();

    // Create update asset payload
    let asset_data = json!({
        "id": asset_id.to_string(),
        "organization_id": Uuid::new_v4().to_string(),
        "asset_type": "Domain",
        "value": "updated.example.com",
        "status": "Active",
        "attributes": {
            "hostname": "updated"
        }
    });

    // Create a request to update the asset
    let request = Request::builder()
        .uri(format!("/api/assets/{}", asset_id))
        .method("PUT")
        .header("Content-Type", "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(asset_data.to_string()))
        .unwrap();

    // Send the request to the router
    let response = router.oneshot(request).await.unwrap();

    // Check that the response has a 422 status (since MockAssetService::update_asset is unimplemented)
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_delete_asset() {
    // Create the router with mock services
    let router = api::routes::create_router(create_test_app_state());
    let token = authenticate_test_user(&router).await;

    // Create a random asset ID
    let asset_id = Uuid::new_v4();

    // Create a request to delete the asset
    let request = Request::builder()
        .uri(format!("/api/assets/{}", asset_id))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    // Send the request to the router
    let response = router.oneshot(request).await.unwrap();

    // Check that the response has a 204 No Content status
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
