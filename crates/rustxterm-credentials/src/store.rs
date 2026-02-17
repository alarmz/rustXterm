use std::path::Path;

use crate::crypto;
use crate::database::{CredentialDb, CredentialRecord};
use crate::error::CredentialError;
use crate::keyring_backend;

pub struct CredentialStore {
    pub(crate) db: CredentialDb,
    pub(crate) master_key: Vec<u8>,
}

impl CredentialStore {
    /// Open the credential store. Creates the database and master key if needed.
    ///
    /// If the OS keyring is unavailable (e.g., headless Linux without a secret service),
    /// falls back to a deterministic key derived from the data directory path.
    /// This fallback is less secure -- credentials are only as safe as file permissions.
    pub fn open(data_dir: &Path) -> Result<Self, CredentialError> {
        let db_path = data_dir.join("credentials.db");
        let db = CredentialDb::open(&db_path)?;

        let master_key = match keyring_backend::get_or_create_master_key() {
            Ok(key) => key,
            Err(e) => {
                tracing::warn!(
                    "Keyring unavailable ({e}), using fallback key. \
                     Stored credentials are protected by file permissions only."
                );
                let path_bytes = data_dir.as_os_str().as_encoded_bytes();
                crypto::derive_key(path_bytes, b"rustxterm-fallback").to_vec()
            }
        };

        Ok(Self { db, master_key })
    }

    /// Save a new credential (password-based).
    pub fn save(
        &self,
        name: &str,
        username: &str,
        password: &str,
    ) -> Result<i64, CredentialError> {
        let salt = crypto::generate_salt();
        let key = crypto::derive_key(&self.master_key, &salt);
        let (encrypted, nonce) = crypto::encrypt(password.as_bytes(), &key)?;

        self.db
            .insert(name, username, &encrypted, &nonce, &salt, None, "password")
    }

    /// Retrieve and decrypt the password for a credential.
    pub fn decrypt_password(&self, id: i64) -> Result<String, CredentialError> {
        let row = self.db.get(id)?;
        let salt: [u8; crypto::SALT_LEN] = row
            .salt
            .try_into()
            .map_err(|_| CredentialError::Decryption("invalid salt length".to_string()))?;
        let nonce: [u8; crypto::NONCE_LEN] = row
            .nonce
            .try_into()
            .map_err(|_| CredentialError::Decryption("invalid nonce length".to_string()))?;
        let key = crypto::derive_key(&self.master_key, &salt);
        let plaintext = crypto::decrypt(&row.encrypted_password, &nonce, &key)?;
        String::from_utf8(plaintext)
            .map_err(|e| CredentialError::Decryption(format!("invalid UTF-8: {e}")))
    }

    /// List all credentials (metadata only, no decrypted passwords).
    pub fn list(&self) -> Result<Vec<CredentialRecord>, CredentialError> {
        self.db.list()
    }

    /// Delete a credential by ID.
    pub fn delete(&self, id: i64) -> Result<bool, CredentialError> {
        self.db.delete(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::CredentialDb;
    use tempfile::TempDir;

    fn test_store() -> (CredentialStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let db = CredentialDb::open(&dir.path().join("test.db")).unwrap();
        let master_key =
            crypto::derive_key(b"test-master-key", b"test-salt-value-1234567890123456").to_vec();
        (CredentialStore { db, master_key }, dir)
    }

    #[test]
    fn test_save_and_decrypt_password() {
        let (store, _dir) = test_store();
        let id = store.save("my-server", "admin", "s3cret!").unwrap();
        let decrypted = store.decrypt_password(id).unwrap();
        assert_eq!(decrypted, "s3cret!");
    }

    #[test]
    fn test_list_after_save() {
        let (store, _dir) = test_store();
        store.save("server-a", "user1", "pass1").unwrap();
        store.save("server-b", "user2", "pass2").unwrap();
        let list = store.list().unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_delete_credential() {
        let (store, _dir) = test_store();
        let id = store.save("temp-cred", "user", "password").unwrap();
        assert!(store.delete(id).unwrap());
        assert!(matches!(
            store.decrypt_password(id),
            Err(CredentialError::NotFound(_))
        ));
    }

    #[test]
    fn test_decrypt_nonexistent() {
        let (store, _dir) = test_store();
        assert!(matches!(
            store.decrypt_password(999),
            Err(CredentialError::NotFound(999))
        ));
    }
}
