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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ports() {
        assert_eq!(ProtocolType::Ssh.default_port(), Some(22));
        assert_eq!(ProtocolType::Sftp.default_port(), Some(22));
        assert_eq!(ProtocolType::Ftp.default_port(), Some(21));
        assert_eq!(ProtocolType::Ftps.default_port(), Some(990));
        assert_eq!(ProtocolType::Telnet.default_port(), Some(23));
        assert_eq!(ProtocolType::Rdp.default_port(), Some(3389));
        assert_eq!(ProtocolType::Vnc.default_port(), Some(5900));
        assert_eq!(ProtocolType::Rlogin.default_port(), Some(513));
        assert_eq!(ProtocolType::Mosh.default_port(), Some(60001));
        assert_eq!(ProtocolType::Serial.default_port(), None);
        assert_eq!(ProtocolType::Shell.default_port(), None);
    }

    #[test]
    fn test_display_names() {
        assert_eq!(ProtocolType::Ssh.display_name(), "SSH");
        assert_eq!(ProtocolType::Sftp.display_name(), "SFTP");
        assert_eq!(ProtocolType::Ftp.display_name(), "FTP");
        assert_eq!(ProtocolType::Ftps.display_name(), "FTPS");
        assert_eq!(ProtocolType::Telnet.display_name(), "Telnet");
        assert_eq!(ProtocolType::Rdp.display_name(), "RDP");
        assert_eq!(ProtocolType::Vnc.display_name(), "VNC");
        assert_eq!(ProtocolType::Serial.display_name(), "Serial");
        assert_eq!(ProtocolType::Shell.display_name(), "Local Shell");
        assert_eq!(ProtocolType::Rlogin.display_name(), "Rlogin");
        assert_eq!(ProtocolType::Mosh.display_name(), "Mosh");
    }

    #[test]
    fn test_protocol_serde_roundtrip() {
        let variants = [
            ProtocolType::Ssh,
            ProtocolType::Sftp,
            ProtocolType::Ftp,
            ProtocolType::Ftps,
            ProtocolType::Telnet,
            ProtocolType::Rdp,
            ProtocolType::Vnc,
            ProtocolType::Serial,
            ProtocolType::Shell,
            ProtocolType::Rlogin,
            ProtocolType::Mosh,
        ];
        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: ProtocolType = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, deserialized);
        }
    }
}
