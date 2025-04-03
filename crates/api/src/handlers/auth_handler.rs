use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use std::sync::Arc;
use uuid;

use crate::{errors::ApiError, middleware::auth::generate_token, state::AppState};

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

// Response DTO
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthResponseDto {
    pub token: String,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterUserDto>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Validate input (basic checks)
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(ApiError::BadRequest(
            "Organization ID, email and password cannot be empty".to_string(),
        ));
    }
    // More robust validation (email format, password strength) should be added

    // 2. Hash password (will be done in UserService)
    // 3. Call UserService to create user
    let user = state
        .user_service
        .register_user(&payload.organization_id, &payload.email, &payload.password)
        .await?;

    // 4. Generate JWT token
    let token = generate_token(
        &user.id.to_string(),
        &format!("{:?}", user.role),
        Some(&user.organization_id.to_string()),
        &state.config,
    )?;

    // 5. Return token in response
    Ok((StatusCode::CREATED, Json(AuthResponseDto { token })))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginUserDto>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: Implement user login logic
    // 1. Validate input
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(ApiError::BadRequest(
            "Email and password cannot be empty".to_string(),
        ));
    }

    // 2. Call UserService to find user by email/username and verify password
    let user = state
        .user_service
        .login_user(&payload.email, &payload.password)
        .await?;

    // 4. Generate JWT token
    let token = generate_token(
        &user.id.to_string(),
        &format!("{:?}", user.role),
        Some(&user.organization_id.to_string()),
        &state.config,
    )?;

    // 5. Return token in response
    Ok(Json(AuthResponseDto { token }))
}
