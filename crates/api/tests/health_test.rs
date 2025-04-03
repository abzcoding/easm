use api::test_utils::*;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint() {
    // Create a mock config
    let router = api::routes::create_router(create_test_app_state());
    // Create a request to the health endpoint
    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    // Send the request to the router
    let response = router.oneshot(request).await.unwrap();

    // Check that the response has a 200 OK status
    assert_eq!(response.status(), StatusCode::OK);

    // Check the response body
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check that the response contains the expected data
    assert_eq!(body["status"], "ok");
}
