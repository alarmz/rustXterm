use crate::error::CredentialError;
use rand::RngCore;

const SERVICE_NAME: &str = "rustxterm";
const MASTER_KEY_USER: &str = "master-key";

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(s: &str) -> Result<Vec<u8>, CredentialError> {
    if s.len() % 2 != 0 {
        return Err(CredentialError::Keyring(
            "invalid hex in stored key: odd length".to_string(),
        ));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| CredentialError::Keyring(format!("invalid hex in stored key: {e}")))
        })
        .collect()
}

/// Store the master encryption key in the OS keyring.
pub fn store_master_key(key: &[u8]) -> Result<(), CredentialError> {
    let encoded = hex_encode(key);
    let entry = keyring::Entry::new(SERVICE_NAME, MASTER_KEY_USER)
        .map_err(|e| CredentialError::Keyring(e.to_string()))?;
    entry
        .set_password(&encoded)
        .map_err(|e| CredentialError::Keyring(e.to_string()))
}

/// Retrieve the master encryption key from the OS keyring.
/// Returns None if no key is stored.
pub fn get_master_key() -> Result<Option<Vec<u8>>, CredentialError> {
    let entry = keyring::Entry::new(SERVICE_NAME, MASTER_KEY_USER)
        .map_err(|e| CredentialError::Keyring(e.to_string()))?;
    match entry.get_password() {
        Ok(encoded) => {
            let key = hex_decode(&encoded)?;
            Ok(Some(key))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(CredentialError::Keyring(e.to_string())),
    }
}

/// Get the master key from keyring, or generate and store a new one.
pub fn get_or_create_master_key() -> Result<Vec<u8>, CredentialError> {
    if let Some(key) = get_master_key()? {
        return Ok(key);
    }

    let mut key = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    store_master_key(&key)?;
    Ok(key)
}
