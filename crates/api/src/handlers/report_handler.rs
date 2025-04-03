use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{AssetType, Severity};
use std::sync::Arc;
use uuid::Uuid;

use crate::{errors::ApiError, state::AppState};

/// Query parameters for report generation
#[derive(Debug, Deserialize)]
pub struct ReportParams {
    pub organization_id: Option<Uuid>,
    pub asset_type: Option<AssetType>,
    pub severity: Option<Severity>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub format: Option<ReportFormat>,
}

/// Available report formats
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    Json,
    Csv,
    Pdf,
}

impl Default for ReportFormat {
    fn default() -> Self {
        Self::Json
    }
}

/// Response for vulnerability report
#[derive(Debug, Serialize)]
pub struct VulnerabilityReportResponse {
    pub report_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub generated_at: DateTime<Utc>,
    pub total_vulnerabilities: usize,
    pub severity_counts: SeverityCounts,
    pub status_counts: StatusCounts,
    pub top_vulnerable_assets: Vec<AssetSummary>,
}

/// Response for asset report
#[derive(Debug, Serialize)]
pub struct AssetReportResponse {
    pub report_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub generated_at: DateTime<Utc>,
    pub total_assets: usize,
    pub asset_type_counts: AssetTypeCounts,
    pub recently_discovered_assets: Vec<AssetSummary>,
}

/// Counts by severity
#[derive(Debug, Serialize)]
pub struct SeverityCounts {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub info: usize,
}

/// Counts by status
#[derive(Debug, Serialize)]
pub struct StatusCounts {
    pub open: usize,
    pub closed: usize,
    pub accepted_risk: usize,
    pub false_positive: usize,
}

/// Counts by asset type
#[derive(Debug, Serialize)]
pub struct AssetTypeCounts {
    pub domain: usize,
    pub ip_address: usize,
    pub web_app: usize,
    pub certificate: usize,
    pub code_repo: usize,
}

/// Summary of an asset for reports
#[derive(Debug, Serialize)]
pub struct AssetSummary {
    pub id: Uuid,
    pub asset_type: AssetType,
    pub value: String,
    pub vulnerability_count: usize,
    pub highest_severity: Option<Severity>,
}

/// Generate a vulnerability report
pub async fn generate_vulnerability_report(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<ReportParams>,
) -> Result<Json<VulnerabilityReportResponse>, ApiError> {
    // In a real implementation, this would query the vulnerability service
    // to get real data. For now, we'll return dummy data.

    // Create a response with sample data
    let report = VulnerabilityReportResponse {
        report_id: Uuid::new_v4(),
        organization_id: params.organization_id,
        generated_at: Utc::now(),
        total_vulnerabilities: 125,
        severity_counts: SeverityCounts {
            critical: 12,
            high: 28,
            medium: 45,
            low: 30,
            info: 10,
        },
        status_counts: StatusCounts {
            open: 85,
            closed: 30,
            accepted_risk: 5,
            false_positive: 5,
        },
        top_vulnerable_assets: vec![
            AssetSummary {
                id: Uuid::new_v4(),
                asset_type: AssetType::WebApp,
                value: "https://app.example.com".to_string(),
                vulnerability_count: 23,
                highest_severity: Some(Severity::Critical),
            },
            AssetSummary {
                id: Uuid::new_v4(),
                asset_type: AssetType::Domain,
                value: "api.example.com".to_string(),
                vulnerability_count: 15,
                highest_severity: Some(Severity::High),
            },
        ],
    };

    match params.format.unwrap_or_default() {
        ReportFormat::Json => Ok(Json(report)),
        ReportFormat::Csv => {
            // For CSV and PDF formats, we'll still use the same report object for now
            // but in a real implementation, this would generate and return a file
            Ok(Json(report))
        }
        ReportFormat::Pdf => {
            // For CSV and PDF formats, we'll still use the same report object for now
            // but in a real implementation, this would generate and return a file
            Ok(Json(report))
        }
    }
}

/// Generate an asset report
pub async fn generate_asset_report(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<ReportParams>,
) -> Result<Json<AssetReportResponse>, ApiError> {
    // In a real implementation, this would query the asset service
    // to get real data. For now, we'll return dummy data.

    // Create a response with sample data
    let report = AssetReportResponse {
        report_id: Uuid::new_v4(),
        organization_id: params.organization_id,
        generated_at: Utc::now(),
        total_assets: 78,
        asset_type_counts: AssetTypeCounts {
            domain: 25,
            ip_address: 32,
            web_app: 15,
            certificate: 5,
            code_repo: 1,
        },
        recently_discovered_assets: vec![
            AssetSummary {
                id: Uuid::new_v4(),
                asset_type: AssetType::Domain,
                value: "staging.example.com".to_string(),
                vulnerability_count: 3,
                highest_severity: Some(Severity::Medium),
            },
            AssetSummary {
                id: Uuid::new_v4(),
                asset_type: AssetType::IPAddress,
                value: "203.0.113.42".to_string(),
                vulnerability_count: 5,
                highest_severity: Some(Severity::High),
            },
        ],
    };

    match params.format.unwrap_or_default() {
        ReportFormat::Json => Ok(Json(report)),
        ReportFormat::Csv => {
            // For CSV and PDF formats, we'll still use the same report object for now
            // but in a real implementation, this would generate and return a file
            Ok(Json(report))
        }
        ReportFormat::Pdf => {
            // For CSV and PDF formats, we'll still use the same report object for now
            // but in a real implementation, this would generate and return a file
            Ok(Json(report))
        }
    }
}

/// Download a previously generated report by ID
pub async fn download_report(
    State(_state): State<Arc<AppState>>,
    Path(report_id): Path<Uuid>,
    Query(params): Query<ReportParams>,
) -> Result<impl IntoResponse, ApiError> {
    // In a real implementation, this would retrieve a previously generated report
    // from storage. For now, we'll return a simple message.

    Ok((
        StatusCode::OK,
        format!(
            "Would download report {} in {:?} format",
            report_id,
            params.format.unwrap_or_default()
        )
        .into_response(),
    ))
}
