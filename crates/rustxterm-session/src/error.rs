use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("session not found: {0}")]
    NotFound(i64),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
