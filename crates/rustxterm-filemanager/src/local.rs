//! Local filesystem operations.

use std::path::Path;
use std::time::UNIX_EPOCH;

use rustxterm_ssh::FileEntry;

use crate::error::FileManagerError;

/// Local filesystem operations using `tokio::fs`.
pub struct LocalFs;

impl LocalFs {
    /// List entries in a local directory.
    pub async fn list_dir(path: &str) -> Result<Vec<FileEntry>, FileManagerError> {
        let mut entries = Vec::new();
        let mut reader = tokio::fs::read_dir(path).await?;

        while let Some(entry) = reader.next_entry().await? {
            let meta = entry.metadata().await?;
            let name = entry.file_name().to_string_lossy().to_string();
            let full_path = entry.path().to_string_lossy().to_string();

            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs());

            #[cfg(unix)]
            let permissions = {
                use std::os::unix::fs::PermissionsExt;
                Some(meta.permissions().mode())
            };
            #[cfg(not(unix))]
            let permissions = None;

            entries.push(FileEntry {
                name,
                path: full_path,
                is_dir: meta.is_dir(),
                size: meta.len(),
                modified,
                permissions,
            });
        }

        Ok(entries)
    }

    /// Get metadata for a single local path.
    pub async fn stat(path: &str) -> Result<FileEntry, FileManagerError> {
        let meta = tokio::fs::metadata(path).await?;
        let p = Path::new(path);
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        #[cfg(unix)]
        let permissions = {
            use std::os::unix::fs::PermissionsExt;
            Some(meta.permissions().mode())
        };
        #[cfg(not(unix))]
        let permissions = None;

        Ok(FileEntry {
            name,
            path: path.to_string(),
            is_dir: meta.is_dir(),
            size: meta.len(),
            modified,
            permissions,
        })
    }

    /// Create a local directory.
    pub async fn mkdir(path: &str) -> Result<(), FileManagerError> {
        tokio::fs::create_dir(path).await?;
        Ok(())
    }

    /// Remove a local file or directory (recursively).
    pub async fn remove(path: &str) -> Result<(), FileManagerError> {
        let meta = tokio::fs::metadata(path).await?;
        if meta.is_dir() {
            tokio::fs::remove_dir_all(path).await?;
        } else {
            tokio::fs::remove_file(path).await?;
        }
        Ok(())
    }

    /// Rename a local file or directory.
    pub async fn rename(old: &str, new: &str) -> Result<(), FileManagerError> {
        tokio::fs::rename(old, new).await?;
        Ok(())
    }

    /// Return the user's home directory, or "/" as fallback.
    pub fn home_dir() -> String {
        dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let dir_path = tmp.path().to_str().unwrap();

        // Create some files
        tokio::fs::write(tmp.path().join("file1.txt"), b"hello")
            .await
            .unwrap();
        tokio::fs::create_dir(tmp.path().join("subdir"))
            .await
            .unwrap();

        let entries = LocalFs::list_dir(dir_path).await.unwrap();
        assert_eq!(entries.len(), 2);

        let file = entries.iter().find(|e| e.name == "file1.txt").unwrap();
        assert!(!file.is_dir);
        assert_eq!(file.size, 5);
        assert!(file.modified.is_some());

        let dir = entries.iter().find(|e| e.name == "subdir").unwrap();
        assert!(dir.is_dir);
    }

    #[tokio::test]
    async fn test_stat() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("data.bin");
        tokio::fs::write(&file_path, b"binary data").await.unwrap();

        let entry = LocalFs::stat(file_path.to_str().unwrap()).await.unwrap();
        assert_eq!(entry.name, "data.bin");
        assert!(!entry.is_dir);
        assert_eq!(entry.size, 11);
    }

    #[tokio::test]
    async fn test_mkdir_and_remove() {
        let tmp = tempfile::tempdir().unwrap();
        let new_dir = tmp.path().join("newdir");
        let new_dir_str = new_dir.to_str().unwrap();

        LocalFs::mkdir(new_dir_str).await.unwrap();
        assert!(new_dir.exists());

        LocalFs::remove(new_dir_str).await.unwrap();
        assert!(!new_dir.exists());
    }

    #[tokio::test]
    async fn test_rename() {
        let tmp = tempfile::tempdir().unwrap();
        let old = tmp.path().join("old.txt");
        let new = tmp.path().join("new.txt");
        tokio::fs::write(&old, b"content").await.unwrap();

        LocalFs::rename(old.to_str().unwrap(), new.to_str().unwrap())
            .await
            .unwrap();
        assert!(!old.exists());
        assert!(new.exists());
    }

    #[test]
    fn test_home_dir_returns_string() {
        let home = LocalFs::home_dir();
        assert!(!home.is_empty());
    }

    #[tokio::test]
    async fn test_stat_not_found() {
        let result = LocalFs::stat("/nonexistent_path_12345").await;
        assert!(result.is_err());
    }
}
