//! Remote filesystem operations (SFTP).

use std::sync::Arc;

use rustxterm_ssh::{FileEntry, SshSftpSession};

use crate::error::FileManagerError;

/// Remote filesystem backed by an SFTP session.
pub struct RemoteFs {
    sftp: Arc<SshSftpSession>,
}

impl RemoteFs {
    pub fn new(sftp: Arc<SshSftpSession>) -> Self {
        Self { sftp }
    }

    pub async fn list_dir(&self, path: &str) -> Result<Vec<FileEntry>, FileManagerError> {
        Ok(self.sftp.read_dir(path).await?)
    }

    pub async fn stat(&self, path: &str) -> Result<FileEntry, FileManagerError> {
        Ok(self.sftp.stat(path).await?)
    }

    pub async fn mkdir(&self, path: &str) -> Result<(), FileManagerError> {
        Ok(self.sftp.mkdir(path).await?)
    }

    pub async fn remove_file(&self, path: &str) -> Result<(), FileManagerError> {
        Ok(self.sftp.remove_file(path).await?)
    }

    pub async fn remove_dir(&self, path: &str) -> Result<(), FileManagerError> {
        Ok(self.sftp.remove_dir(path).await?)
    }

    pub async fn rename(&self, old: &str, new: &str) -> Result<(), FileManagerError> {
        Ok(self.sftp.rename(old, new).await?)
    }

    pub async fn chmod(&self, path: &str, mode: u32) -> Result<(), FileManagerError> {
        Ok(self.sftp.chmod(path, mode).await?)
    }

    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>, FileManagerError> {
        Ok(self.sftp.read_file(path).await?)
    }

    pub async fn write_file(&self, path: &str, data: &[u8]) -> Result<(), FileManagerError> {
        Ok(self.sftp.write_file(path, data).await?)
    }

    pub async fn canonicalize(&self, path: &str) -> Result<String, FileManagerError> {
        Ok(self.sftp.canonicalize(path).await?)
    }
}
