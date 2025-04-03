pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod state;
pub mod test_utils;

use std::net::SocketAddr;

use shared::{config::Config, errors::Result};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

use crate::routes::create_router;
use crate::state::AppState;

pub async fn run(config: Config) -> Result<()> {
    // Create the application state
    let state = AppState::new(&config).await?;

    // Set up tracing middleware
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(Level::INFO));

    // Build the router with routes
    let app = create_router(state).layer(trace_layer);

    // Build the server address
    let addr = SocketAddr::from((config.host, config.port));

    // Start the server
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        shared::errors::Error::external_service(format!("Failed to bind to address: {}", e))
    })?;

    axum::serve(listener, app)
        .await
        .map_err(|e| shared::errors::Error::external_service(format!("Server error: {}", e)))?;

    Ok(())
}
