use serde::{Deserialize, Serialize};
use shared::types::ID;

/// JobAssetLink model - links jobs to assets they discovered/updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobAssetLink {
    /// Associated job ID
    pub job_id: ID,

    /// Associated asset ID
    pub asset_id: ID,
}

impl JobAssetLink {
    /// Create a new job-asset link
    pub fn new(job_id: ID, asset_id: ID) -> Self {
        Self { job_id, asset_id }
    }
}
