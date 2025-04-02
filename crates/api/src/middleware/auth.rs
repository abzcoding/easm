use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use shared::config::Config;

use crate::{errors::ApiError, state::AppState};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub sub: String,         // Subject (user ID)
    pub role: String,        // User role
    pub org: Option<String>, // Organization ID
    pub exp: usize,          // Expiration time
}

/// Authentication middleware
pub async fn auth_middleware<B>(
    State(state): State<Arc<AppState>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError> {
    // Skip auth for health check endpoint
    if req.uri().path() == "/health" {
        return Ok(next.run(req).await);
    }

    // Get authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(ApiError::Unauthorized);
    };

    // Check if it's a Bearer token
    if !auth_header.starts_with("Bearer ") {
        return Err(ApiError::Unauthorized);
    }

    // Extract the token
    let token = &auth_header["Bearer ".len()..];

    // Validate token
    let claims = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(claims) => claims.claims,
        Err(e) => match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                return Err(ApiError::TokenExpired)
            }
            _ => return Err(ApiError::InvalidToken),
        },
    };

    // Continue to the next handler
    Ok(next.run(req).await)
}

/// Create auth middleware layer
pub fn auth<S>(
    state: Arc<AppState>,
) -> axum::middleware::from_fn_with_state<
    Arc<AppState>,
    fn(
        State<Arc<AppState>>,
        Request<axum::body::Body>,
        Next<axum::body::Body>,
    ) -> impl axum::response::IntoResponse,
    S,
> {
    axum::middleware::from_fn_with_state(state, auth_middleware)
}

/// Generate JWT token for authenticated user
pub fn generate_token(
    _user_id: &str,
    _role: &str,
    _organization_id: Option<&str>,
    _config: &Config,
) -> Result<String, ApiError> {
    // This would be implemented in a real application
    // For now, return an error
    Err(ApiError::InternalServerError(
        "Token generation not implemented".to_string(),
    ))
}
