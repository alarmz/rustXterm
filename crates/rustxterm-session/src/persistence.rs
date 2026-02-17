use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;

use rustxterm_core::protocol::ProtocolType;
use rustxterm_core::session::{SessionConfig, SessionInfo};

use crate::error::SessionError;

pub struct SessionDb {
    conn: Connection,
}

fn parse_datetime(s: &str) -> rusqlite::Result<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc())
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            )
        })
}

fn parse_optional_datetime(s: Option<String>) -> rusqlite::Result<Option<DateTime<Utc>>> {
    s.map(|s| parse_datetime(&s)).transpose()
}

impl SessionDb {
    pub fn open(db_path: &Path) -> Result<Self, SessionError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(SessionError::Io)?;
        }
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<(), SessionError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS session_groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                parent_id INTEGER,
                sort_order INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                group_id INTEGER,
                protocol TEXT NOT NULL,
                host TEXT,
                port INTEGER,
                username TEXT,
                credential_id INTEGER,
                config_json TEXT NOT NULL DEFAULT '{}',
                color_tag TEXT,
                notes TEXT,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                auto_connect INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_connected TEXT,
                FOREIGN KEY (group_id) REFERENCES session_groups(id)
            );",
        )?;
        Ok(())
    }

    pub fn insert(&self, info: &SessionInfo) -> Result<i64, SessionError> {
        let config_json = serde_json::to_string(&info.config)?;
        let protocol = format!("{:?}", info.protocol);

        self.conn.execute(
            "INSERT INTO sessions (name, group_id, protocol, host, port, username, credential_id,
             config_json, color_tag, notes, is_favorite, auto_connect, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                info.name,
                info.group_id,
                protocol,
                info.host,
                info.port,
                info.username,
                info.credential_id,
                config_json,
                info.color_tag,
                info.notes,
                info.is_favorite,
                info.auto_connect,
                info.sort_order,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update(&self, info: &SessionInfo) -> Result<(), SessionError> {
        let config_json = serde_json::to_string(&info.config)?;
        let protocol = format!("{:?}", info.protocol);

        let affected = self.conn.execute(
            "UPDATE sessions SET name = ?1, group_id = ?2, protocol = ?3, host = ?4, port = ?5,
             username = ?6, credential_id = ?7, config_json = ?8, color_tag = ?9, notes = ?10,
             is_favorite = ?11, auto_connect = ?12, sort_order = ?13,
             updated_at = datetime('now') WHERE id = ?14",
            params![
                info.name,
                info.group_id,
                protocol,
                info.host,
                info.port,
                info.username,
                info.credential_id,
                config_json,
                info.color_tag,
                info.notes,
                info.is_favorite,
                info.auto_connect,
                info.sort_order,
                info.id,
            ],
        )?;
        if affected == 0 {
            return Err(SessionError::NotFound(info.id));
        }
        Ok(())
    }

    pub fn get(&self, id: i64) -> Result<SessionInfo, SessionError> {
        self.conn
            .query_row(
                "SELECT id, name, group_id, protocol, host, port, username, credential_id,
                 config_json, color_tag, notes, is_favorite, auto_connect, sort_order,
                 created_at, updated_at, last_connected
                 FROM sessions WHERE id = ?1",
                params![id],
                |row| Self::row_to_session_info(row),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => SessionError::NotFound(id),
                other => SessionError::Database(other),
            })
    }

    pub fn list_all(&self) -> Result<Vec<SessionInfo>, SessionError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, group_id, protocol, host, port, username, credential_id,
             config_json, color_tag, notes, is_favorite, auto_connect, sort_order,
             created_at, updated_at, last_connected
             FROM sessions ORDER BY sort_order, name",
        )?;

        let rows = stmt.query_map([], |row| Self::row_to_session_info(row))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn delete(&self, id: i64) -> Result<bool, SessionError> {
        let affected = self
            .conn
            .execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn update_last_connected(&self, id: i64) -> Result<(), SessionError> {
        self.conn.execute(
            "UPDATE sessions SET last_connected = datetime('now'), updated_at = datetime('now')
             WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    fn row_to_session_info(row: &rusqlite::Row) -> rusqlite::Result<SessionInfo> {
        let protocol_str: String = row.get(3)?;
        let config_json: String = row.get(8)?;
        let created_at_str: String = row.get(14)?;
        let updated_at_str: String = row.get(15)?;
        let last_connected_str: Option<String> = row.get(16)?;

        let protocol = parse_protocol(&protocol_str)?;
        let config: SessionConfig = serde_json::from_str(&config_json).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            )
        })?;

        Ok(SessionInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            group_id: row.get(2)?,
            protocol,
            host: row.get(4)?,
            port: row.get(5)?,
            username: row.get(6)?,
            credential_id: row.get(7)?,
            config,
            color_tag: row.get(9)?,
            notes: row.get(10)?,
            is_favorite: row.get(11)?,
            auto_connect: row.get(12)?,
            sort_order: row.get(13)?,
            created_at: parse_datetime(&created_at_str)?,
            updated_at: parse_datetime(&updated_at_str)?,
            last_connected: parse_optional_datetime(last_connected_str)?,
        })
    }
}

