use thiserror::Error;

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("encryption error: {0}")]
    Encryption(String),

    #[error("decryption error: {0}")]
    Decryption(String),

    #[error("keyring error: {0}")]
    Keyring(String),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("credential not found: {0}")]
    NotFound(i64),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
