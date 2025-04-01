use serde::{Deserialize, Serialize};
use shared::types::{Timestamp, ID};

/// Organization model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// Unique identifier
    pub id: ID,

    /// Organization name
    pub name: String,

    /// Creation timestamp
    pub created_at: Timestamp,

    /// Last updated timestamp
    pub updated_at: Timestamp,
}

impl Organization {
    /// Create a new organization
    pub fn new(name: String) -> Self {
        use chrono::Utc;
        use uuid::Uuid;

        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            name,
            created_at: now,
            updated_at: now,
        }
    }
}
