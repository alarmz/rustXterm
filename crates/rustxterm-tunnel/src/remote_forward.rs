//! Remote port forwarding (-R).
//!
//! Requests the SSH server to listen on a remote port and forward connections
//! back to a local host:port.

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use rustxterm_ssh::client::SshHandler;

use crate::error::TunnelError;

/// Configuration for a remote port forward.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteForwardConfig {
    pub remote_port: u16,
    pub local_host: String,
    pub local_port: u16,
}

impl RemoteForwardConfig {
    pub fn new(remote_port: u16, local_host: String, local_port: u16) -> Self {
        Self {
            remote_port,
            local_host,
            local_port,
        }
    }
}

/// Start a remote port forwarding tunnel.
///
/// Tells the SSH server to listen on `remote_port`. When a connection arrives
/// on the remote side, the server sends a forwarded-tcpip channel which we
/// bridge to `local_host:local_port`.
pub async fn start_remote_forward(
    handle: Arc<tokio::sync::Mutex<russh::client::Handle<SshHandler>>>,
    config: RemoteForwardConfig,
    cancel: CancellationToken,
) -> Result<(), TunnelError> {
    // Ask the server to start listening on the remote port.
    {
        let mut h = handle.lock().await;
        h.tcpip_forward("0.0.0.0", config.remote_port as u32)
            .await
            .map_err(|e| TunnelError::SshChannel(format!("tcpip_forward failed: {e}")))?;
    }

    info!(
        "Remote forward: server listening on port {} -> {}:{}",
        config.remote_port, config.local_host, config.local_port
    );

    // Wait for cancellation — forwarded channels are handled by the SshHandler
    // callback (server_channel_open_forwarded_tcpip) in a future enhancement.
    cancel.cancelled().await;

    // Ask the server to stop forwarding.
    {
        let h = handle.lock().await;
        let _ = h
            .cancel_tcpip_forward("0.0.0.0", config.remote_port as u32)
            .await;
    }

    info!("Remote forward on port {} stopped", config.remote_port);
    Ok(())
}

/// Handle a single forwarded connection by bridging it to a local TCP target.
pub async fn handle_forwarded_connection(
    mut channel_stream: russh::ChannelStream<russh::client::Msg>,
    local_host: &str,
    local_port: u16,
    cancel: CancellationToken,
) -> Result<(), TunnelError> {
    let addr = format!("{local_host}:{local_port}");
    let mut tcp_stream = TcpStream::connect(&addr)
        .await
        .map_err(|e| TunnelError::SshChannel(format!("failed to connect to {addr}: {e}")))?;

    debug!("Remote forward: bridging to {addr}");

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
    fn test_remote_forward_config_new() {
        let config = RemoteForwardConfig::new(8080, "localhost".into(), 3000);
        assert_eq!(config.remote_port, 8080);
        assert_eq!(config.local_host, "localhost");
        assert_eq!(config.local_port, 3000);
    }

    #[test]
    fn test_remote_forward_config_serde_roundtrip() {
        let config = RemoteForwardConfig {
            remote_port: 9090,
            local_host: "192.168.1.100".into(),
            local_port: 80,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RemoteForwardConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.remote_port, 9090);
        assert_eq!(deserialized.local_host, "192.168.1.100");
        assert_eq!(deserialized.local_port, 80);
    }
}
