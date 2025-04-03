use serde::{Deserialize, Serialize};
use shared::types::{Timestamp, ID};

/// Technology model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Technology {
    /// Unique identifier
    pub id: ID,

    /// Associated asset ID
    pub asset_id: ID,

    /// Technology name
    pub name: String,

    /// Detected version
    pub version: Option<String>,

    /// Technology category
    pub category: Option<String>,

    /// Creation timestamp
    pub created_at: Timestamp,

    /// Last updated timestamp
    pub updated_at: Timestamp,
}

impl Technology {
    /// Create a new technology
    pub fn new(
        asset_id: ID,
        name: String,
        version: Option<String>,
        category: Option<String>,
    ) -> Self {
        use chrono::Utc;
        use uuid::Uuid;

        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            asset_id,
            name,
            version,
            category,
            created_at: now,
            updated_at: now,
        }
    }
}
