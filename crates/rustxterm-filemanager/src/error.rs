use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileManagerError {
    #[error("sftp error: {0}")]
    Sftp(#[from] rustxterm_ssh::SshError),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("transfer cancelled: {0}")]
    TransferCancelled(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_variants() {
        let cases = vec![
            (
                FileManagerError::NotFound("/missing".into()),
                "not found: /missing",
            ),
            (
                FileManagerError::PermissionDenied("/root".into()),
                "permission denied: /root",
            ),
            (
                FileManagerError::TransferCancelled("xfer-1".into()),
                "transfer cancelled: xfer-1",
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let err: FileManagerError = io_err.into();
        assert!(matches!(err, FileManagerError::Io(_)));
        assert!(err.to_string().contains("gone"));
    }

    #[test]
    fn test_from_ssh_error() {
        let ssh_err = rustxterm_ssh::SshError::Sftp("bad path".into());
        let err: FileManagerError = ssh_err.into();
        assert!(matches!(err, FileManagerError::Sftp(_)));
        assert!(err.to_string().contains("bad path"));
    }
}
