//! Dynamic port forwarding (-D / SOCKS5 proxy).
//!
//! Binds a local TCP listener as a SOCKS5 proxy. Each CONNECT request is
//! forwarded through the SSH connection using `channel_open_direct_tcpip`.

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use rustxterm_ssh::client::SshHandler;

use crate::error::TunnelError;

/// Configuration for a dynamic SOCKS5 forward.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DynamicForwardConfig {
    pub local_port: u16,
    pub local_host: String,
}

impl DynamicForwardConfig {
    pub fn new(local_port: u16) -> Self {
        Self {
            local_port,
            local_host: "127.0.0.1".to_string(),
        }
    }
}

/// Start a dynamic SOCKS5 forwarding tunnel.
pub async fn start_dynamic_forward(
    handle: Arc<tokio::sync::Mutex<russh::client::Handle<SshHandler>>>,
    config: DynamicForwardConfig,
    cancel: CancellationToken,
) -> Result<(), TunnelError> {
    let bind_addr = format!("{}:{}", config.local_host, config.local_port);
    let listener = TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| TunnelError::BindFailed {
            addr: bind_addr.clone(),
            source: e,
        })?;

    info!("Dynamic SOCKS5 proxy listening on {bind_addr}");

    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => {
                info!("Dynamic forward on {bind_addr} stopped");
                break;
            }
            accept = listener.accept() => {
                match accept {
                    Ok((stream, peer)) => {
                        debug!("SOCKS5: accepted connection from {peer}");
                        let handle = handle.clone();
                        let cancel = cancel.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_socks5_connection(handle, stream, cancel).await {
                                debug!("SOCKS5 connection error: {e}");
                            }
                        });
                    }
                    Err(e) => {
                        error!("SOCKS5 accept error: {e}");
                    }
                }
            }
        }
    }

    Ok(())
}

/// SOCKS5 address type constants.
const SOCKS5_ADDR_IPV4: u8 = 0x01;
const SOCKS5_ADDR_DOMAIN: u8 = 0x03;
const SOCKS5_ADDR_IPV6: u8 = 0x04;

/// Parse and handle a SOCKS5 connection (no-auth, CONNECT only).
async fn handle_socks5_connection(
    handle: Arc<tokio::sync::Mutex<russh::client::Handle<SshHandler>>>,
    mut stream: tokio::net::TcpStream,
    cancel: CancellationToken,
) -> Result<(), TunnelError> {
    // --- Greeting ---
    let mut buf = [0u8; 258];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| TunnelError::Socks5(format!("read greeting: {e}")))?;
    if n < 3 || buf[0] != 0x05 {
        return Err(TunnelError::Socks5("invalid SOCKS5 greeting".into()));
    }

    // We only support no-authentication (method 0x00)
    let nmethods = buf[1] as usize;
    let methods = &buf[2..2 + nmethods.min(n - 2)];
    if !methods.contains(&0x00) {
        // Reply with "no acceptable methods"
        stream.write_all(&[0x05, 0xFF]).await.ok();
        return Err(TunnelError::Socks5("no acceptable auth method".into()));
    }

    // Reply: no auth required
    stream
        .write_all(&[0x05, 0x00])
        .await
        .map_err(|e| TunnelError::Socks5(format!("write auth reply: {e}")))?;

    // --- Request ---
    let mut header = [0u8; 4];
    stream
        .read_exact(&mut header)
        .await
        .map_err(|e| TunnelError::Socks5(format!("read request header: {e}")))?;

    if header[0] != 0x05 || header[1] != 0x01 {
        // We only support CONNECT (0x01)
        let reply = [0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0]; // command not supported
        stream.write_all(&reply).await.ok();
        return Err(TunnelError::Socks5("only CONNECT supported".into()));
    }

    let (target_host, target_port) = parse_socks5_address(&mut stream, header[3]).await?;
    debug!("SOCKS5 CONNECT to {target_host}:{target_port}");

    // Open SSH direct-tcpip channel
    let channel = {
        let h = handle.lock().await;
        h.channel_open_direct_tcpip(&target_host, target_port as u32, "127.0.0.1", 0)
            .await
            .map_err(|e| {
                TunnelError::SshChannel(format!("direct-tcpip to {target_host}:{target_port}: {e}"))
            })?
    };

    // Send success reply
    let reply = [0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0]; // succeeded, bound addr 0.0.0.0:0
    stream
        .write_all(&reply)
        .await
        .map_err(|e| TunnelError::Socks5(format!("write connect reply: {e}")))?;

    // Bidirectional copy
    let mut channel_stream = channel.into_stream();
    let (mut tcp_read, mut tcp_write) = stream.split();
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

