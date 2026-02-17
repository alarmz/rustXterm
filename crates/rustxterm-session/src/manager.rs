use std::path::Path;

use rustxterm_core::session::SessionInfo;

use crate::error::SessionError;
use crate::persistence::SessionDb;

pub struct SessionManager {
    db: SessionDb,
}

impl SessionManager {
    pub fn open(data_dir: &Path) -> Result<Self, SessionError> {
        let db_path = data_dir.join("sessions.db");
        let db = SessionDb::open(&db_path)?;
        Ok(Self { db })
    }

    pub fn save_session(&self, info: &SessionInfo) -> Result<i64, SessionError> {
        if info.id > 0 {
            self.db.update(info)?;
            Ok(info.id)
        } else {
            self.db.insert(info)
        }
    }

    pub fn load_session(&self, id: i64) -> Result<SessionInfo, SessionError> {
        self.db.get(id)
    }

    pub fn list_all(&self) -> Result<Vec<SessionInfo>, SessionError> {
        self.db.list_all()
    }

    pub fn delete_session(&self, id: i64) -> Result<bool, SessionError> {
        self.db.delete(id)
    }

    pub fn update_last_connected(&self, id: i64) -> Result<(), SessionError> {
        self.db.update_last_connected(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rustxterm_core::protocol::ProtocolType;
    use rustxterm_core::session::{SessionConfig, SshConfig};
    use tempfile::TempDir;

    fn test_manager() -> (SessionManager, TempDir) {
        let dir = TempDir::new().unwrap();
        let manager = SessionManager::open(dir.path()).unwrap();
        (manager, dir)
    }

    fn sample_session(name: &str) -> SessionInfo {
        SessionInfo {
            id: 0,
            name: name.to_string(),
            group_id: None,
            protocol: ProtocolType::Ssh,
            host: Some("example.com".to_string()),
            port: Some(22),
            username: Some("user".to_string()),
            credential_id: None,
            config: SessionConfig::Ssh(SshConfig::default()),
            color_tag: None,
            notes: None,
            is_favorite: false,
            auto_connect: false,
            sort_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_connected: None,
        }
    }

    #[test]
    fn test_save_and_load_session() {
        let (mgr, _dir) = test_manager();
        let info = sample_session("My Server");
        let id = mgr.save_session(&info).unwrap();
        assert!(id > 0);

        let loaded = mgr.load_session(id).unwrap();
        assert_eq!(loaded.name, "My Server");
        assert_eq!(loaded.protocol, ProtocolType::Ssh);
    }

    #[test]
    fn test_list_all_sessions() {
        let (mgr, _dir) = test_manager();
        mgr.save_session(&sample_session("Server A")).unwrap();
        mgr.save_session(&sample_session("Server B")).unwrap();
        let list = mgr.list_all().unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_delete_session() {
        let (mgr, _dir) = test_manager();
        let id = mgr.save_session(&sample_session("Temporary")).unwrap();
        assert!(mgr.delete_session(id).unwrap());
        assert!(matches!(
            mgr.load_session(id),
            Err(SessionError::NotFound(_))
        ));
    }

    #[test]
    fn test_update_last_connected() {
        let (mgr, _dir) = test_manager();
        let id = mgr.save_session(&sample_session("Server")).unwrap();

        let before = mgr.load_session(id).unwrap();
        assert!(before.last_connected.is_none());

        mgr.update_last_connected(id).unwrap();
        let after = mgr.load_session(id).unwrap();
        assert!(after.last_connected.is_some());
    }
}
