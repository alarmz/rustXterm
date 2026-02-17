use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::Result;
use crate::protocol::ProtocolType;

/// Callback type for receiving data from a connection.
pub type DataCallback = Box<dyn Fn(&[u8]) + Send + Sync>;

/// Reference to a stored credential.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CredentialRef {
    pub id: i64,
    pub name: String,
}

/// Configuration for establishing a connection.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    pub protocol: ProtocolType,
    pub username: Option<String>,
    pub credential_id: Option<i64>,
    pub ssh_gateway: Option<Box<ConnectionConfig>>,
    pub extra: HashMap<String, String>,
}

/// Connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error,
}

/// Every connection protocol handler implements this trait.
#[async_trait]
pub trait ConnectionHandler: Send + Sync {
    /// Establish the connection.
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<()>;

    /// Gracefully disconnect.
    async fn disconnect(&mut self) -> Result<()>;

    /// Send raw bytes through the connection.
    async fn send(&mut self, data: &[u8]) -> Result<()>;

    /// Resize the remote PTY (if applicable).
    async fn resize(&mut self, cols: u16, rows: u16) -> Result<()>;

    /// Get the protocol type of this handler.
    fn protocol_type(&self) -> ProtocolType;

    /// Get the current connection state.
    fn state(&self) -> ConnectionState;
}
