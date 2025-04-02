use axum::{
    middleware::map_response_with_state,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    handlers::{
        asset_handler::{create_asset, delete_asset, get_asset, list_assets, update_asset},
        health_handler::health_check,
        vulnerability_handler::{
            create_vulnerability, delete_vulnerability, get_vulnerability, list_vulnerabilities,
            update_vulnerability,
        },
    },
    middleware::auth::auth,
    state::AppState,
};

/// Creates the main router with all routes
pub fn create_router(state: AppState) -> Router {
    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Wrap the state in an Arc
    let state = Arc::new(state);

    // Create a router with all routes
    Router::new()
        // Health check endpoint (no auth)
        .route("/health", get(health_check))
        // API routes (with auth)
        .nest(
            "/api",
            Router::new()
                // Assets API
                .route("/assets", get(list_assets).post(create_asset))
                .route(
                    "/assets/:id",
                    get(get_asset).put(update_asset).delete(delete_asset),
                )
                // Vulnerabilities API
                .route(
                    "/vulnerabilities",
                    get(list_vulnerabilities).post(create_vulnerability),
                )
                .route(
                    "/vulnerabilities/:id",
                    get(get_vulnerability)
                        .put(update_vulnerability)
                        .delete(delete_vulnerability),
                ),
        )
        // Add state
        .with_state(state)
        // Add middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
