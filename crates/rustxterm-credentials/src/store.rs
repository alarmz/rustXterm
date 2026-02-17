use std::path::Path;

use crate::crypto;
use crate::database::{CredentialDb, CredentialRecord};
use crate::error::CredentialError;
use crate::keyring_backend;

pub struct CredentialStore {
    db: CredentialDb,
    master_key: Vec<u8>,
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
