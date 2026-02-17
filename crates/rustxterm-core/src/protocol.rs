use serde::{Deserialize, Serialize};

/// All supported connection protocol types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolType {
    Ssh,
    Sftp,
    Ftp,
    Ftps,
    Telnet,
    Rdp,
    Vnc,
    Serial,
    Shell,
    Rlogin,
    Mosh,
}

impl ProtocolType {
    pub fn default_port(&self) -> Option<u16> {
        match self {
            Self::Ssh | Self::Sftp => Some(22),
            Self::Ftp => Some(21),
            Self::Ftps => Some(990),
            Self::Telnet => Some(23),
            Self::Rdp => Some(3389),
            Self::Vnc => Some(5900),
            Self::Rlogin => Some(513),
            Self::Mosh => Some(60001),
            Self::Serial | Self::Shell => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Ssh => "SSH",
            Self::Sftp => "SFTP",
            Self::Ftp => "FTP",
            Self::Ftps => "FTPS",
            Self::Telnet => "Telnet",
            Self::Rdp => "RDP",
            Self::Vnc => "VNC",
            Self::Serial => "Serial",
            Self::Shell => "Local Shell",
            Self::Rlogin => "Rlogin",
            Self::Mosh => "Mosh",
        }
    }
}
