use serde::{Deserialize, Serialize};
use shared::types::{JobStatus, JobType, Timestamp, ID};

/// Discovery Job model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryJob {
    /// Unique identifier
    pub id: ID,

    /// Organization this job belongs to
    pub organization_id: ID,

    /// Job type
    pub job_type: JobType,

    /// Job status
    pub status: JobStatus,

    /// Optional target for the job
    pub target: Option<String>,

    /// When the job started
    pub started_at: Option<Timestamp>,

    /// When the job completed
    pub completed_at: Option<Timestamp>,

    /// Creation timestamp
    pub created_at: Timestamp,

    /// Last updated timestamp
    pub updated_at: Timestamp,

    /// Job logs
    pub logs: Option<String>,

    /// Job configuration
    pub configuration: serde_json::Value,
}

impl DiscoveryJob {
    /// Create a new discovery job
    pub fn new(
        organization_id: ID,
        job_type: JobType,
        target: Option<String>,
        configuration: Option<serde_json::Value>,
    ) -> Self {
        use chrono::Utc;
        use uuid::Uuid;

        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            organization_id,
            job_type,
            status: JobStatus::Pending,
            target,
            started_at: None,
            completed_at: None,
            created_at: now,
            updated_at: now,
            logs: None,
            configuration: configuration
                .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
        }
    }
}
