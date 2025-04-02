use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use backend::models::Asset;
use serde::{Deserialize, Serialize};
use shared::types::{AssetStatus, AssetType, ID};
use std::sync::Arc;
use uuid::Uuid;

use crate::{errors::Result, state::AppState};

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

/// List assets with optional filters
pub async fn list_assets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AssetQuery>,
) -> Result<Json<AssetListResponse>> {
    let limit = query.limit.unwrap_or(20);
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
    Json(asset): Json<Asset>,
) -> Result<(StatusCode, Json<Asset>)> {
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
