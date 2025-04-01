use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn database<S: AsRef<str>>(message: S) -> Self {
        Error::Database(message.as_ref().to_string())
    }

    pub fn validation<S: AsRef<str>>(message: S) -> Self {
        Error::Validation(message.as_ref().to_string())
    }

    pub fn authentication<S: AsRef<str>>(message: S) -> Self {
        Error::Authentication(message.as_ref().to_string())
    }

    pub fn authorization<S: AsRef<str>>(message: S) -> Self {
        Error::Authorization(message.as_ref().to_string())
    }

    pub fn not_found<S: AsRef<str>>(message: S) -> Self {
        Error::NotFound(message.as_ref().to_string())
    }

    pub fn external_service<S: AsRef<str>>(message: S) -> Self {
        Error::ExternalService(message.as_ref().to_string())
    }

    pub fn configuration<S: AsRef<str>>(message: S) -> Self {
        Error::Configuration(message.as_ref().to_string())
    }

    pub fn internal<S: AsRef<str>>(message: S) -> Self {
        Error::Internal(message.as_ref().to_string())
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Error::Database("row not found".to_string()),
            _ => Error::Database(err.to_string()),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Internal(format!("IO error: {}", err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Internal(format!("JSON error: {}", err))
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Error::Authentication(format!("JWT error: {}", err))
    }
}

impl From<redis::RedisError> for Error {
    fn from(err: redis::RedisError) -> Self {
        Error::ExternalService(format!("Redis error: {:?}", err))
    }
}
