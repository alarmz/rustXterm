use thiserror::Error;

#[derive(Debug, Error)]
pub enum SshError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("channel error: {0}")]
    ChannelError(String),

    #[error("timeout")]
    Timeout,

    #[error("ssh protocol error: {0}")]
    Protocol(#[from] russh::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
