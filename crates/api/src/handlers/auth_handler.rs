use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use std::sync::Arc;
use uuid;

use crate::{
    errors::{convert_result, ApiError, Result},
    middleware::auth::generate_token,
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

// Response DTO
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthResponseDto {
    pub token: String,
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

    // 5. Return token in response
    Ok((StatusCode::CREATED, Json(AuthResponseDto { token })))
}

/// Login a user
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginUserDto>,
) -> Result<impl IntoResponse> {
    // TODO: Implement user login logic
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
