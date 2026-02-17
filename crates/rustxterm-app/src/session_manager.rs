use portable_pty::{MasterPty, PtySize};
use rustxterm_credentials::CredentialStore;
use rustxterm_session::SessionManager;
use rustxterm_ssh::SshClient;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

use crate::pty_manager::PtyManager;

/// A running terminal session (local shell or SSH).
pub enum ActiveSession {
    Local {
        writer: Box<dyn Write + Send>,
        master: Box<dyn MasterPty + Send>,
    },
    Ssh {
        client: SshClient,
    },
}

/// Manages all active sessions plus persistent storage.
pub struct AppSessionManager {
    pub active: HashMap<String, ActiveSession>,
    pub pty_manager: PtyManager,
    pub session_db: SessionManager,
    pub credential_store: CredentialStore,
}

impl AppSessionManager {
    pub fn new() -> Result<Self, String> {
        let data_dir = Self::data_dir();

        let session_db = SessionManager::open(&data_dir)
            .map_err(|e| format!("Failed to open session DB: {e}"))?;

        let credential_store = CredentialStore::open(&data_dir)
            .map_err(|e| format!("Failed to open credential store: {e}"))?;

        Ok(Self {
            active: HashMap::new(),
            pty_manager: PtyManager::default(),
            session_db,
            credential_store,
        })
    }

    fn data_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rustxterm")
    }

    /// Spawn a local shell session. Returns the session_id and a reader.
    pub fn spawn_local(
        &mut self,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<Box<dyn std::io::Read + Send>, String> {
        let reader = self
            .pty_manager
            .spawn(session_id, cols, rows)
            .map_err(|e| e.to_string())?;

        // Move the session from PtyManager into our active sessions map.
        if let Some(pty_session) = self.pty_manager.take_session(session_id) {
            self.active.insert(
                session_id.to_string(),
                ActiveSession::Local {
                    writer: pty_session.writer,
                    master: pty_session.master,
                },
            );
        }

        Ok(reader)
    }

    /// Insert a pre-connected SSH session (connection done outside the lock).
    pub fn insert_ssh_session(&mut self, session_id: &str, client: SshClient) {
        self.active
            .insert(session_id.to_string(), ActiveSession::Ssh { client });
    }

    /// Write data to any session (local or SSH).
    pub fn write(&mut self, session_id: &str, data: &[u8]) -> Result<(), String> {
        match self.active.get_mut(session_id) {
            Some(ActiveSession::Local { writer, .. }) => {
                writer.write_all(data).map_err(|e| e.to_string())?;
                writer.flush().map_err(|e| e.to_string())?;
                Ok(())
            }
            Some(ActiveSession::Ssh { client }) => client.write(data).map_err(|e| e.to_string()),
            None => Err(format!("session not found: {session_id}")),
        }
    }

    /// Resize any session.
    pub fn resize(&mut self, session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        match self.active.get_mut(session_id) {
            Some(ActiveSession::Local { master, .. }) => master
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|e| e.to_string()),
            Some(ActiveSession::Ssh { client }) => {
                client.resize(cols, rows).map_err(|e| e.to_string())
            }
            None => Err(format!("session not found: {session_id}")),
        }
    }

    /// Close any session.
    pub fn close(&mut self, session_id: &str) {
        self.active.remove(session_id);
    }
}
