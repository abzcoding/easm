use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use shared::config::Config;

use crate::{errors::ApiError, state::AppState};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub sub: String,         // Subject (user ID)
    pub role: String,        // User role
    pub org: Option<String>, // Organization ID
    pub exp: usize,          // Expiration time
    pub iat: usize,          // Issued at
}

/// Authentication middleware
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
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
    let _claims = match decode::<Claims>(
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

/// Generate JWT token for authenticated user
pub fn generate_token(
    user_id: &str,
    role: &str,
    organization_id: Option<&str>,
    config: &Config,
) -> Result<String, ApiError> {
    // Calculate expiration time (24 hours from now)
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    // Calculate issued at time
    let issued_at = Utc::now().timestamp() as usize;

    // Create the claims
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        org: organization_id.map(|id| id.to_string()),
        exp: expiration,
        iat: issued_at,
    };

    // Encode the token
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| ApiError::InternalServerError(format!("Token generation error: {}", e)))
}
