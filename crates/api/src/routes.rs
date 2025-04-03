use axum::{middleware::from_fn_with_state, routing::get, Router};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    handlers::{
        asset_handler::{create_asset, delete_asset, get_asset, list_assets, update_asset},
        auth_handler::{login, register},
        health_handler::health_check,
        organization_handler::{
            create_organization, delete_organization, get_organization, list_organizations,
            update_organization,
        },
        vulnerability_handler::{
            create_vulnerability, delete_vulnerability, get_vulnerability, list_vulnerabilities,
            update_vulnerability,
        },
    },
    middleware::auth::auth_middleware,
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
        // API routes
        // Auth routes (NO middleware)
        .route("/api/auth/register", axum::routing::post(register))
        .route("/api/auth/login", axum::routing::post(login))
        .route(
            "/api/organizations",
            get(list_organizations).post(create_organization),
        )
        .route(
            "/api/organizations/{id}",
            get(get_organization)
                .put(update_organization)
                .delete(delete_organization),
        )
        // Assets API
        .route("/api/assets", get(list_assets).post(create_asset))
        .route(
            "/api/assets/{id}",
            get(get_asset).put(update_asset).delete(delete_asset),
        )
        // Vulnerabilities API
        .route(
            "/api/vulnerabilities",
            get(list_vulnerabilities).post(create_vulnerability),
        )
        .route(
            "/api/vulnerabilities/{id}",
            get(get_vulnerability)
                .put(update_vulnerability)
                .delete(delete_vulnerability),
        )
        // Apply auth middleware to these routes
        //.route_layer(from_fn_with_state(state.clone(), auth_middleware))
        // Add state
        .with_state(state)
        // Add middleware (cors applies to all routes, including /health)
        // TraceLayer also applies to all routes
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
