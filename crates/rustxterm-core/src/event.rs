use serde::{Deserialize, Serialize};

/// Application-wide events for inter-module communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AppEvent {
    /// Terminal data received from remote/local shell.
    TerminalOutput { session_id: String, data: Vec<u8> },

    /// Terminal input from user.
    TerminalInput { session_id: String, data: Vec<u8> },

    /// Connection state changed.
    ConnectionStateChanged { session_id: String, state: String },

    /// File transfer progress update.
    TransferProgress {
        transfer_id: String,
        bytes_transferred: u64,
        total_bytes: u64,
    },

    /// Session created/updated/deleted.
    SessionChanged { action: String, session_id: String },

    /// Notification to display to user.
    Notification {
        level: NotificationLevel,
        title: String,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_output_serde_roundtrip() {
        let event = AppEvent::TerminalOutput {
            session_id: "sess-1".into(),
            data: vec![27, 91, 72],
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"TerminalOutput\""));
        assert!(json.contains("\"payload\""));
        let deserialized: AppEvent = serde_json::from_str(&json).unwrap();
        if let AppEvent::TerminalOutput { session_id, data } = deserialized {
            assert_eq!(session_id, "sess-1");
            assert_eq!(data, vec![27, 91, 72]);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn test_terminal_input_serde_roundtrip() {
        let event = AppEvent::TerminalInput {
            session_id: "sess-2".into(),
            data: vec![108, 115, 10],
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: AppEvent = serde_json::from_str(&json).unwrap();
        if let AppEvent::TerminalInput { session_id, data } = deserialized {
            assert_eq!(session_id, "sess-2");
            assert_eq!(data, vec![108, 115, 10]);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn test_connection_state_changed_serde() {
        let event = AppEvent::ConnectionStateChanged {
            session_id: "sess-3".into(),
            state: "Connected".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: AppEvent = serde_json::from_str(&json).unwrap();
        if let AppEvent::ConnectionStateChanged { session_id, state } = deserialized {
            assert_eq!(session_id, "sess-3");
            assert_eq!(state, "Connected");
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn test_transfer_progress_serde() {
        let event = AppEvent::TransferProgress {
            transfer_id: "xfer-1".into(),
            bytes_transferred: 1024,
            total_bytes: 4096,
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: AppEvent = serde_json::from_str(&json).unwrap();
        if let AppEvent::TransferProgress {
            transfer_id,
            bytes_transferred,
            total_bytes,
        } = deserialized
        {
            assert_eq!(transfer_id, "xfer-1");
            assert_eq!(bytes_transferred, 1024);
            assert_eq!(total_bytes, 4096);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn test_session_changed_serde() {
        let event = AppEvent::SessionChanged {
            action: "created".into(),
            session_id: "sess-4".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: AppEvent = serde_json::from_str(&json).unwrap();
        if let AppEvent::SessionChanged { action, session_id } = deserialized {
            assert_eq!(action, "created");
            assert_eq!(session_id, "sess-4");
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn test_notification_serde() {
        let event = AppEvent::Notification {
            level: NotificationLevel::Warning,
            title: "Disk Full".into(),
            message: "Less than 1GB remaining".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: AppEvent = serde_json::from_str(&json).unwrap();
        if let AppEvent::Notification {
            level,
            title,
            message,
        } = deserialized
        {
            assert_eq!(level, NotificationLevel::Warning);
            assert_eq!(title, "Disk Full");
            assert_eq!(message, "Less than 1GB remaining");
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn test_notification_level_rename() {
        let json = serde_json::to_string(&NotificationLevel::Info).unwrap();
        assert_eq!(json, "\"info\"");
        let json = serde_json::to_string(&NotificationLevel::Warning).unwrap();
        assert_eq!(json, "\"warning\"");
        let json = serde_json::to_string(&NotificationLevel::Error).unwrap();
        assert_eq!(json, "\"error\"");
        let json = serde_json::to_string(&NotificationLevel::Success).unwrap();
        assert_eq!(json, "\"success\"");
    }
}
