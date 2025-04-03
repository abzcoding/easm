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
    
    /// Create a builder for more complex asset construction
    pub fn builder(organization_id: ID, asset_type: AssetType, value: String) -> AssetBuilder {
        AssetBuilder::new(organization_id, asset_type, value)
    }
}

/// Builder for creating Asset instances with more control
pub struct AssetBuilder {
    organization_id: ID,
    asset_type: AssetType,
    value: String,
    status: AssetStatus,
    attributes: serde_json::Value,
    // We'll use the same timestamp for all time fields by default
    timestamp: Option<Timestamp>,
    first_seen: Option<Timestamp>,
    last_seen: Option<Timestamp>,
}

impl AssetBuilder {
    pub fn new(organization_id: ID, asset_type: AssetType, value: String) -> Self {
        Self {
            organization_id,
            asset_type, 
            value,
            status: AssetStatus::Active,
            attributes: serde_json::Value::Object(serde_json::Map::new()),
            timestamp: None,
            first_seen: None,
            last_seen: None,
        }
    }
    
    pub fn status(mut self, status: AssetStatus) -> Self {
        self.status = status;
        self
    }
    
    pub fn attributes(mut self, attributes: serde_json::Value) -> Self {
        self.attributes = attributes;
        self
    }
    
    pub fn timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
    
    pub fn first_seen(mut self, first_seen: Timestamp) -> Self {
        self.first_seen = Some(first_seen);
        self
    }
    
    pub fn last_seen(mut self, last_seen: Timestamp) -> Self {
        self.last_seen = Some(last_seen);
        self
    }
    
    pub fn build(self) -> Asset {
        use chrono::Utc;
        use uuid::Uuid;
        
        let now = self.timestamp.unwrap_or_else(|| Utc::now());
        
        Asset {
            id: Uuid::new_v4(),
            organization_id: self.organization_id,
            asset_type: self.asset_type,
            value: self.value,
            status: self.status,
            first_seen: self.first_seen.unwrap_or(now),
            last_seen: self.last_seen.unwrap_or(now),
            created_at: now,
            updated_at: now,
            attributes: self.attributes,
        }
    }
}
