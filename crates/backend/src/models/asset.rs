use serde::{Deserialize, Serialize};
use shared::types::{AssetStatus, AssetType, Timestamp, ID};
use std::collections::HashMap;

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

    /// Get asset relationship type from attributes
    pub fn get_relationships(&self) -> HashMap<String, Vec<ID>> {
        if let Some(relationships) = self.attributes.get("relationships") {
            if let Ok(parsed) =
                serde_json::from_value::<HashMap<String, Vec<ID>>>(relationships.clone())
            {
                return parsed;
            }
        }
        HashMap::new()
    }

    /// Add a relationship to another asset
    pub fn add_relationship(&mut self, relationship_type: &str, related_asset_id: ID) -> &mut Self {
        let mut relationships = self.get_relationships();
        relationships
            .entry(relationship_type.to_string())
            .or_default()
            .push(related_asset_id);

        // Update the attributes with the new relationships
        let mut attributes = if let serde_json::Value::Object(map) = &self.attributes {
            map.clone()
        } else {
            serde_json::Map::new()
        };

        attributes.insert(
            "relationships".to_string(),
            serde_json::to_value(relationships)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        );

        self.attributes = serde_json::Value::Object(attributes);
        self
    }

    /// Remove a relationship to another asset
    pub fn remove_relationship(
        &mut self,
        relationship_type: &str,
        related_asset_id: ID,
    ) -> &mut Self {
        let mut relationships = self.get_relationships();

        if let Some(assets) = relationships.get_mut(relationship_type) {
            assets.retain(|id| *id != related_asset_id);
            if assets.is_empty() {
                relationships.remove(relationship_type);
            }
        }

        // Update the attributes with the new relationships
        let mut attributes = if let serde_json::Value::Object(map) = &self.attributes {
            map.clone()
        } else {
            serde_json::Map::new()
        };

        attributes.insert(
            "relationships".to_string(),
            serde_json::to_value(relationships)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        );

        self.attributes = serde_json::Value::Object(attributes);
        self
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

        let now = self.timestamp.unwrap_or_else(Utc::now);

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

/// Types of relationships between assets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssetRelationshipType {
    /// Asset is a subdomain of another
    Subdomain,
    /// Asset is hosted on another asset (e.g., web app on an IP)
    HostedOn,
    /// Asset redirects to another
    RedirectsTo,
    /// Asset has a DNS record pointing to another
    DnsPointsTo,
    /// Asset is a certificate for another
    CertificateFor,
    /// Asset is administratively related (e.g., part of same system)
    AdministrativelyRelated,
    /// Asset from same registrar
    SameRegistrar,
    /// Asset from same issuer
    SameIssuer,
    /// Asset depends on another
    DependsOn,
    /// Asset secures another
    Secures,
    /// Asset is in the same network
    SameNetwork,
    /// Asset has a public IP
    HasPublicIP,
    /// Asset belongs to another
    BelongsTo,
    /// Custom relationship type
    Custom(String),
}

impl AssetRelationshipType {
    pub fn as_str(&self) -> String {
        match self {
            Self::Subdomain => "subdomain".to_string(),
            Self::HostedOn => "hosted_on".to_string(),
            Self::RedirectsTo => "redirects_to".to_string(),
            Self::DnsPointsTo => "dns_points_to".to_string(),
            Self::CertificateFor => "certificate_for".to_string(),
            Self::AdministrativelyRelated => "administratively_related".to_string(),
            Self::SameRegistrar => "same_registrar".to_string(),
            Self::SameIssuer => "same_issuer".to_string(),
            Self::DependsOn => "depends_on".to_string(),
            Self::Secures => "secures".to_string(),
            Self::SameNetwork => "same_network".to_string(),
            Self::HasPublicIP => "has_public_ip".to_string(),
            Self::BelongsTo => "belongs_to".to_string(),
            Self::Custom(s) => s.clone(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "subdomain" => Self::Subdomain,
            "hosted_on" => Self::HostedOn,
            "redirects_to" => Self::RedirectsTo,
            "dns_points_to" => Self::DnsPointsTo,
            "certificate_for" => Self::CertificateFor,
            "administratively_related" => Self::AdministrativelyRelated,
            "same_registrar" => Self::SameRegistrar,
            "same_issuer" => Self::SameIssuer,
            "depends_on" => Self::DependsOn,
            "secures" => Self::Secures,
            "same_network" => Self::SameNetwork,
            "has_public_ip" => Self::HasPublicIP,
            "belongs_to" => Self::BelongsTo,
            _ => Self::Custom(s.to_string()),
        }
    }
}

/// Represents a relationship between two assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRelationship {
    /// ID of the source asset
    pub source_asset_id: ID,
    /// ID of the target asset
    pub target_asset_id: ID,
    /// Type of relationship
    pub relationship_type: String,
    /// Optional metadata about the relationship
    pub metadata: Option<serde_json::Value>,
}

impl AssetRelationship {
    pub fn new(
        source_asset_id: ID,
        target_asset_id: ID,
        relationship_type: AssetRelationshipType,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            source_asset_id,
            target_asset_id,
            relationship_type: relationship_type.as_str(),
            metadata,
        }
    }
}
