use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use backend::Error as BackendError;
use serde_json::json;
use shared::errors::Error as SharedError;

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
                SharedError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
                SharedError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
                SharedError::Authentication(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
                SharedError::Authorization(_) => (StatusCode::FORBIDDEN, self.to_string()),
                SharedError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
                SharedError::ExternalService(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
                SharedError::Configuration(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
                }
                SharedError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            },
            ApiError::Backend(err) => match err {
                BackendError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            },
            ApiError::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::InvalidToken => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::TokenExpired => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };

        let body = Json(json!({
            "error": {
                "message": message,
                "code": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, ApiError>;
