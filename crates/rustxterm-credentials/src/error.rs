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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_string_variants() {
        let cases = vec![
            (
                CredentialError::Encryption("bad key".into()),
                "encryption error: bad key",
            ),
            (
                CredentialError::Decryption("corrupted".into()),
                "decryption error: corrupted",
            ),
            (
                CredentialError::Keyring("locked".into()),
                "keyring error: locked",
            ),
            (CredentialError::NotFound(42), "credential not found: 42"),
        ];
        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err: CredentialError = io_err.into();
        assert!(matches!(err, CredentialError::Io(_)));
        assert!(err.to_string().contains("access denied"));
    }
}
