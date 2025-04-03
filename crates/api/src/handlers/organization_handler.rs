use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    errors::{convert_result, ApiError, Result},
    state::AppState,
};
use backend::models::Organization; // Use trait instead of impl
use shared::types::{PaginationParams, ID}; // Use ID alias

// DTOs
#[derive(Deserialize)]
pub struct CreateOrganizationDto {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateOrganizationDto {
    pub name: String,
}

// Handlers
pub async fn create_organization(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateOrganizationDto>,
) -> Result<impl IntoResponse> {
    // TODO: Add authorization check (e.g., only Admins can create orgs?)
    if payload.name.is_empty() {
        return Err(ApiError::BadRequest(
            "Organization name cannot be empty".to_string(),
        ));
    }
    let org = Organization::new(payload.name);
    let created_org = convert_result(state.organization_service.create_organization(&org).await)?;
    Ok((StatusCode::CREATED, Json(created_org)))
}

pub async fn get_organization(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<ID>, // Use ID alias
) -> Result<impl IntoResponse> {
    // TODO: Add authorization check (e.g., user belongs to this org or is admin)
    let org = convert_result(state.organization_service.get_organization(org_id).await)?;
    Ok(Json(org))
}

pub async fn list_organizations(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse> {
    // TODO: Add authorization check (e.g., only Admins can list all orgs?)
    let orgs = convert_result(
        state
            .organization_service
            .list_organizations(pagination.page_size as usize, pagination.offset() as usize)
            .await,
    )?;
    Ok(Json(orgs))
}

pub async fn update_organization(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<ID>, // Use ID alias
    Json(payload): Json<UpdateOrganizationDto>,
) -> Result<impl IntoResponse> {
    // TODO: Add authorization check
    if payload.name.is_empty() {
        return Err(ApiError::BadRequest(
            "Organization name cannot be empty".to_string(),
        ));
    }
    // Get existing org first to update it
    let mut org = convert_result(state.organization_service.get_organization(org_id).await)?;
    org.name = payload.name;
    // Note: Need to handle updated_at timestamp, likely in service/repo
    let updated_org = convert_result(state.organization_service.update_organization(&org).await)?;
    Ok(Json(updated_org))
}

pub async fn delete_organization(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<ID>, // Use ID alias
) -> Result<impl IntoResponse> {
    // TODO: Add authorization check
    let deleted = convert_result(state.organization_service.delete_organization(org_id).await)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        // This case might not happen if get_organization is checked first in a real scenario
        Err(ApiError::NotFound("Organization not found".to_string()))
    }
}
