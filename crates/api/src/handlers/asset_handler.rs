use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use backend::models::Asset;
use serde::{Deserialize, Serialize};
use shared::types::{AssetStatus, AssetType, ID};
use std::{net::IpAddr, sync::Arc};
use url::Url;
use uuid::Uuid;

use crate::{errors::ApiError, errors::Result, state::AppState};

// Validation functions
fn is_valid_domain(domain: &str) -> bool {
    // Basic domain validation - more comprehensive validation could be implemented
    if domain.is_empty() || domain.len() > 255 {
        return false;
    }

    // Check for valid domain format (alphanumeric + hyphens, with dots as separators)
    let labels: Vec<&str> = domain.split('.').collect();

    // At least one dot and a TLD
    if labels.len() < 2 {
        return false;
    }

    // Each label should follow domain naming rules
    for label in labels {
        // Labels must be 1-63 characters
        if label.is_empty() || label.len() > 63 {
            return false;
        }

        // Check characters (alphanumeric or hyphen, but not starting/ending with hyphen)
        if label.starts_with('-') || label.ends_with('-') {
            return false;
        }

        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return false;
        }
    }

    true
}

fn is_valid_ip_address(ip: &str) -> bool {
    // Parse as IP address
    ip.parse::<IpAddr>().is_ok()
}

fn is_valid_webapp(url: &str) -> bool {
    match Url::parse(url) {
        Ok(parsed_url) => {
            // Check if scheme is http or https
            let scheme = parsed_url.scheme();
            if scheme != "http" && scheme != "https" {
                return false;
            }

            // Must have a host
            if parsed_url.host().is_none() {
                return false;
            }

            true
        }
        Err(_) => false,
    }
}

fn is_valid_certificate(cert: &str) -> bool {
    // Basic validation for certificate fingerprints or domains
    // For simplicity, assuming it's either a domain or a SHA1/SHA256 fingerprint
    if is_valid_domain(cert) {
        return true;
    }

    // Check if it looks like a SHA1 (40 hex chars) or SHA256 (64 hex chars) fingerprint
    let clean_cert = cert.replace(":", "").replace(" ", "");
    (clean_cert.len() == 40 || clean_cert.len() == 64)
        && clean_cert.chars().all(|c| c.is_ascii_hexdigit())
}

fn is_valid_code_repo(repo_url: &str) -> bool {
    // Basic URL validation for code repositories
    match Url::parse(repo_url) {
        Ok(parsed_url) => {
            let scheme = parsed_url.scheme();
            // Common repository schemes
            if scheme != "http" && scheme != "https" && scheme != "git" && scheme != "ssh" {
                return false;
            }

            // Must have a host
            if parsed_url.host().is_none() {
                return false;
            }

            true
        }
        Err(_) => false,
    }
}

#[derive(Debug, Deserialize)]
pub struct AssetQuery {
    organization_id: Option<Uuid>,
    asset_type: Option<AssetType>,
    status: Option<AssetStatus>,
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct AssetListResponse {
    assets: Vec<Asset>,
    total: usize,
}

/// Request struct for creating a new asset without requiring an ID
#[derive(Debug, Deserialize)]
pub struct CreateAssetRequest {
    /// Organization this asset belongs to
    pub organization_id: ID,
    /// Type of asset
    pub asset_type: AssetType,
    /// The actual asset identifier (domain, IP, etc.)
    pub value: String,
    /// Additional attributes specific to asset type
    pub attributes: Option<serde_json::Value>,
}

/// List assets with optional filters
pub async fn list_assets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AssetQuery>,
) -> Result<Json<AssetListResponse>> {
    // Limit maximum page size
    const MAX_LIMIT: usize = 100;
    let limit = query.limit.unwrap_or(20).min(MAX_LIMIT);
    let offset = query.offset.unwrap_or(0);

    // Get assets from service
    let assets = state
        .asset_service
        .list_assets(
            query.organization_id,
            query.asset_type,
            query.status,
            limit,
            offset,
        )
        .await?;

    // Get total count for pagination
    let total = state
        .asset_service
        .count_assets(query.organization_id, query.asset_type, query.status)
        .await?;

    Ok(Json(AssetListResponse { assets, total }))
}

/// Get a single asset by ID
pub async fn get_asset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ID>,
) -> Result<Json<Asset>> {
    let asset = state.asset_service.get_asset(id).await?;
    Ok(Json(asset))
}

/// Create a new asset
pub async fn create_asset(
    State(state): State<Arc<AppState>>,
    Json(asset_request): Json<CreateAssetRequest>,
) -> Result<(StatusCode, Json<Asset>)> {
    // Validate input based on asset type
    match asset_request.asset_type {
        AssetType::Domain => {
            if !is_valid_domain(&asset_request.value) {
                return Err(ApiError::BadRequest(format!(
                    "Invalid domain format: {}",
                    asset_request.value
                )));
            }
        }
        AssetType::IPAddress => {
            if !is_valid_ip_address(&asset_request.value) {
                return Err(ApiError::BadRequest(format!(
                    "Invalid IP address format: {}",
                    asset_request.value
                )));
            }
        }
        AssetType::WebApp => {
            if !is_valid_webapp(&asset_request.value) {
                return Err(ApiError::BadRequest(format!(
                    "Invalid web application URL: {}",
                    asset_request.value
                )));
            }
        }
        AssetType::Certificate => {
            if !is_valid_certificate(&asset_request.value) {
                return Err(ApiError::BadRequest(format!(
                    "Invalid certificate identifier: {}",
                    asset_request.value
                )));
            }
        }
        AssetType::CodeRepo => {
            if !is_valid_code_repo(&asset_request.value) {
                return Err(ApiError::BadRequest(format!(
                    "Invalid code repository URL: {}",
                    asset_request.value
                )));
            }
        }
    }

    // Create asset model from request
    let asset = Asset::new(
        asset_request.organization_id,
        asset_request.asset_type,
        asset_request.value,
        asset_request.attributes,
    );

    let created_asset = state.asset_service.create_asset(&asset).await?;
    Ok((StatusCode::CREATED, Json(created_asset)))
}

/// Update an existing asset
pub async fn update_asset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ID>,
    Json(mut asset): Json<Asset>,
) -> Result<Json<Asset>> {
    // Ensure ID in path matches body
    if asset.id != id {
        asset.id = id;
    }

    let updated_asset = state.asset_service.update_asset(&asset).await?;
    Ok(Json(updated_asset))
}

/// Delete an asset by ID
pub async fn delete_asset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ID>,
) -> Result<StatusCode> {
    state.asset_service.delete_asset(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
