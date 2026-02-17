use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;

use crate::error::CredentialError;

const PBKDF2_ITERATIONS: u32 = 600_000;
pub const SALT_LEN: usize = 32;
pub const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

/// Derive a 256-bit encryption key from a master password and salt using PBKDF2.
pub fn derive_key(master_password: &[u8], salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(master_password, salt, PBKDF2_ITERATIONS, &mut key);
    key
}

/// Generate a random salt.
pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Encrypt plaintext using AES-256-GCM.
/// Returns (ciphertext, nonce).
pub fn encrypt(
    plaintext: &[u8],
    key: &[u8; KEY_LEN],
) -> Result<(Vec<u8>, [u8; NONCE_LEN]), CredentialError> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| CredentialError::Encryption(e.to_string()))?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CredentialError::Encryption(e.to_string()))?;

    Ok((ciphertext, nonce_bytes))
}

/// Decrypt ciphertext using AES-256-GCM.
pub fn decrypt(
    ciphertext: &[u8],
    nonce: &[u8; NONCE_LEN],
    key: &[u8; KEY_LEN],
) -> Result<Vec<u8>, CredentialError> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| CredentialError::Decryption(e.to_string()))?;

    let nonce = Nonce::from_slice(nonce);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| CredentialError::Decryption(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let password = b"master-password";
        let salt = generate_salt();
        let key = derive_key(password, &salt);

        let plaintext = b"my-secret-credential";
        let (ciphertext, nonce) = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &nonce, &key).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = derive_key(b"password1", &generate_salt());
        let key2 = derive_key(b"password2", &generate_salt());

        let (ciphertext, nonce) = encrypt(b"secret", &key1).unwrap();
        assert!(decrypt(&ciphertext, &nonce, &key2).is_err());
    }
}
