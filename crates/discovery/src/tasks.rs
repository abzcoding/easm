use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiscoveryTaskType {
    DnsEnumeration,
    PortScan,
    WebAppScan,
    CertificateTransparency,
    // Add other task types as needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryTask {
    pub job_id: Uuid,
    pub organization_id: Uuid,
    pub task_type: DiscoveryTaskType,
    pub target: String, // e.g., domain name, IP range
                        // Add other task-specific configurations here (e.g., ports to scan)
}
