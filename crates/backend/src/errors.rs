use shared::Error as SharedError;
use sqlx;
use thiserror::Error;

/// Error type for the backend crate
#[derive(Error, Debug)]
pub enum Error {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Shared(#[from] SharedError),
}

/// Convert sqlx::Error to backend::Error
impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Error::NotFound("Record not found".to_string()),
            sqlx::Error::Database(e) => {
                if let Some(code) = e.code() {
                    if code == "23505" {
                        // Unique violation
                        Error::Conflict("Record already exists".to_string())
                    } else {
                        Error::Database(format!("Database error: {}", e))
                    }
                } else {
                    Error::Database(format!("Database error: {}", e))
                }
            }
            _ => Error::Database(format!("Database error: {}", err)),
        }
    }
}

/// Result type for the backend crate
pub type Result<T> = std::result::Result<T, Error>;
