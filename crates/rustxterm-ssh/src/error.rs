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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SshError::ConnectionFailed("refused".to_string());
        assert_eq!(err.to_string(), "connection failed: refused");

        let err = SshError::AuthenticationFailed("bad password".to_string());
        assert_eq!(err.to_string(), "authentication failed: bad password");

        let err = SshError::ChannelError("closed".to_string());
        assert_eq!(err.to_string(), "channel error: closed");

        let err = SshError::Timeout;
        assert_eq!(err.to_string(), "timeout");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let ssh_err: SshError = io_err.into();
        assert!(matches!(ssh_err, SshError::Io(_)));
        assert!(ssh_err.to_string().contains("refused"));
    }
}