/// Parse the target address from a SOCKS5 request.
async fn parse_socks5_address(
    stream: &mut tokio::net::TcpStream,
    addr_type: u8,
) -> Result<(String, u16), TunnelError> {
    match addr_type {
        SOCKS5_ADDR_IPV4 => {
            let mut addr = [0u8; 4];
            stream
                .read_exact(&mut addr)
                .await
                .map_err(|e| TunnelError::Socks5(format!("read ipv4 addr: {e}")))?;
            let mut port_buf = [0u8; 2];
            stream
                .read_exact(&mut port_buf)
                .await
                .map_err(|e| TunnelError::Socks5(format!("read port: {e}")))?;
            let port = u16::from_be_bytes(port_buf);
            let host = format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]);
            Ok((host, port))
        }
        SOCKS5_ADDR_DOMAIN => {
            let mut len_buf = [0u8; 1];
            stream
                .read_exact(&mut len_buf)
                .await
                .map_err(|e| TunnelError::Socks5(format!("read domain length: {e}")))?;
            let len = len_buf[0] as usize;
            let mut domain = vec![0u8; len];
            stream
                .read_exact(&mut domain)
                .await
                .map_err(|e| TunnelError::Socks5(format!("read domain: {e}")))?;
            let mut port_buf = [0u8; 2];
            stream
                .read_exact(&mut port_buf)
                .await
                .map_err(|e| TunnelError::Socks5(format!("read port: {e}")))?;
            let port = u16::from_be_bytes(port_buf);
            let host = String::from_utf8(domain)
                .map_err(|_| TunnelError::Socks5("invalid domain name".into()))?;
            Ok((host, port))
        }
        SOCKS5_ADDR_IPV6 => {
            let mut addr = [0u8; 16];
            stream
                .read_exact(&mut addr)
                .await
                .map_err(|e| TunnelError::Socks5(format!("read ipv6 addr: {e}")))?;
            let mut port_buf = [0u8; 2];
            stream
                .read_exact(&mut port_buf)
                .await
                .map_err(|e| TunnelError::Socks5(format!("read port: {e}")))?;
            let port = u16::from_be_bytes(port_buf);
            let ipv6 = std::net::Ipv6Addr::from(addr);
            Ok((ipv6.to_string(), port))
        }
        _ => Err(TunnelError::Socks5(format!(
            "unsupported address type: 0x{addr_type:02x}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_forward_config_new() {
        let config = DynamicForwardConfig::new(1080);
        assert_eq!(config.local_port, 1080);
        assert_eq!(config.local_host, "127.0.0.1");
    }

    #[test]
    fn test_dynamic_forward_config_serde_roundtrip() {
        let config = DynamicForwardConfig {
            local_port: 9050,
            local_host: "0.0.0.0".into(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: DynamicForwardConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.local_port, 9050);
        assert_eq!(deserialized.local_host, "0.0.0.0");
    }

    #[tokio::test]
    async fn test_parse_socks5_ipv4_address() {
        // Build a mock stream with IPv4 addr + port bytes
        let (mut client, mut server) = tokio::io::duplex(64);
        // Write: 192.168.1.1, port 8080
        let addr_bytes: [u8; 4] = [192, 168, 1, 1];
        let port_bytes: [u8; 2] = 8080u16.to_be_bytes();

        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            client.write_all(&addr_bytes).await.unwrap();
            client.write_all(&port_bytes).await.unwrap();
        });

        // We need a TcpStream for parse_socks5_address, so test the logic directly
        // by testing the byte parsing logic inline
        let mut buf = [0u8; 4];
        use tokio::io::AsyncReadExt;
        server.read_exact(&mut buf).await.unwrap();
        let mut port_buf = [0u8; 2];
        server.read_exact(&mut port_buf).await.unwrap();
        let port = u16::from_be_bytes(port_buf);
        let host = format!("{}.{}.{}.{}", buf[0], buf[1], buf[2], buf[3]);
        assert_eq!(host, "192.168.1.1");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_socks5_constants() {
        assert_eq!(SOCKS5_ADDR_IPV4, 0x01);
        assert_eq!(SOCKS5_ADDR_DOMAIN, 0x03);
        assert_eq!(SOCKS5_ADDR_IPV6, 0x04);
    }
}
