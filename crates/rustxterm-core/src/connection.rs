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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_ref_serde_roundtrip() {
        let cred = CredentialRef {
            id: 42,
            name: "my-server-key".into(),
        };
        let json = serde_json::to_string(&cred).unwrap();
        let deserialized: CredentialRef = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, 42);
        assert_eq!(deserialized.name, "my-server-key");
    }

    #[test]
    fn test_connection_config_serde_roundtrip() {
        let config = ConnectionConfig {
            host: "example.com".into(),
            port: 22,
            protocol: ProtocolType::Ssh,
            username: Some("admin".into()),
            credential_id: Some(5),
            ssh_gateway: None,
            extra: HashMap::from([("keepalive".into(), "60".into())]),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ConnectionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.host, "example.com");
        assert_eq!(deserialized.port, 22);
        assert_eq!(deserialized.username, Some("admin".into()));
        assert_eq!(deserialized.credential_id, Some(5));
        assert!(deserialized.ssh_gateway.is_none());
        assert_eq!(deserialized.extra.get("keepalive").unwrap(), "60");
    }

    #[test]
    fn test_connection_config_with_gateway() {
        let gateway = ConnectionConfig {
            host: "bastion.example.com".into(),
            port: 22,
            protocol: ProtocolType::Ssh,
            username: Some("jump".into()),
            credential_id: None,
            ssh_gateway: None,
            extra: HashMap::new(),
        };
        let config = ConnectionConfig {
            host: "internal.example.com".into(),
            port: 2222,
            protocol: ProtocolType::Ssh,
            username: Some("root".into()),
            credential_id: None,
            ssh_gateway: Some(Box::new(gateway)),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ConnectionConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.ssh_gateway.is_some());
        let gw = deserialized.ssh_gateway.unwrap();
        assert_eq!(gw.host, "bastion.example.com");
        assert_eq!(gw.username, Some("jump".into()));
    }

    #[test]
    fn test_connection_state_serde_all_variants() {
        let states = vec![
            (ConnectionState::Disconnected, "\"Disconnected\""),
            (ConnectionState::Connecting, "\"Connecting\""),
            (ConnectionState::Connected, "\"Connected\""),
            (ConnectionState::Reconnecting, "\"Reconnecting\""),
            (ConnectionState::Error, "\"Error\""),
        ];
        for (state, expected_json) in states {
            let json = serde_json::to_string(&state).unwrap();
            assert_eq!(json, expected_json);
            let deserialized: ConnectionState = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, state);
        }
    }

    #[test]
    fn test_connection_state_equality() {
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
    }
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
