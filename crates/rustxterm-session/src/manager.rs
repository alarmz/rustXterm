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
