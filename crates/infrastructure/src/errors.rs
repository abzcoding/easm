use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Initialization error: {0}")]
    Initialization(String),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Not found: {entity} with id {id}")]
    NotFound { entity: String, id: String },

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Concurrency error: {0}")]
    Concurrency(String),

    #[error("Unknown database error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;
