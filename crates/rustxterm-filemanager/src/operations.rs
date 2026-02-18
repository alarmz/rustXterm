//! File transfer operations (upload/download with progress).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Direction of a file transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferDirection {
    Download,
    Upload,
}

/// Status of a file transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Information about a single file transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferInfo {
    pub id: String,
    pub source_path: String,
    pub dest_path: String,
    pub direction: TransferDirection,
    pub total_bytes: u64,
    pub transferred_bytes: u64,
    pub status: TransferStatus,
}

/// Progress update sent during a transfer.
#[derive(Debug, Clone)]
pub struct TransferProgress {
    pub id: String,
    pub transferred_bytes: u64,
    pub total_bytes: u64,
}

/// Manages active file transfers.
pub struct TransferManager {
    transfers: HashMap<String, TransferInfo>,
}

impl TransferManager {
    pub fn new() -> Self {
        Self {
            transfers: HashMap::new(),
        }
    }

    /// Register a new transfer and return its ID.
    pub fn register(
        &mut self,
        source_path: String,
        dest_path: String,
        direction: TransferDirection,
        total_bytes: u64,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let info = TransferInfo {
            id: id.clone(),
            source_path,
            dest_path,
            direction,
            total_bytes,
            transferred_bytes: 0,
            status: TransferStatus::Pending,
        };
        self.transfers.insert(id.clone(), info);
        id
    }

    /// Update progress for a transfer.
    pub fn update_progress(&mut self, id: &str, transferred_bytes: u64, status: TransferStatus) {
        if let Some(info) = self.transfers.get_mut(id) {
            info.transferred_bytes = transferred_bytes;
            info.status = status;
        }
    }

    /// Cancel a transfer by marking it as cancelled.
    pub fn cancel(&mut self, id: &str) -> bool {
        if let Some(info) = self.transfers.get_mut(id) {
            info.status = TransferStatus::Cancelled;
            true
        } else {
            false
        }
    }

    /// List all transfers.
    pub fn list(&self) -> Vec<TransferInfo> {
        self.transfers.values().cloned().collect()
    }

    /// Get a specific transfer by ID.
    pub fn get(&self, id: &str) -> Option<&TransferInfo> {
        self.transfers.get(id)
    }

    /// Remove completed/cancelled/failed transfers.
    pub fn cleanup(&mut self) {
        self.transfers.retain(|_, info| {
            matches!(
                info.status,
                TransferStatus::Pending | TransferStatus::InProgress
            )
        });
    }
}

impl Default for TransferManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_info_serde_roundtrip() {
        let info = TransferInfo {
            id: "xfer-1".into(),
            source_path: "/remote/file.txt".into(),
            dest_path: "/local/file.txt".into(),
            direction: TransferDirection::Download,
            total_bytes: 4096,
            transferred_bytes: 1024,
            status: TransferStatus::InProgress,
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: TransferInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "xfer-1");
        assert_eq!(deserialized.direction, TransferDirection::Download);
        assert_eq!(deserialized.status, TransferStatus::InProgress);
        assert_eq!(deserialized.transferred_bytes, 1024);
    }

    #[test]
    fn test_transfer_direction_serde() {
        let json = serde_json::to_string(&TransferDirection::Upload).unwrap();
        assert_eq!(json, "\"upload\"");
        let json = serde_json::to_string(&TransferDirection::Download).unwrap();
        assert_eq!(json, "\"download\"");
    }

    #[test]
    fn test_transfer_status_serde() {
        let statuses = vec![
            (TransferStatus::Pending, "\"pending\""),
            (TransferStatus::InProgress, "\"inprogress\""),
            (TransferStatus::Completed, "\"completed\""),
            (TransferStatus::Failed, "\"failed\""),
            (TransferStatus::Cancelled, "\"cancelled\""),
        ];
        for (status, expected) in statuses {
            assert_eq!(serde_json::to_string(&status).unwrap(), expected);
        }
    }

    #[test]
    fn test_transfer_manager_register_and_list() {
        let mut manager = TransferManager::new();
        let id = manager.register(
            "/remote/a.txt".into(),
            "/local/a.txt".into(),
            TransferDirection::Download,
            8192,
        );
        assert!(!id.is_empty());

        let transfers = manager.list();
        assert_eq!(transfers.len(), 1);
        assert_eq!(transfers[0].source_path, "/remote/a.txt");
        assert_eq!(transfers[0].total_bytes, 8192);
        assert_eq!(transfers[0].status, TransferStatus::Pending);
    }

    #[test]
    fn test_transfer_manager_update_progress() {
        let mut manager = TransferManager::new();
        let id = manager.register("/a".into(), "/b".into(), TransferDirection::Upload, 1000);

        manager.update_progress(&id, 500, TransferStatus::InProgress);
        let info = manager.get(&id).unwrap();
        assert_eq!(info.transferred_bytes, 500);
        assert_eq!(info.status, TransferStatus::InProgress);

        manager.update_progress(&id, 1000, TransferStatus::Completed);
        let info = manager.get(&id).unwrap();
        assert_eq!(info.transferred_bytes, 1000);
        assert_eq!(info.status, TransferStatus::Completed);
    }

    #[test]
    fn test_transfer_manager_cancel() {
        let mut manager = TransferManager::new();
        let id = manager.register("/a".into(), "/b".into(), TransferDirection::Download, 100);
        assert!(manager.cancel(&id));
        assert_eq!(manager.get(&id).unwrap().status, TransferStatus::Cancelled);
        assert!(!manager.cancel("nonexistent"));
    }

    #[test]
    fn test_transfer_manager_cleanup() {
        let mut manager = TransferManager::new();
        let id1 = manager.register("/a".into(), "/b".into(), TransferDirection::Download, 100);
        let id2 = manager.register("/c".into(), "/d".into(), TransferDirection::Upload, 200);

        manager.update_progress(&id1, 100, TransferStatus::Completed);
        // id2 stays Pending

        manager.cleanup();
        assert_eq!(manager.list().len(), 1);
        assert!(manager.get(&id2).is_some());
        assert!(manager.get(&id1).is_none());
    }

    #[test]
    fn test_transfer_manager_default() {
        let manager = TransferManager::default();
        assert!(manager.list().is_empty());
    }
}
