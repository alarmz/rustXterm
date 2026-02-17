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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_variants() {
        let err = SessionError::NotFound(99);
        assert_eq!(err.to_string(), "session not found: 99");
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken pipe");
        let err: SessionError = io_err.into();
        assert!(matches!(err, SessionError::Io(_)));
        assert!(err.to_string().contains("broken pipe"));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<String>("not-json").unwrap_err();
        let err: SessionError = json_err.into();
        assert!(matches!(err, SessionError::Serialization(_)));
    }
}
