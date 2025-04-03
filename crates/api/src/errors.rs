use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use backend::Result as BackendResult;
use serde_json::json;
use shared::errors::{
    AppError, AuthError, ConflictError, ExternalServiceError, NotFoundError, ValidationError,
};
use tracing;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    AppError(#[from] AppError),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message, error_code) = match &self {
            ApiError::AppError(err) => match err {
                AppError::Database(_) => {
                    tracing::error!(error = ?self, "Database error occurred");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Database error occurred".to_string(),
                        "DATABASE_ERROR",
                    )
                }
                AppError::Validation(val_err) => {
                    let code = match val_err {
                        ValidationError::InvalidInput(_) => "INVALID_INPUT",
                        ValidationError::MissingField(_) => "MISSING_FIELD",
                        ValidationError::ValueOutOfRange(_) => "VALUE_OUT_OF_RANGE",
                        ValidationError::InvalidFormat(_) => "INVALID_FORMAT",
                        ValidationError::InvalidRelationship(_) => "INVALID_RELATIONSHIP",
                    };
                    (StatusCode::BAD_REQUEST, val_err.to_string(), code)
                }
                AppError::Auth(auth_err) => {
                    let (status_code, code) = match auth_err {
                        AuthError::InvalidCredentials => {
                            (StatusCode::UNAUTHORIZED, "INVALID_CREDENTIALS")
                        }
                        AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "TOKEN_EXPIRED"),
                        AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "INVALID_TOKEN"),
                        AuthError::UserNotFound => (StatusCode::UNAUTHORIZED, "USER_NOT_FOUND"),
                        AuthError::PasswordHash(_) => {
                            (StatusCode::INTERNAL_SERVER_ERROR, "PASSWORD_HASH_ERROR")
                        }
                        AuthError::TokenCreation(_) => {
                            (StatusCode::INTERNAL_SERVER_ERROR, "TOKEN_CREATION_ERROR")
                        }
                    };
                    (status_code, auth_err.to_string(), code)
                }
                AppError::NotFound(not_found_err) => {
                    let code = match not_found_err {
                        NotFoundError::Asset(_) => "ASSET_NOT_FOUND",
                        NotFoundError::User(_) => "USER_NOT_FOUND",
                        NotFoundError::Organization(_) => "ORGANIZATION_NOT_FOUND",
                        NotFoundError::Vulnerability(_) => "VULNERABILITY_NOT_FOUND",
                        NotFoundError::DiscoveryJob(_) => "DISCOVERY_JOB_NOT_FOUND",
                        NotFoundError::Technology(_) => "TECHNOLOGY_NOT_FOUND",
                        NotFoundError::Port(_) => "PORT_NOT_FOUND",
                        NotFoundError::Report(_) => "REPORT_NOT_FOUND",
                    };
                    (StatusCode::NOT_FOUND, not_found_err.to_string(), code)
                }
                AppError::Conflict(conflict_err) => {
                    let code = match conflict_err {
                        ConflictError::UserExists(_) => "USER_EXISTS",
                        ConflictError::OrganizationExists(_) => "ORGANIZATION_EXISTS",
                        ConflictError::AssetExists(_) => "ASSET_EXISTS",
                        ConflictError::VulnerabilityExists(_) => "VULNERABILITY_EXISTS",
                        ConflictError::PortExists(_) => "PORT_EXISTS",
                        ConflictError::ConflictingUpdate => "CONFLICTING_UPDATE",
                    };
                    (StatusCode::CONFLICT, conflict_err.to_string(), code)
                }
                AppError::ExternalService(ext_err) => {
                    let code = match ext_err {
                        ExternalServiceError::DnsResolution(_) => "DNS_RESOLUTION_ERROR",
                        ExternalServiceError::PortScanning(_) => "PORT_SCANNING_ERROR",
                        ExternalServiceError::WebCrawling(_) => "WEB_CRAWLING_ERROR",
                        ExternalServiceError::CertificateTransparency(_) => {
                            "CERTIFICATE_TRANSPARENCY_ERROR"
                        }
                        ExternalServiceError::NotificationService(_) => {
                            "NOTIFICATION_SERVICE_ERROR"
                        }
                        ExternalServiceError::ExternalApi(_) => "EXTERNAL_API_ERROR",
                        ExternalServiceError::ConnectionTimeout(_) => "CONNECTION_TIMEOUT",
                    };
                    tracing::error!(error = ?self, "External service error occurred");
                    (StatusCode::BAD_GATEWAY, ext_err.to_string(), code)
                }
                AppError::Config(_) => {
                    tracing::error!(error = ?self, "Configuration error occurred");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Configuration error occurred".to_string(),
                        "CONFIG_ERROR",
                    )
                }
                AppError::RateLimitExceeded(_) => (
                    StatusCode::TOO_MANY_REQUESTS,
                    "Rate limit exceeded".to_string(),
                    "RATE_LIMIT_EXCEEDED",
                ),
                AppError::PermissionDenied(_) => (
                    StatusCode::FORBIDDEN,
                    "Permission denied".to_string(),
                    "PERMISSION_DENIED",
                ),
                AppError::Internal(_) => {
                    tracing::error!(error = ?self, "Internal error occurred");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error occurred".to_string(),
                        "INTERNAL_ERROR",
                    )
                }
            },
            ApiError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                self.to_string(),
                "INVALID_CREDENTIALS",
            ),
            ApiError::InvalidToken => (StatusCode::UNAUTHORIZED, self.to_string(), "INVALID_TOKEN"),
            ApiError::TokenExpired => (StatusCode::UNAUTHORIZED, self.to_string(), "TOKEN_EXPIRED"),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string(), "UNAUTHORIZED"),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, self.to_string(), "FORBIDDEN"),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, self.to_string(), "NOT_FOUND"),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, self.to_string(), "BAD_REQUEST"),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, self.to_string(), "CONFLICT"),
            ApiError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                self.to_string(),
                "RATE_LIMITED",
            ),
            ApiError::InternalServerError(_) => {
                tracing::error!(error = ?self, "Internal server error occurred");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occurred".to_string(),
                    "INTERNAL_SERVER_ERROR",
                )
            }
        };

        let response_message = if status.is_server_error() && !cfg!(debug_assertions) {
            "An internal server error occurred.".to_string()
        } else {
            message
        };

        let body = Json(json!({
            "error": {
                "message": response_message,
                "code": error_code,
                "status": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

impl From<uuid::Error> for ApiError {
    fn from(_: uuid::Error) -> Self {
        ApiError::BadRequest("Invalid UUID format".to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest(format!("Invalid JSON: {}", err))
    }
}

/// Convert a backend::Error to ApiError
pub fn convert_backend_error(err: backend::Error) -> ApiError {
    match err {
        backend::Error::Validation(msg) => ApiError::BadRequest(msg),
        backend::Error::Authentication(_) => ApiError::InvalidCredentials,
        backend::Error::Authorization(_) => ApiError::Forbidden,
        backend::Error::NotFound(msg) => ApiError::NotFound(msg),
        backend::Error::Conflict(msg) => ApiError::Conflict(msg),
        backend::Error::Database(msg) => {
            ApiError::InternalServerError(format!("Database error: {}", msg))
        }
        backend::Error::DatabaseConnection(msg) => {
            ApiError::InternalServerError(format!("Database connection error: {}", msg))
        }
        backend::Error::DatabaseQuery(msg) => {
            ApiError::InternalServerError(format!("Database query error: {}", msg))
        }
        backend::Error::Internal(msg) => ApiError::InternalServerError(msg),
        backend::Error::Network(msg) => {
            ApiError::InternalServerError(format!("Network error: {}", msg))
        }
        backend::Error::Dependency(msg) => {
            ApiError::InternalServerError(format!("Dependency error: {}", msg))
        }
        backend::Error::RateLimit(_) => ApiError::RateLimited,
        backend::Error::BadRequest(msg) => ApiError::BadRequest(msg),
        backend::Error::Shared(err) => ApiError::AppError(err),
    }
}

pub type Result<T> = std::result::Result<T, ApiError>;

/// Helper function to convert a BackendResult to a Result
pub fn convert_result<T>(result: BackendResult<T>) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(err) => Err(convert_backend_error(err)),
    }
}
