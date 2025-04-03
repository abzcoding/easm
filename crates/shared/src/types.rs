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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, PartialOrd, Ord)]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, PartialOrd, Ord)]
#[sqlx(type_name = "VARCHAR", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum UserRole {
    Admin,
    Analyst,
}

// Implement FromStr for UserRole
impl std::str::FromStr for UserRole {
    type Err = String; // Simple error type

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ADMIN" => Ok(UserRole::Admin),
            "ANALYST" => Ok(UserRole::Analyst),
            _ => Err(format!("Invalid user role: {}", s)),
        }
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
