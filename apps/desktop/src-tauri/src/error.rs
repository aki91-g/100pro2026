// import
use serde::{Serialize, Serializer};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string().as_str())
    }
}

// alias for convenience
pub type AppResult<T> = Result<T, AppError>;