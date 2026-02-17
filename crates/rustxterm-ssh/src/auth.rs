use std::path::Path;
use std::sync::Arc;

use crate::error::SshError;

/// Authenticate with password.
pub async fn authenticate_password(
    handle: &mut russh::client::Handle<super::client::SshHandler>,
    username: &str,
    password: &str,
) -> Result<bool, SshError> {
    handle
        .authenticate_password(username, password)
        .await
        .map_err(|e| SshError::AuthenticationFailed(e.to_string()))
}

/// Authenticate with a public key file.
pub async fn authenticate_publickey(
    handle: &mut russh::client::Handle<super::client::SshHandler>,
    username: &str,
    key_path: &Path,
    passphrase: Option<&str>,
) -> Result<bool, SshError> {
    let key_pair = russh_keys::load_secret_key(key_path, passphrase)
        .map_err(|e| SshError::AuthenticationFailed(format!("failed to load key: {e}")))?;

    handle
        .authenticate_publickey(username, Arc::new(key_pair))
        .await
        .map_err(|e| SshError::AuthenticationFailed(e.to_string()))
}
