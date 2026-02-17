use thiserror::Error;

#[derive(Debug, Error)]
pub enum SshError {
    #[error("{0}")]
    General(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
