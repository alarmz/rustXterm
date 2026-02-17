use thiserror::Error;

/// Top-level application errors.
#[derive(Debug, Error)]
pub enum RustXtermError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Credential error: {0}")]
    Credential(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("File transfer error: {0}")]
    FileTransfer(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, RustXtermError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_string_variants() {
        let cases = vec![
            (
                RustXtermError::Connection("refused".into()),
                "Connection error: refused",
            ),
            (
                RustXtermError::AuthenticationFailed("bad password".into()),
                "Authentication failed: bad password",
            ),
            (
                RustXtermError::SessionNotFound("abc-123".into()),
                "Session not found: abc-123",
            ),
            (
                RustXtermError::Credential("missing key".into()),
                "Credential error: missing key",
            ),
            (
                RustXtermError::Protocol("unknown".into()),
                "Protocol error: unknown",
            ),
            (
                RustXtermError::Config("invalid port".into()),
                "Configuration error: invalid port",
            ),
            (
                RustXtermError::FileTransfer("timeout".into()),
                "File transfer error: timeout",
            ),
            (
                RustXtermError::Plugin("not found".into()),
                "Plugin error: not found",
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err: RustXtermError = io_err.into();
        assert!(matches!(err, RustXtermError::Io(_)));
        assert!(err.to_string().contains("file missing"));
    }

    #[test]
    fn test_from_anyhow_error() {
        let anyhow_err = anyhow::anyhow!("something went wrong");
        let err: RustXtermError = anyhow_err.into();
        assert!(matches!(err, RustXtermError::Other(_)));
        assert!(err.to_string().contains("something went wrong"));
    }
}
