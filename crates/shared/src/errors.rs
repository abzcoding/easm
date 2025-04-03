use thiserror::Error;
use uuid::Uuid;

/// Common error types used across the application
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),

    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Not found: {0}")]
    NotFound(#[from] NotFoundError),

    #[error("Resource conflict: {0}")]
    Conflict(#[from] ConflictError),

    #[error("External service error: {0}")]
    ExternalService(#[from] ExternalServiceError),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

/// Authentication-related errors
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("User not found")]
    UserNotFound,

    #[error("Password hash error: {0}")]
    PasswordHash(String),

    #[error("Token creation error: {0}")]
    TokenCreation(String),
}

/// Validation-related errors
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Value out of range: {0}")]
    ValueOutOfRange(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Invalid relationship: {0}")]
    InvalidRelationship(String),
}

/// Not found errors
#[derive(Error, Debug)]
pub enum NotFoundError {
    #[error("Asset not found: {0}")]
    Asset(Uuid),

    #[error("User not found: {0}")]
    User(Uuid),

    #[error("Organization not found: {0}")]
    Organization(Uuid),

    #[error("Vulnerability not found: {0}")]
    Vulnerability(Uuid),

    #[error("Discovery job not found: {0}")]
    DiscoveryJob(Uuid),

    #[error("Technology not found: {0}")]
    Technology(Uuid),

    #[error("Port not found: {0}")]
    Port(Uuid),

    #[error("Report not found: {0}")]
    Report(Uuid),
}

/// Resource conflict errors
#[derive(Error, Debug)]
pub enum ConflictError {
    #[error("User already exists: {0}")]
    UserExists(String),

    #[error("Organization already exists: {0}")]
    OrganizationExists(String),

    #[error("Asset already exists: {0}")]
    AssetExists(String),

    #[error("Vulnerability already exists: {0}")]
    VulnerabilityExists(String),

    #[error("Port already exists on asset: {0}")]
    PortExists(String),

    #[error("Conflicting resource update")]
    ConflictingUpdate,
}

/// External service errors
#[derive(Error, Debug)]
pub enum ExternalServiceError {
    #[error("DNS resolution error: {0}")]
    DnsResolution(String),

    #[error("Port scanning error: {0}")]
    PortScanning(String),

    #[error("Web crawling error: {0}")]
    WebCrawling(String),

    #[error("Certificate transparency error: {0}")]
    CertificateTransparency(String),

    #[error("Notification service error: {0}")]
    NotificationService(String),

    #[error("External API error: {0}")]
    ExternalApi(String),

    #[error("Connection timeout: {0}")]
    ConnectionTimeout(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl AppError {
    pub fn database<S: AsRef<str>>(_message: S) -> Self {
        AppError::Database(sqlx::Error::RowNotFound)
    }

    pub fn validation<S: AsRef<str>>(message: S) -> Self {
        AppError::Validation(ValidationError::InvalidInput(message.as_ref().to_string()))
    }

    pub fn authentication<S: AsRef<str>>(_message: S) -> Self {
        AppError::Auth(AuthError::InvalidCredentials)
    }

    pub fn authorization<S: AsRef<str>>(message: S) -> Self {
        AppError::PermissionDenied(message.as_ref().to_string())
    }

    pub fn not_found<S: AsRef<str>>(_message: S) -> Self {
        AppError::NotFound(NotFoundError::Asset(Uuid::new_v4()))
    }

    pub fn external_service<S: AsRef<str>>(message: S) -> Self {
        AppError::ExternalService(ExternalServiceError::DnsResolution(
            message.as_ref().to_string(),
        ))
    }

    pub fn configuration<S: AsRef<str>>(message: S) -> Self {
        AppError::Config(crate::config::ConfigError::new(message.as_ref()))
    }

    pub fn internal<S: AsRef<str>>(message: S) -> Self {
        AppError::Internal(message.as_ref().to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::internal(format!("IO error: {}", err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::internal(format!("JSON error: {}", err))
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::authentication(format!("JWT error: {}", err))
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::external_service(format!("Redis error: {:?}", err))
    }
}
