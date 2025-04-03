use shared::errors::AppError as SharedError;
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

    #[error("Database connection error: {0}")]
    DatabaseConnection(String),

    #[error("Database query error: {0}")]
    DatabaseQuery(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Dependency error: {0}")]
    Dependency(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

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
                    match code.as_ref() {
                        "23505" => Error::Conflict("Record already exists".to_string()),
                        "23503" => Error::Conflict("Foreign key constraint violation".to_string()),
                        "42P01" => Error::DatabaseQuery(format!("Relation does not exist: {}", e)),
                        "42703" => Error::DatabaseQuery(format!("Column does not exist: {}", e)),
                        "53300" => Error::DatabaseConnection("Too many connections".to_string()),
                        _ => Error::Database(format!("Database error ({:?}): {}", code, e)),
                    }
                } else {
                    Error::Database(format!("Database error: {}", e))
                }
            }
            sqlx::Error::Configuration(e) => {
                Error::DatabaseConnection(format!("Database configuration error: {}", e))
            }
            sqlx::Error::PoolTimedOut => {
                Error::DatabaseConnection("Connection pool timeout".to_string())
            }
            sqlx::Error::PoolClosed => {
                Error::DatabaseConnection("Connection pool closed".to_string())
            }
            sqlx::Error::WorkerCrashed => {
                Error::DatabaseConnection("Database worker crashed".to_string())
            }
            sqlx::Error::Io(e) => Error::Network(format!("IO error: {}", e)),
            sqlx::Error::Tls(e) => Error::Network(format!("TLS error: {}", e)),
            _ => Error::Database(format!("Database error: {}", err)),
        }
    }
}

/// Result type for the backend crate
pub type Result<T> = std::result::Result<T, Error>;

/// Extension trait to add context to errors
pub trait ErrorExt<T> {
    /// Add context to an error result
    fn with_context(self, context: impl FnOnce() -> String) -> Result<T>;
}

impl<T, E> ErrorExt<T> for std::result::Result<T, E>
where
    E: Into<Error>,
{
    fn with_context(self, context: impl FnOnce() -> String) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => {
                let error: Error = err.into();
                match error {
                    Error::NotFound(_) => Err(Error::NotFound(context())),
                    Error::Conflict(_) => Err(Error::Conflict(context())),
                    Error::Validation(_) => Err(Error::Validation(context())),
                    Error::Authentication(_) => Err(Error::Authentication(context())),
                    Error::Authorization(_) => Err(Error::Authorization(context())),
                    Error::DatabaseConnection(_) => Err(Error::DatabaseConnection(context())),
                    Error::DatabaseQuery(_) => Err(Error::DatabaseQuery(context())),
                    Error::Database(_) => Err(Error::Database(context())),
                    Error::Network(_) => Err(Error::Network(context())),
                    Error::Dependency(_) => Err(Error::Dependency(context())),
                    Error::RateLimit(_) => Err(Error::RateLimit(context())),
                    Error::BadRequest(_) => Err(Error::BadRequest(context())),
                    Error::Internal(_) => Err(Error::Internal(context())),
                    Error::Shared(shared) => Err(Error::Shared(shared)),
                }
            }
        }
    }
}
