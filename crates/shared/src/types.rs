use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum AssetStatus {
    Active,
    Inactive,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum AssetType {
    Domain,
    IPAddress,
    WebApp,
    Certificate,
    CodeRepo,
    CloudResource,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, PartialOrd, Ord, Hash,
)]
#[sqlx(type_name = "VARCHAR", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum VulnerabilityStatus {
    Open,
    Closed,
    AcceptedRisk,
    FalsePositive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum Protocol {
    TCP,
    UDP,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum PortStatus {
    Open,
    Closed,
    Filtered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum JobType {
    DnsEnum,
    PortScan,
    WebCrawl,
    CertScan,
    VulnScan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DiscoveryMethod {
    Manual,
    DNSEnumeration,
    PortScan,
    WebCrawl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "PENDING"),
            JobStatus::Running => write!(f, "RUNNING"),
            JobStatus::Completed => write!(f, "COMPLETED"),
            JobStatus::Failed => write!(f, "FAILED"),
            JobStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, PartialOrd, Ord)]
#[sqlx(type_name = "VARCHAR", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum UserRole {
    Admin,    // Full access
    Manager,  // Org-wide access, can manage users and assets
    Analyst,  // Can view and add findings
    ReadOnly, // Read-only access
}

// Implement FromStr for UserRole
impl std::str::FromStr for UserRole {
    type Err = String; // Simple error type

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ADMIN" => Ok(UserRole::Admin),
            "MANAGER" => Ok(UserRole::Manager),
            "ANALYST" => Ok(UserRole::Analyst),
            "READONLY" => Ok(UserRole::ReadOnly),
            _ => Err(format!("Invalid user role: {}", s)),
        }
    }
}

impl UserRole {
    /// Check if this role can perform admin operations
    pub fn can_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    /// Check if this role can manage users in an organization
    pub fn can_manage_users(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Manager)
    }

    /// Check if this role can modify assets
    pub fn can_modify_assets(&self) -> bool {
        matches!(
            self,
            UserRole::Admin | UserRole::Manager | UserRole::Analyst
        )
    }

    /// Check if this role can view assets
    pub fn can_view_assets(&self) -> bool {
        true // All roles can view assets
    }

    /// Check if this role can modify vulnerabilities
    pub fn can_modify_vulnerabilities(&self) -> bool {
        matches!(
            self,
            UserRole::Admin | UserRole::Manager | UserRole::Analyst
        )
    }

    /// Check if this role can run discovery jobs
    pub fn can_run_discovery(&self) -> bool {
        matches!(
            self,
            UserRole::Admin | UserRole::Manager | UserRole::Analyst
        )
    }
}

pub type ID = Uuid;

pub type Timestamp = chrono::DateTime<chrono::Utc>;

/// Parameters for pagination
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: u32,
    pub page_size: u32,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 10,
        }
    }
}

impl PaginationParams {
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.page_size
    }
}
