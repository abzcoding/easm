use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use backend::models::{Asset, DiscoveryJob};
use discovery::tasks::{DiscoveryTask, DiscoveryTaskType, NucleiTaskParams};
use serde::{Deserialize, Serialize};
use shared::types::{JobStatus, JobType, ID};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::{convert_result, ApiError, Result},
    state::AppState,
};

/// Query parameters for listing discovery tasks
#[derive(Debug, Deserialize)]
pub struct DiscoveryTaskQuery {
    organization_id: Option<Uuid>,
    job_type: Option<JobType>,
    status: Option<JobStatus>,
    limit: Option<usize>,
    offset: Option<usize>,
}

/// Response for listing discovery tasks
#[derive(Debug, Serialize)]
pub struct DiscoveryTaskListResponse {
    tasks: Vec<DiscoveryJob>,
    total: usize,
}

/// Request for creating a new discovery task
#[derive(Debug, Deserialize)]
pub struct CreateDiscoveryTaskRequest {
    /// Organization ID
    pub organization_id: ID,
    /// Asset ID to run the task against (optional - if not provided, target field is required)
    pub asset_id: Option<ID>,
    /// Target to scan (domain, IP, URL) - only used if asset_id is not provided
    pub target: Option<String>,
    /// Type of discovery task
    pub task_type: DiscoveryTaskType,
    /// Configuration for Nuclei scans (only used when task_type is VulnerabilityScanNuclei)
    pub nuclei_params: Option<NucleiTaskParams>,
}

/// List discovery tasks with filtering
pub async fn list_discovery_tasks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DiscoveryTaskQuery>,
) -> Result<Json<DiscoveryTaskListResponse>> {
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);

    // Get discovery jobs
    let tasks = convert_result(
        state
            .discovery_job_repository
            .list_jobs(
                query.organization_id,
                query.job_type,
                query.status,
                limit,
                offset,
            )
            .await,
    )?;

    // Get total count for pagination
    let total = convert_result(
        state
            .discovery_job_repository
            .count_jobs(query.organization_id, query.job_type, query.status)
            .await,
    )?;

    Ok(Json(DiscoveryTaskListResponse { tasks, total }))
}

/// Get a single discovery task by ID
pub async fn get_discovery_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ID>,
) -> Result<Json<DiscoveryJob>> {
    let task = convert_result(state.discovery_job_repository.get_job(id).await)?;
    Ok(Json(task))
}

/// Create a new discovery task for an asset
pub async fn create_discovery_task(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateDiscoveryTaskRequest>,
) -> Result<(StatusCode, Json<DiscoveryJob>)> {
    // Validate request
    if request.asset_id.is_none() && request.target.is_none() {
        return Err(ApiError::BadRequest(
            "Either asset_id or target must be provided".to_string(),
        ));
    }

    let target = if let Some(asset_id) = request.asset_id {
        // Get the asset to extract its value as the target
        let asset = convert_result(state.asset_service.get_asset(asset_id).await)?;
        asset.value
    } else {
        // Use the provided target
        request.target.unwrap()
    };

    // Map DiscoveryTaskType to JobType
    let job_type = match request.task_type {
        DiscoveryTaskType::DnsEnumeration => JobType::DnsEnum,
        DiscoveryTaskType::PortScan | DiscoveryTaskType::PortScanNaabu => JobType::PortScan,
        DiscoveryTaskType::WebAppScan | DiscoveryTaskType::WebAppScanHttpx => JobType::WebCrawl,
        DiscoveryTaskType::CertificateTransparency => JobType::CertScan,
        DiscoveryTaskType::VulnerabilityScanNuclei => JobType::VulnScan,
    };

    // Create configuration JSON
    let mut config = serde_json::Map::new();
    config.insert(
        "discovery_task_type".to_string(),
        serde_json::Value::String(format!("{:?}", request.task_type)),
    );

    // Add Nuclei parameters if provided
    if let Some(nuclei_params) = request.nuclei_params {
        config.insert(
            "nuclei_params".to_string(),
            serde_json::to_value(nuclei_params).unwrap_or_default(),
        );
    }

    // Create the discovery job
    let job = DiscoveryJob::new(
        request.organization_id,
        job_type,
        Some(target),
        Some(serde_json::Value::Object(config)),
    );

    // Save the job
    let created_job = convert_result(state.discovery_job_repository.create_job(&job).await)?;

    // If asset_id was provided, create a link between the asset and job
    if let Some(asset_id) = request.asset_id {
        // This would normally happen in a service layer, but we're handling it here for simplicity
        let link = backend::models::JobAssetLink {
            job_id: created_job.id,
            asset_id,
        };
        convert_result(
            state
                .discovery_job_repository
                .link_job_to_asset(&link)
                .await,
        )?;
    }

    Ok((StatusCode::CREATED, Json(created_job)))
}

/// Cancel a discovery task
pub async fn cancel_discovery_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ID>,
) -> Result<Json<DiscoveryJob>> {
    // Get the job
    let mut job = convert_result(state.discovery_job_repository.get_job(id).await)?;

    // Only cancel if the job is pending or running
    if job.status == JobStatus::Pending || job.status == JobStatus::Running {
        job.status = JobStatus::Cancelled;
        job = convert_result(state.discovery_job_repository.update_job(&job).await)?;
    } else {
        return Err(ApiError::BadRequest(format!(
            "Cannot cancel job with status {}",
            job.status
        )));
    }

    Ok(Json(job))
}

/// Delete a discovery task
pub async fn delete_discovery_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ID>,
) -> Result<StatusCode> {
    convert_result(state.discovery_job_repository.delete_job(id).await)?;
    Ok(StatusCode::NO_CONTENT)
}
