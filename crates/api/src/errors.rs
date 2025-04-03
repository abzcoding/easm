use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use backend::Error as BackendError;
use serde_json::json;
use shared::errors::Error as SharedError;
use tracing;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    Shared(#[from] SharedError),

    #[error(transparent)]
    Backend(#[from] BackendError),

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

    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::Shared(err) => match err {
                SharedError::Database(_) => {
                    tracing::error!(error = ?self, "Database error occurred");
                    (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
                }
                SharedError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
                SharedError::Authentication(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
                SharedError::Authorization(_) => (StatusCode::FORBIDDEN, self.to_string()),
                SharedError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
                SharedError::ExternalService(_) => {
                    tracing::error!(error = ?self, "External service error occurred");
                    (StatusCode::BAD_GATEWAY, self.to_string())
                }
                SharedError::Configuration(_) => {
                    tracing::error!(error = ?self, "Configuration error occurred");
                    (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
                }
                SharedError::Internal(_) => {
                    tracing::error!(error = ?self, "Internal shared error occurred");
                    (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
                }
            },
            ApiError::Backend(err) => match err {
                BackendError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
                BackendError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
                BackendError::Authentication(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
                BackendError::Authorization(_) => (StatusCode::FORBIDDEN, self.to_string()),
                BackendError::Conflict(_) => (StatusCode::CONFLICT, self.to_string()),
                BackendError::Database(_) => {
                    tracing::error!(error = ?self, "Backend database error occurred");
                    (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
                }
                BackendError::Internal(_) => {
                    tracing::error!(error = ?self, "Backend internal error occurred");
                    (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
                }
                _ => {
                    tracing::error!(error = ?self, "Unexpected backend error occurred");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "An unexpected backend error occurred".to_string(),
                    )
                }
            },
            ApiError::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::InvalidToken => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::TokenExpired => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::InternalServerError(_) => {
                tracing::error!(error = ?self, "Explicit internal server error occurred");
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };

        let response_message = if status.is_server_error() {
            "An internal server error occurred.".to_string()
        } else {
            message
        };

        let body = Json(json!({
            "error": {
                "message": response_message,
                "code": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, ApiError>;
