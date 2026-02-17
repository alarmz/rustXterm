use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("{0}")]
    General(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
