use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::CredentialError;

/// Metadata for a stored credential (without the decrypted password).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialRecord {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub credential_type: String,
    pub key_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Internal row with encrypted data.
pub(crate) struct CredentialRow {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub encrypted_password: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: Vec<u8>,
    pub credential_type: String,
    pub key_path: Option<String>,
}

pub struct CredentialDb {
    conn: Connection,
}

impl CredentialDb {
    pub fn open(db_path: &Path) -> Result<Self, CredentialError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<(), CredentialError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS credentials (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                username TEXT NOT NULL DEFAULT '',
                encrypted_password BLOB NOT NULL,
                nonce BLOB NOT NULL,
                salt BLOB NOT NULL,
                key_path TEXT,
                credential_type TEXT NOT NULL DEFAULT 'password',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )?;
        Ok(())
    }

    pub fn insert(
        &self,
        name: &str,
        username: &str,
        encrypted_password: &[u8],
        nonce: &[u8],
        salt: &[u8],
        key_path: Option<&str>,
        credential_type: &str,
    ) -> Result<i64, CredentialError> {
        self.conn.execute(
            "INSERT INTO credentials (name, username, encrypted_password, nonce, salt, key_path, credential_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![name, username, encrypted_password, nonce, salt, key_path, credential_type],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get(&self, id: i64) -> Result<CredentialRow, CredentialError> {
        self.conn
            .query_row(
                "SELECT id, name, username, encrypted_password, nonce, salt, credential_type, key_path
                 FROM credentials WHERE id = ?1",
                params![id],
                |row| {
                    Ok(CredentialRow {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        username: row.get(2)?,
                        encrypted_password: row.get(3)?,
                        nonce: row.get(4)?,
                        salt: row.get(5)?,
                        credential_type: row.get(6)?,
                        key_path: row.get(7)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => CredentialError::NotFound(id),
                other => CredentialError::Database(other),
            })
    }

    pub fn list(&self) -> Result<Vec<CredentialRecord>, CredentialError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, username, credential_type, key_path, created_at, updated_at
             FROM credentials ORDER BY name",
        )?;
        let records = stmt.query_map([], |row| {
            Ok(CredentialRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                username: row.get(2)?,
                credential_type: row.get(3)?,
                key_path: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        records.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn delete(&self, id: i64) -> Result<bool, CredentialError> {
        let affected = self
            .conn
            .execute("DELETE FROM credentials WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }
}
