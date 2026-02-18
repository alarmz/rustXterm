use std::sync::{Arc, Mutex};

use rustxterm_filemanager::local::LocalFs;
use rustxterm_filemanager::operations::{TransferDirection, TransferInfo, TransferStatus};
use rustxterm_ssh::{FileEntry, SshSftpSession};
use tauri::State;

use crate::session_manager::{ActiveSession, AppSessionManager};

// ── SFTP session lifecycle ──────────────────────────────────────────

#[tauri::command]
pub async fn open_sftp(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
) -> Result<(), String> {
    // Get the Arc<Handle> from the active SSH session (briefly lock, then release).
    let handle = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        match manager.active.get(&session_id) {
            Some(ActiveSession::Ssh { client }) => client.handle(),
            _ => return Err(format!("Not an SSH session: {session_id}")),
        }
    };

    // Open SFTP subsystem outside the lock (async).
    let sftp = SshSftpSession::open(&handle)
        .await
        .map_err(|e| e.to_string())?;

    // Store the SFTP session.
    {
        let mut manager = state.lock().map_err(|e| e.to_string())?;
        manager
            .sftp_sessions
            .insert(session_id, Arc::new(sftp));
    }

    Ok(())
}

#[tauri::command]
pub async fn close_sftp(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
) -> Result<(), String> {
    let sftp = {
        let mut manager = state.lock().map_err(|e| e.to_string())?;
        manager.sftp_sessions.remove(&session_id)
    };

    if let Some(sftp) = sftp {
        sftp.close().await.map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ── SFTP file operations ────────────────────────────────────────────

fn get_sftp(
    state: &State<'_, Mutex<AppSessionManager>>,
    session_id: &str,
) -> Result<Arc<SshSftpSession>, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    manager
        .sftp_sessions
        .get(session_id)
        .cloned()
        .ok_or_else(|| format!("No SFTP session for: {session_id}"))
}

#[tauri::command]
pub async fn sftp_list_dir(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
    path: String,
) -> Result<Vec<FileEntry>, String> {
    let sftp = get_sftp(&state, &session_id)?;
    sftp.read_dir(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sftp_stat(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
    path: String,
) -> Result<FileEntry, String> {
    let sftp = get_sftp(&state, &session_id)?;
    sftp.stat(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sftp_mkdir(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
    path: String,
) -> Result<(), String> {
    let sftp = get_sftp(&state, &session_id)?;
    sftp.mkdir(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sftp_remove(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
    path: String,
    is_dir: bool,
) -> Result<(), String> {
    let sftp = get_sftp(&state, &session_id)?;
    if is_dir {
        sftp.remove_dir(&path).await.map_err(|e| e.to_string())
    } else {
        sftp.remove_file(&path).await.map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn sftp_rename(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
    old_path: String,
    new_path: String,
) -> Result<(), String> {
    let sftp = get_sftp(&state, &session_id)?;
    sftp.rename(&old_path, &new_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sftp_chmod(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
    path: String,
    mode: u32,
) -> Result<(), String> {
    let sftp = get_sftp(&state, &session_id)?;
    sftp.chmod(&path, mode).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sftp_read_file(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
    path: String,
) -> Result<Vec<u8>, String> {
    let sftp = get_sftp(&state, &session_id)?;
    sftp.read_file(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sftp_write_file(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
    path: String,
    data: Vec<u8>,
) -> Result<(), String> {
    let sftp = get_sftp(&state, &session_id)?;
    sftp.write_file(&path, &data)
        .await
        .map_err(|e| e.to_string())
}

// ── File transfers ──────────────────────────────────────────────────

#[tauri::command]
pub async fn sftp_download(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
    remote_path: String,
    local_path: String,
) -> Result<String, String> {
    let sftp = get_sftp(&state, &session_id)?;

    // Stat the remote file to get its size.
    let entry = sftp.stat(&remote_path).await.map_err(|e| e.to_string())?;

    // Register the transfer.
    let transfer_id = {
        let mut manager = state.lock().map_err(|e| e.to_string())?;
        manager.transfer_manager.register(
            remote_path.clone(),
            local_path.clone(),
            TransferDirection::Download,
            entry.size,
        )
    };

    // Read remote file data.
    let data = sftp
        .read_file(&remote_path)
        .await
        .map_err(|e| e.to_string())?;

    // Write to local filesystem.
    tokio::fs::write(&local_path, &data)
        .await
        .map_err(|e| e.to_string())?;

    // Mark completed.
    {
        let mut manager = state.lock().map_err(|e| e.to_string())?;
        manager.transfer_manager.update_progress(
            &transfer_id,
            data.len() as u64,
            TransferStatus::Completed,
        );
    }

    Ok(transfer_id)
}

#[tauri::command]
pub async fn sftp_upload(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
    local_path: String,
    remote_path: String,
) -> Result<String, String> {
    let sftp = get_sftp(&state, &session_id)?;

    // Read local file.
    let data = tokio::fs::read(&local_path)
        .await
        .map_err(|e| e.to_string())?;

    // Register the transfer.
    let transfer_id = {
        let mut manager = state.lock().map_err(|e| e.to_string())?;
        manager.transfer_manager.register(
            local_path.clone(),
            remote_path.clone(),
            TransferDirection::Upload,
            data.len() as u64,
        )
    };

    // Write to remote filesystem.
    sftp.write_file(&remote_path, &data)
        .await
        .map_err(|e| e.to_string())?;

    // Mark completed.
    {
        let mut manager = state.lock().map_err(|e| e.to_string())?;
        manager.transfer_manager.update_progress(
            &transfer_id,
            data.len() as u64,
            TransferStatus::Completed,
        );
    }

    Ok(transfer_id)
}

#[tauri::command]
pub fn sftp_cancel_transfer(
    state: State<'_, Mutex<AppSessionManager>>,
    transfer_id: String,
) -> Result<bool, String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    Ok(manager.transfer_manager.cancel(&transfer_id))
}

#[tauri::command]
pub fn sftp_list_transfers(
    state: State<'_, Mutex<AppSessionManager>>,
) -> Result<Vec<TransferInfo>, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    Ok(manager.transfer_manager.list())
}

// ── Local filesystem operations ─────────────────────────────────────

#[tauri::command]
pub async fn local_list_dir(path: String) -> Result<Vec<FileEntry>, String> {
    LocalFs::list_dir(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn local_home_dir() -> Result<String, String> {
    LocalFs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())
}
