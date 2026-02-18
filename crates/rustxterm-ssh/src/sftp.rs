//! SFTP subsystem — high-level wrapper around `russh_sftp`.

use russh_sftp::client::SftpSession;
use serde::{Deserialize, Serialize};

use crate::client::SshHandler;
use crate::error::SshError;

/// A single directory/file entry returned by SFTP operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<u64>,
    pub permissions: Option<u32>,
}

/// An SFTP session bound to an existing SSH connection.
pub struct SshSftpSession {
    session: SftpSession,
}

impl SshSftpSession {
    /// Open an SFTP subsystem over an existing SSH connection.
    pub async fn open(
        handle: &tokio::sync::Mutex<russh::client::Handle<SshHandler>>,
    ) -> Result<Self, SshError> {
        let channel = {
            let h = handle.lock().await;
            h.channel_open_session()
                .await
                .map_err(|e| SshError::Sftp(format!("failed to open channel: {e}")))?
        };

        channel
            .request_subsystem(true, "sftp")
            .await
            .map_err(|e| SshError::Sftp(format!("failed to request sftp subsystem: {e}")))?;

        let session = SftpSession::new(channel.into_stream())
            .await
            .map_err(|e| SshError::Sftp(format!("failed to init sftp session: {e}")))?;

        Ok(Self { session })
    }

    /// List entries in a remote directory.
    pub async fn read_dir(&self, path: &str) -> Result<Vec<FileEntry>, SshError> {
        let read_dir = self
            .session
            .read_dir(path.to_string())
            .await
            .map_err(|e| SshError::Sftp(e.to_string()))?;

        let base = if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{path}/")
        };

        let entries = read_dir
            .map(|entry| {
                let name = entry.file_name();
                let meta = entry.metadata();
                let full_path = format!("{base}{name}");
                FileEntry {
                    name,
                    path: full_path,
                    is_dir: meta.is_dir(),
                    size: meta.size.unwrap_or(0),
                    modified: meta.mtime.map(|t| t as u64),
                    permissions: meta.permissions,
                }
            })
            .collect();

        Ok(entries)
    }

    /// Get metadata for a single remote path.
    pub async fn stat(&self, path: &str) -> Result<FileEntry, SshError> {
        let meta = self
            .session
            .metadata(path.to_string())
            .await
            .map_err(|e| SshError::Sftp(e.to_string()))?;

        let name = path.rsplit('/').next().unwrap_or(path).to_string();

        Ok(FileEntry {
            name,
            path: path.to_string(),
            is_dir: meta.is_dir(),
            size: meta.size.unwrap_or(0),
            modified: meta.mtime.map(|t| t as u64),
            permissions: meta.permissions,
        })
    }

    /// Create a remote directory.
    pub async fn mkdir(&self, path: &str) -> Result<(), SshError> {
        self.session
            .create_dir(path.to_string())
            .await
            .map_err(|e| SshError::Sftp(e.to_string()))
    }

    /// Remove a remote file.
    pub async fn remove_file(&self, path: &str) -> Result<(), SshError> {
        self.session
            .remove_file(path.to_string())
            .await
            .map_err(|e| SshError::Sftp(e.to_string()))
    }

    /// Remove a remote directory.
    pub async fn remove_dir(&self, path: &str) -> Result<(), SshError> {
        self.session
            .remove_dir(path.to_string())
            .await
            .map_err(|e| SshError::Sftp(e.to_string()))
    }

    /// Rename a remote file or directory.
    pub async fn rename(&self, old: &str, new: &str) -> Result<(), SshError> {
        self.session
            .rename(old.to_string(), new.to_string())
            .await
            .map_err(|e| SshError::Sftp(e.to_string()))
    }

    /// Change permissions on a remote path.
    pub async fn chmod(&self, path: &str, mode: u32) -> Result<(), SshError> {
        use russh_sftp::protocol::FileAttributes;

        let attrs = FileAttributes {
            permissions: Some(mode),
            ..Default::default()
        };

        self.session
            .set_metadata(path.to_string(), attrs)
            .await
            .map_err(|e| SshError::Sftp(e.to_string()))
    }

    /// Read an entire remote file into memory.
    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>, SshError> {
        self.session
            .read(path.to_string())
            .await
            .map_err(|e| SshError::Sftp(e.to_string()))
    }

    /// Write data to a remote file (creates or overwrites).
    pub async fn write_file(&self, path: &str, data: &[u8]) -> Result<(), SshError> {
        self.session
            .write(path.to_string(), data)
            .await
            .map_err(|e| SshError::Sftp(e.to_string()))
    }

    /// Resolve a path to its canonical absolute form.
    pub async fn canonicalize(&self, path: &str) -> Result<String, SshError> {
        self.session
            .canonicalize(path.to_string())
            .await
            .map_err(|e| SshError::Sftp(e.to_string()))
    }

    /// Close the SFTP session.
    pub async fn close(&self) -> Result<(), SshError> {
        self.session
            .close()
            .await
            .map_err(|e| SshError::Sftp(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_entry_serde_roundtrip() {
        let entry = FileEntry {
            name: "config.toml".into(),
            path: "/home/user/config.toml".into(),
            is_dir: false,
            size: 1024,
            modified: Some(1700000000),
            permissions: Some(0o644),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: FileEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "config.toml");
        assert_eq!(deserialized.path, "/home/user/config.toml");
        assert!(!deserialized.is_dir);
        assert_eq!(deserialized.size, 1024);
        assert_eq!(deserialized.modified, Some(1700000000));
        assert_eq!(deserialized.permissions, Some(0o644));
    }

    #[test]
    fn test_file_entry_directory() {
        let entry = FileEntry {
            name: "docs".into(),
            path: "/home/user/docs".into(),
            is_dir: true,
            size: 0,
            modified: None,
            permissions: Some(0o755),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: FileEntry = serde_json::from_str(&json).unwrap();
        assert!(deserialized.is_dir);
        assert_eq!(deserialized.modified, None);
    }

    #[test]
    fn test_file_entry_minimal_fields() {
        let entry = FileEntry {
            name: "tmp".into(),
            path: "/tmp".into(),
            is_dir: true,
            size: 0,
            modified: None,
            permissions: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"permissions\":null"));
        let deserialized: FileEntry = serde_json::from_str(&json).unwrap();
        assert!(deserialized.permissions.is_none());
    }
}
