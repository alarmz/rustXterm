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
    ConnectionStateChanged {
        session_id: String,
        state: String,
    },

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}
