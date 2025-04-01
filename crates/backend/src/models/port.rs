use serde::{Deserialize, Serialize};
use shared::types::{PortStatus, Protocol, Timestamp, ID};

/// Port model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    /// Unique identifier
    pub id: ID,

    /// Associated asset ID
    pub asset_id: ID,

    /// Port number
    pub port_number: i32,

    /// Protocol (TCP, UDP)
    pub protocol: Protocol,

    /// Service name (e.g., http, ssh, unknown)
    pub service_name: Option<String>,

    /// Service banner
    pub banner: Option<String>,

    /// Port status
    pub status: PortStatus,

    /// When the port was first seen
    pub first_seen: Timestamp,

    /// When the port was last seen
    pub last_seen: Timestamp,

    /// Creation timestamp
    pub created_at: Timestamp,

    /// Last updated timestamp
    pub updated_at: Timestamp,
}

impl Port {
    /// Create a new port
    pub fn new(
        asset_id: ID,
        port_number: i32,
        protocol: Protocol,
        service_name: Option<String>,
        banner: Option<String>,
    ) -> Self {
        use chrono::Utc;
        use uuid::Uuid;

        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            asset_id,
            port_number,
            protocol,
            service_name,
            banner,
            status: PortStatus::Open,
            first_seen: now,
            last_seen: now,
            created_at: now,
            updated_at: now,
        }
    }
}
