use axum::{http::StatusCode, Json};
use serde_json::json;

/// Health check endpoint to verify API is running
pub async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}
