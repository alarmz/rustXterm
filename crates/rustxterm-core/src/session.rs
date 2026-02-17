use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::protocol::ProtocolType;

/// A saved session bookmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: i64,
    pub name: String,
    pub group_id: Option<i64>,
    pub protocol: ProtocolType,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub credential_id: Option<i64>,
    pub config: SessionConfig,
    pub color_tag: Option<String>,
    pub notes: Option<String>,
    pub is_favorite: bool,
    pub auto_connect: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_connected: Option<DateTime<Utc>>,
}

/// Protocol-specific session configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SessionConfig {
    Ssh(SshConfig),
    Telnet(TelnetConfig),
    Rdp(RdpConfig),
    Vnc(VncConfig),
    Ftp(FtpConfig),
    Serial(SerialConfig),
    Shell(ShellConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub auth_method: SshAuthMethod,
    pub private_key_path: Option<String>,
    pub ssh_gateway_session_id: Option<i64>,
    pub x11_forwarding: bool,
    pub agent_forwarding: bool,
    pub compression: bool,
    pub keepalive_interval: u32,
    pub terminal_type: String,
    pub encoding: String,
    pub startup_commands: Vec<String>,
    pub environment: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SshAuthMethod {
    Password,
    PublicKey,
    KeyboardInteractive,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelnetConfig {
    pub encoding: String,
    pub terminal_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdpConfig {
    pub width: u32,
    pub height: u32,
    pub color_depth: u8,
    pub fullscreen: bool,
    pub clipboard_sharing: bool,
    pub audio_redirection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VncConfig {
    pub encoding: String,
    pub view_only: bool,
    pub quality: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpConfig {
    pub use_tls: bool,
    pub passive_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialConfig {
    pub device: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: String,
    pub flow_control: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShellConfig {
    pub shell_path: Option<String>,
    pub working_directory: Option<String>,
    pub environment: std::collections::HashMap<String, String>,
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            auth_method: SshAuthMethod::Password,
            private_key_path: None,
            ssh_gateway_session_id: None,
            x11_forwarding: false,
            agent_forwarding: false,
            compression: false,
            keepalive_interval: 60,
            terminal_type: "xterm-256color".to_string(),
            encoding: "UTF-8".to_string(),
            startup_commands: Vec::new(),
            environment: std::collections::HashMap::new(),
        }
    }
}
