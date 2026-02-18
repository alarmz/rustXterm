//! Local port forwarding (-L).
//!
//! Binds a local TCP listener and forwards connections through the SSH channel
//! to a remote host:port using `channel_open_direct_tcpip`.

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use rustxterm_ssh::client::SshHandler;

use crate::error::TunnelError;

/// Configuration for a local port forward.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LocalForwardConfig {
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub local_host: String,
}

impl LocalForwardConfig {
    pub fn new(local_port: u16, remote_host: String, remote_port: u16) -> Self {
        Self {
            local_port,
            remote_host,
            remote_port,
            local_host: "127.0.0.1".to_string(),
        }
    }
}

/// Start a local port forwarding tunnel.
///
/// Returns a `CancellationToken` that can be used to stop the tunnel.
pub async fn start_local_forward(
    handle: Arc<tokio::sync::Mutex<russh::client::Handle<SshHandler>>>,
    config: LocalForwardConfig,
    cancel: CancellationToken,
) -> Result<(), TunnelError> {
    let bind_addr = format!("{}:{}", config.local_host, config.local_port);
    let listener = TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| TunnelError::BindFailed {
            addr: bind_addr.clone(),
            source: e,
        })?;

    info!(
        "Local forward listening on {bind_addr} -> {}:{}",
        config.remote_host, config.remote_port
    );

    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => {
                info!("Local forward on {bind_addr} stopped");
                break;
            }
            accept = listener.accept() => {
                match accept {
                    Ok((stream, peer)) => {
                        debug!("Local forward: accepted connection from {peer}");
                        let handle = handle.clone();
                        let remote_host = config.remote_host.clone();
                        let remote_port = config.remote_port;
                        let cancel = cancel.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_local_forward_connection(
                                handle, stream, &remote_host, remote_port, cancel,
                            ).await {
                                error!("Local forward connection error: {e}");
                            }
                        });
                    }
                    Err(e) => {
                        error!("Local forward accept error: {e}");
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_local_forward_connection(
    handle: Arc<tokio::sync::Mutex<russh::client::Handle<SshHandler>>>,
    mut tcp_stream: tokio::net::TcpStream,
    remote_host: &str,
    remote_port: u16,
    cancel: CancellationToken,
) -> Result<(), TunnelError> {
    let channel = {
        let h = handle.lock().await;
        h.channel_open_direct_tcpip(remote_host, remote_port as u32, "127.0.0.1", 0)
            .await
            .map_err(|e| TunnelError::SshChannel(e.to_string()))?
    };

    let mut channel_stream = channel.into_stream();
    let (mut tcp_read, mut tcp_write) = tcp_stream.split();

    let mut buf_ssh = vec![0u8; 32768];
    let mut buf_tcp = vec![0u8; 32768];

    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => break,
            result = channel_stream.read(&mut buf_ssh) => {
                match result {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if tcp_write.write_all(&buf_ssh[..n]).await.is_err() {
                            break;
                        }
                    }
                }
            }
            result = tcp_read.read(&mut buf_tcp) => {
                match result {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if channel_stream.write_all(&buf_tcp[..n]).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_forward_config_new() {
        let config = LocalForwardConfig::new(8080, "db.internal".into(), 5432);
        assert_eq!(config.local_port, 8080);
        assert_eq!(config.remote_host, "db.internal");
        assert_eq!(config.remote_port, 5432);
        assert_eq!(config.local_host, "127.0.0.1");
    }

    #[test]
    fn test_local_forward_config_serde_roundtrip() {
        let config = LocalForwardConfig {
            local_port: 3306,
            remote_host: "mysql.internal".into(),
            remote_port: 3306,
            local_host: "0.0.0.0".into(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: LocalForwardConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.local_port, 3306);
        assert_eq!(deserialized.remote_host, "mysql.internal");
        assert_eq!(deserialized.local_host, "0.0.0.0");
    }
}
