use thiserror::Error;

#[derive(Debug, Error)]
pub enum TerminalError {
    #[error("{0}")]
    General(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_display() {
        let err = TerminalError::General("resize failed".into());
        assert_eq!(err.to_string(), "resize failed");
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionReset, "reset");
        let err: TerminalError = io_err.into();
        assert!(matches!(err, TerminalError::Io(_)));
        assert!(err.to_string().contains("reset"));
    }

    #[test]
    fn test_from_anyhow_error() {
        let anyhow_err = anyhow::anyhow!("unexpected failure");
        let err: TerminalError = anyhow_err.into();
        assert!(matches!(err, TerminalError::Other(_)));
        assert!(err.to_string().contains("unexpected failure"));
    }
}
