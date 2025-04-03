use axum::{
    middleware::from_fn_with_state,
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
        auth_handler::{login, register},
        discovery_task_handler::{
            cancel_discovery_task, create_discovery_task, delete_discovery_task,
            get_discovery_task, list_discovery_tasks,
        },
        health_handler::health_check,
        organization_handler::{
            create_organization, delete_organization, get_organization, list_organizations,
            update_organization,
        },
        report_handler,
        vulnerability_handler::{
            correlate_vulnerabilities, create_vulnerability, delete_vulnerability,
            find_similar_vulnerabilities, get_vulnerability, list_vulnerabilities,
            update_vulnerability,
        },
    },
    middleware::auth::{
        auth_middleware, require_admin, require_asset_modification, require_user_management,
        require_vulnerability_modification,
    },
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
        // Auth routes (NO middleware)
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        // Protected routes with authentication
        .nest(
            "/api",
            Router::new()
                // Organization management - admin or manager only
                .route("/organizations", get(list_organizations))
                .route(
                    "/organizations",
                    post(create_organization)
                        .route_layer(from_fn_with_state(state.clone(), require_admin)),
                )
                .route("/organizations/{id}", get(get_organization))
                .route(
                    "/organizations/{id}",
                    axum::routing::put(update_organization)
                        .route_layer(from_fn_with_state(state.clone(), require_user_management)),
                )
                .route(
                    "/organizations/{id}",
                    axum::routing::delete(delete_organization)
                        .route_layer(from_fn_with_state(state.clone(), require_admin)),
                )
                // Assets API - different permissions for different actions
                .route("/assets", get(list_assets))
                .route(
                    "/assets",
                    post(create_asset).route_layer(from_fn_with_state(
                        state.clone(),
                        require_asset_modification,
                    )),
                )
                .route("/assets/{id}", get(get_asset))
                .route(
                    "/assets/{id}",
                    axum::routing::put(update_asset).route_layer(from_fn_with_state(
                        state.clone(),
                        require_asset_modification,
                    )),
                )
                .route(
                    "/assets/{id}",
                    axum::routing::delete(delete_asset).route_layer(from_fn_with_state(
                        state.clone(),
                        require_asset_modification,
                    )),
                )
                // Discovery Tasks API
                .route("/discovery-tasks", get(list_discovery_tasks))
                .route(
                    "/discovery-tasks",
                    post(create_discovery_task).route_layer(from_fn_with_state(
                        state.clone(),
                        require_asset_modification,
                    )),
                )
                .route("/discovery-tasks/{id}", get(get_discovery_task))
                .route(
                    "/discovery-tasks/{id}/cancel",
                    post(cancel_discovery_task).route_layer(from_fn_with_state(
                        state.clone(),
                        require_asset_modification,
                    )),
                )
                .route(
                    "/discovery-tasks/{id}",
                    axum::routing::delete(delete_discovery_task).route_layer(from_fn_with_state(
                        state.clone(),
                        require_asset_modification,
                    )),
                )
                // Vulnerabilities API - different permissions for different actions
                .route("/vulnerabilities", get(list_vulnerabilities))
                .route(
                    "/vulnerabilities",
                    post(create_vulnerability).route_layer(from_fn_with_state(
                        state.clone(),
                        require_vulnerability_modification,
                    )),
                )
                .route("/vulnerabilities/{id}", get(get_vulnerability))
                .route(
                    "/vulnerabilities/{id}",
                    axum::routing::put(update_vulnerability).route_layer(from_fn_with_state(
                        state.clone(),
                        require_vulnerability_modification,
                    )),
                )
                .route(
                    "/vulnerabilities/{id}",
                    axum::routing::delete(delete_vulnerability).route_layer(from_fn_with_state(
                        state.clone(),
                        require_vulnerability_modification,
                    )),
                )
                // Vulnerability correlation endpoints - available to all authenticated users
                .route("/vulnerabilities/correlate", get(correlate_vulnerabilities))
                .route(
                    "/vulnerabilities/{id}/similar",
                    get(find_similar_vulnerabilities),
                )
                // Reports
                .route(
                    "/reports/vulnerabilities",
                    get(report_handler::generate_vulnerability_report),
                )
                .route(
                    "/reports/assets",
                    get(report_handler::generate_asset_report),
                )
                .route("/reports/{report_id}", get(report_handler::download_report))
                // Apply authentication middleware to all routes under /api
                .route_layer(from_fn_with_state(state.clone(), auth_middleware)),
        )
        // Add state
        .with_state(state)
        // Add middleware (cors applies to all routes, including /health)
        // TraceLayer also applies to all routes
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
