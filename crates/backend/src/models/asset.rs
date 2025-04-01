use serde::{Deserialize, Serialize};
use shared::types::{AssetStatus, AssetType, Timestamp, ID};

/// Asset model representing internet-facing assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// Unique identifier
    pub id: ID,

    /// Organization this asset belongs to
    pub organization_id: ID,

    /// Type of asset
    pub asset_type: AssetType,

    /// The actual asset identifier (domain, IP, etc.)
    pub value: String,

    /// Current status of the asset
    pub status: AssetStatus,

    /// When the asset was first seen
    pub first_seen: Timestamp,

    /// When the asset was last seen
    pub last_seen: Timestamp,

    /// Creation timestamp
    pub created_at: Timestamp,

    /// Last updated timestamp
    pub updated_at: Timestamp,

    /// Additional attributes specific to asset type
    pub attributes: serde_json::Value,
}

impl Asset {
    /// Create a new asset with defaults
    pub fn new(
        organization_id: ID,
        asset_type: AssetType,
        value: String,
        attributes: Option<serde_json::Value>,
    ) -> Self {
        use chrono::Utc;
        use uuid::Uuid;

        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            organization_id,
            asset_type,
            value,
            status: AssetStatus::Active,
            first_seen: now,
            last_seen: now,
            created_at: now,
            updated_at: now,
            attributes: attributes
                .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
        }
    }
}
