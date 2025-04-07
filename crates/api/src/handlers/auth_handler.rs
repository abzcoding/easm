use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid;

use crate::{
    errors::{convert_result, ApiError, Result},
    middleware::auth::{generate_token, revoke_token, Claims},
    state::AppState,
};

// Request DTOs
#[derive(Deserialize)]
pub struct RegisterUserDto {
    pub organization_id: uuid::Uuid,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginUserDto {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshTokenDto {
    pub refresh_token: Option<String>,
}

// Response DTO
#[derive(Serialize, Deserialize)]
pub struct AuthResponseDto {
    pub token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
}

/// Register a new user
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterUserDto>,
) -> Result<impl IntoResponse> {
    // Validate that the passwords match
    if payload.password.is_empty() {
        return Err(ApiError::BadRequest("Password cannot be empty".to_string()));
    }

    // 2. Hash password (will be done in UserService)
    // 3. Call UserService to create user
    let user = convert_result(
        state
            .user_service
            .register_user(&payload.organization_id, &payload.email, &payload.password)
            .await,
    )?;

    // 4. Generate JWT token
    let token = generate_token(
        &user.id.to_string(),
        &format!("{:?}", user.role),
        Some(&user.organization_id.to_string()),
        &state.config,
    )?;

    // 5. Return token in response with expiration time
    Ok((
        StatusCode::CREATED,
        Json(AuthResponseDto {
            token,
            refresh_token: None,
            expires_in: state.config.jwt_expiration,
        }),
    ))
}

/// Login a user
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginUserDto>,
) -> Result<impl IntoResponse> {
    // 1. Validate input
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(ApiError::BadRequest(
            "Email and password cannot be empty".to_string(),
        ));
    }

    // 2. Call UserService to find user by email/username and verify password
    let user = convert_result(
        state
            .user_service
            .login_user(&payload.email, &payload.password)
            .await,
    )?;

    // 3. Generate JWT token
    let token = generate_token(
        &user.id.to_string(),
        &format!("{:?}", user.role),
        Some(&user.organization_id.to_string()),
        &state.config,
    )?;

    // 4. Generate refresh token (in a real implementation, this would be a separate token)
    let refresh_token = uuid::Uuid::new_v4().to_string();

    // 5. Return token in response
    Ok(Json(AuthResponseDto {
        token,
        refresh_token: Some(refresh_token),
        expires_in: state.config.jwt_expiration,
    }))
}

/// Logout a user
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse> {
    // Revoke the current token
    revoke_token(&claims.sub, &claims.jti, &state).await?;

    // Return success response
    Ok(StatusCode::NO_CONTENT)
}

/// Refresh an access token
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<RefreshTokenDto>,
) -> Result<impl IntoResponse> {
    // In a real implementation, validate the refresh token
    // For this example, we'll just generate a new token based on the existing claims

    // Revoke the current token
    revoke_token(&claims.sub, &claims.jti, &state).await?;

    // Generate a new token
    let new_token = generate_token(
        &claims.sub,
        &claims.role,
        claims.org.as_deref(),
        &state.config,
    )?;

    // Return the new token
    Ok(Json(AuthResponseDto {
        token: new_token,
        refresh_token: payload.refresh_token,
        expires_in: state.config.jwt_expiration,
    }))
}