fn parse_protocol(s: &str) -> rusqlite::Result<ProtocolType> {
    match s {
        "Ssh" => Ok(ProtocolType::Ssh),
        "Sftp" => Ok(ProtocolType::Sftp),
        "Ftp" => Ok(ProtocolType::Ftp),
        "Ftps" => Ok(ProtocolType::Ftps),
        "Telnet" => Ok(ProtocolType::Telnet),
        "Rdp" => Ok(ProtocolType::Rdp),
        "Vnc" => Ok(ProtocolType::Vnc),
        "Serial" => Ok(ProtocolType::Serial),
        "Shell" => Ok(ProtocolType::Shell),
        "Rlogin" => Ok(ProtocolType::Rlogin),
        "Mosh" => Ok(ProtocolType::Mosh),
        other => Err(rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            format!("unknown protocol: {other}").into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustxterm_core::session::{SessionConfig, ShellConfig, SshConfig};
    use tempfile::TempDir;

    fn test_db() -> (SessionDb, TempDir) {
        let dir = TempDir::new().unwrap();
        let db = SessionDb::open(&dir.path().join("test.db")).unwrap();
        (db, dir)
    }

    fn sample_session(name: &str, sort_order: i32) -> SessionInfo {
        SessionInfo {
            id: 0,
            name: name.to_string(),
            group_id: None,
            protocol: ProtocolType::Ssh,
            host: Some("example.com".to_string()),
            port: Some(22),
            username: Some("admin".to_string()),
            credential_id: None,
            config: SessionConfig::Ssh(SshConfig::default()),
            color_tag: None,
            notes: None,
            is_favorite: false,
            auto_connect: false,
            sort_order,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_connected: None,
        }
    }

    // ── parse_protocol tests ─────────────────────────────────────

    #[test]
    fn test_parse_protocol_all_variants() {
        let cases = [
            ("Ssh", ProtocolType::Ssh),
            ("Sftp", ProtocolType::Sftp),
            ("Ftp", ProtocolType::Ftp),
            ("Ftps", ProtocolType::Ftps),
            ("Telnet", ProtocolType::Telnet),
            ("Rdp", ProtocolType::Rdp),
            ("Vnc", ProtocolType::Vnc),
            ("Serial", ProtocolType::Serial),
            ("Shell", ProtocolType::Shell),
            ("Rlogin", ProtocolType::Rlogin),
            ("Mosh", ProtocolType::Mosh),
        ];
        for (input, expected) in cases {
            assert_eq!(parse_protocol(input).unwrap(), expected, "failed for {input}");
        }
    }

    #[test]
    fn test_parse_protocol_unknown() {
        assert!(parse_protocol("Unknown").is_err());
        assert!(parse_protocol("").is_err());
    }

    // ── parse_datetime tests ─────────────────────────────────────

    #[test]
    fn test_parse_datetime_valid() {
        let dt = parse_datetime("2024-01-15 10:30:00").unwrap();
        assert_eq!(dt.format("%Y-%m-%d %H:%M:%S").to_string(), "2024-01-15 10:30:00");
    }

    #[test]
    fn test_parse_datetime_invalid() {
        assert!(parse_datetime("not-a-date").is_err());
        assert!(parse_datetime("").is_err());
    }

    // ── SessionDb CRUD tests ─────────────────────────────────────

    #[test]
    fn test_insert_and_get() {
        let (db, _dir) = test_db();
        let info = sample_session("Test SSH", 0);
        let id = db.insert(&info).unwrap();
        assert!(id > 0);

        let loaded = db.get(id).unwrap();
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.name, "Test SSH");
        assert_eq!(loaded.protocol, ProtocolType::Ssh);
        assert_eq!(loaded.host, Some("example.com".to_string()));
        assert_eq!(loaded.port, Some(22));
        assert_eq!(loaded.username, Some("admin".to_string()));
        assert!(!loaded.is_favorite);
        assert!(loaded.last_connected.is_none());
    }

    #[test]
    fn test_update_session() {
        let (db, _dir) = test_db();
        let info = sample_session("Original", 0);
        let id = db.insert(&info).unwrap();

        let mut updated = db.get(id).unwrap();
        updated.name = "Renamed".to_string();
        updated.is_favorite = true;
        db.update(&updated).unwrap();

        let loaded = db.get(id).unwrap();
        assert_eq!(loaded.name, "Renamed");
        assert!(loaded.is_favorite);
    }

    #[test]
    fn test_list_all_ordering() {
        let (db, _dir) = test_db();
        // Insert with different sort_orders (out of order)
        db.insert(&sample_session("Charlie", 2)).unwrap();
        db.insert(&sample_session("Alpha", 0)).unwrap();
        db.insert(&sample_session("Bravo", 1)).unwrap();

        let list = db.list_all().unwrap();
        assert_eq!(list.len(), 3);
        // ORDER BY sort_order, name
        assert_eq!(list[0].name, "Alpha");
        assert_eq!(list[1].name, "Bravo");
        assert_eq!(list[2].name, "Charlie");
    }

    #[test]
    fn test_delete_existing() {
        let (db, _dir) = test_db();
        let id = db.insert(&sample_session("ToDelete", 0)).unwrap();
        assert!(db.delete(id).unwrap());
        assert!(matches!(db.get(id), Err(SessionError::NotFound(_))));
    }

    #[test]
    fn test_delete_nonexistent() {
        let (db, _dir) = test_db();
        assert!(!db.delete(999).unwrap());
    }

    #[test]
    fn test_get_nonexistent() {
        let (db, _dir) = test_db();
        assert!(matches!(db.get(999), Err(SessionError::NotFound(999))));
    }

    #[test]
    fn test_update_last_connected() {
        let (db, _dir) = test_db();
        let id = db.insert(&sample_session("Server", 0)).unwrap();

        // Initially last_connected is None
        let loaded = db.get(id).unwrap();
        assert!(loaded.last_connected.is_none());

        // After updating, it should be Some
        db.update_last_connected(id).unwrap();
        let loaded = db.get(id).unwrap();
        assert!(loaded.last_connected.is_some());
    }

    #[test]
    fn test_shell_config_roundtrip() {
        let (db, _dir) = test_db();
        let mut info = sample_session("Local Shell", 0);
        info.protocol = ProtocolType::Shell;
        info.config = SessionConfig::Shell(ShellConfig::default());
        info.host = None;
        info.port = None;

        let id = db.insert(&info).unwrap();
        let loaded = db.get(id).unwrap();
        assert_eq!(loaded.protocol, ProtocolType::Shell);
        assert!(loaded.host.is_none());
    }
}
