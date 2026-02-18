//! Tunnel lifecycle manager.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::info;

use rustxterm_ssh::client::SshHandler;

use crate::dynamic::{self, DynamicForwardConfig};
use crate::error::TunnelError;
use crate::local_forward::{self, LocalForwardConfig};
use crate::remote_forward::{self, RemoteForwardConfig};

/// Tunnel type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TunnelType {
    Local,
    Remote,
    Dynamic,
}

/// Information about an active tunnel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelInfo {
    pub id: String,
    pub tunnel_type: TunnelType,
    pub description: String,
    pub active: bool,
}

/// Internal state for a running tunnel.
struct RunningTunnel {
    info: TunnelInfo,
    cancel: CancellationToken,
    #[allow(dead_code)]
    task: JoinHandle<()>,
}

/// Manages the lifecycle of SSH tunnels.
pub struct TunnelManager {
    tunnels: HashMap<String, RunningTunnel>,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            tunnels: HashMap::new(),
        }
    }

    /// Start a local port forward (-L).
    pub fn start_local(
        &mut self,
        handle: Arc<tokio::sync::Mutex<russh::client::Handle<SshHandler>>>,
        config: LocalForwardConfig,
    ) -> Result<String, TunnelError> {
        let id = uuid::Uuid::new_v4().to_string();
        let cancel = CancellationToken::new();
        let description = format!(
            "L {}:{} -> {}:{}",
            config.local_host, config.local_port, config.remote_host, config.remote_port
        );

        let cancel_clone = cancel.clone();
        let task = tokio::spawn(async move {
            if let Err(e) = local_forward::start_local_forward(handle, config, cancel_clone).await {
                tracing::error!("Local forward error: {e}");
            }
        });

        self.tunnels.insert(
            id.clone(),
            RunningTunnel {
                info: TunnelInfo {
                    id: id.clone(),
                    tunnel_type: TunnelType::Local,
                    description,
                    active: true,
                },
                cancel,
                task,
            },
        );

        Ok(id)
    }

    /// Start a remote port forward (-R).
    pub fn start_remote(
        &mut self,
        handle: Arc<tokio::sync::Mutex<russh::client::Handle<SshHandler>>>,
        config: RemoteForwardConfig,
    ) -> Result<String, TunnelError> {
        let id = uuid::Uuid::new_v4().to_string();
        let cancel = CancellationToken::new();
        let description = format!(
            "R {} -> {}:{}",
            config.remote_port, config.local_host, config.local_port
        );

        let cancel_clone = cancel.clone();
        let task = tokio::spawn(async move {
            if let Err(e) = remote_forward::start_remote_forward(handle, config, cancel_clone).await
            {
                tracing::error!("Remote forward error: {e}");
            }
        });

        self.tunnels.insert(
            id.clone(),
            RunningTunnel {
                info: TunnelInfo {
                    id: id.clone(),
                    tunnel_type: TunnelType::Remote,
                    description,
                    active: true,
                },
                cancel,
                task,
            },
        );

        Ok(id)
    }

    /// Start a dynamic SOCKS5 forward (-D).
    pub fn start_dynamic(
        &mut self,
        handle: Arc<tokio::sync::Mutex<russh::client::Handle<SshHandler>>>,
        config: DynamicForwardConfig,
    ) -> Result<String, TunnelError> {
        let id = uuid::Uuid::new_v4().to_string();
        let cancel = CancellationToken::new();
        let description = format!("D {}:{}", config.local_host, config.local_port);

        let cancel_clone = cancel.clone();
        let task = tokio::spawn(async move {
            if let Err(e) = dynamic::start_dynamic_forward(handle, config, cancel_clone).await {
                tracing::error!("Dynamic forward error: {e}");
            }
        });

        self.tunnels.insert(
            id.clone(),
            RunningTunnel {
                info: TunnelInfo {
                    id: id.clone(),
                    tunnel_type: TunnelType::Dynamic,
                    description,
                    active: true,
                },
                cancel,
                task,
            },
        );

        Ok(id)
    }

    /// Stop a specific tunnel.
    pub fn stop(&mut self, id: &str) -> Result<(), TunnelError> {
        let tunnel = self
            .tunnels
            .get_mut(id)
            .ok_or_else(|| TunnelError::NotFound(id.to_string()))?;

        if !tunnel.info.active {
            return Err(TunnelError::AlreadyStopped(id.to_string()));
        }

        tunnel.cancel.cancel();
        tunnel.info.active = false;
        info!("Stopped tunnel {id}");
        Ok(())
    }

    /// Stop all active tunnels.
    pub fn stop_all(&mut self) {
        for tunnel in self.tunnels.values_mut() {
            if tunnel.info.active {
                tunnel.cancel.cancel();
                tunnel.info.active = false;
            }
        }
    }

    /// List all tunnels (active and stopped).
    pub fn list(&self) -> Vec<TunnelInfo> {
        self.tunnels.values().map(|t| t.info.clone()).collect()
    }

    /// Remove stopped tunnels from the list.
    pub fn cleanup(&mut self) {
        self.tunnels.retain(|_, t| t.info.active);
    }
}

impl Default for TunnelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tunnel_type_serde() {
        let types = vec![
            (TunnelType::Local, "\"local\""),
            (TunnelType::Remote, "\"remote\""),
            (TunnelType::Dynamic, "\"dynamic\""),
        ];
        for (t, expected) in types {
            assert_eq!(serde_json::to_string(&t).unwrap(), expected);
            let deserialized: TunnelType = serde_json::from_str(expected).unwrap();
            assert_eq!(deserialized, t);
        }
    }

    #[test]
    fn test_tunnel_info_serde_roundtrip() {
        let info = TunnelInfo {
            id: "tunnel-1".into(),
            tunnel_type: TunnelType::Local,
            description: "L 127.0.0.1:8080 -> db:5432".into(),
            active: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: TunnelInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "tunnel-1");
        assert_eq!(deserialized.tunnel_type, TunnelType::Local);
        assert!(deserialized.active);
    }

    #[test]
    fn test_tunnel_manager_new_is_empty() {
        let manager = TunnelManager::new();
        assert!(manager.list().is_empty());
    }

    #[test]
    fn test_tunnel_manager_default() {
        let manager = TunnelManager::default();
        assert!(manager.list().is_empty());
    }

    #[test]
    fn test_stop_nonexistent_tunnel() {
        let mut manager = TunnelManager::new();
        let result = manager.stop("nonexistent");
        assert!(matches!(result, Err(TunnelError::NotFound(_))));
    }
}
