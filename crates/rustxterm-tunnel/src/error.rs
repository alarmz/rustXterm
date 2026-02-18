use thiserror::Error;

#[derive(Debug, Error)]
pub enum TunnelError {
    #[error("bind failed on {addr}: {source}")]
    BindFailed {
        addr: String,
        source: std::io::Error,
    },

    #[error("ssh channel error: {0}")]
    SshChannel(String),

    #[error("socks5 protocol error: {0}")]
    Socks5(String),

    #[error("tunnel not found: {0}")]
    NotFound(String),

    #[error("tunnel already stopped: {0}")]
    AlreadyStopped(String),

    #[error(transparent)]
    Ssh(#[from] rustxterm_ssh::SshError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_variants() {
        let err = TunnelError::SshChannel("closed".into());
        assert_eq!(err.to_string(), "ssh channel error: closed");

        let err = TunnelError::Socks5("invalid version".into());
        assert_eq!(err.to_string(), "socks5 protocol error: invalid version");

        let err = TunnelError::NotFound("tunnel-1".into());
        assert_eq!(err.to_string(), "tunnel not found: tunnel-1");

        let err = TunnelError::AlreadyStopped("tunnel-2".into());
        assert_eq!(err.to_string(), "tunnel already stopped: tunnel-2");
    }

    #[test]
    fn test_bind_failed_display() {
        let err = TunnelError::BindFailed {
            addr: "127.0.0.1:8080".into(),
            source: std::io::Error::new(std::io::ErrorKind::AddrInUse, "in use"),
        };
        assert!(err.to_string().contains("127.0.0.1:8080"));
        assert!(err.to_string().contains("in use"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionReset, "reset");
        let err: TunnelError = io_err.into();
        assert!(matches!(err, TunnelError::Io(_)));
    }

    #[test]
    fn test_from_ssh_error() {
        let ssh_err = rustxterm_ssh::SshError::ChannelError("gone".into());
        let err: TunnelError = ssh_err.into();
        assert!(matches!(err, TunnelError::Ssh(_)));
    }
}
