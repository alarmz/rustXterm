use crate::session_manager::AppSessionManager;
use rustxterm_core::session::SessionInfo;
use rustxterm_credentials::CredentialRecord;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

#[derive(Clone, Serialize)]
struct PtyOutputEvent {
    session_id: String,
    data: Vec<u8>,
}

// ── Local shell ──────────────────────────────────────────────────────

#[tauri::command]
pub fn spawn_shell(
    app: AppHandle,
    state: State<'_, Mutex<AppSessionManager>>,
    cols: u16,
    rows: u16,
) -> Result<String, String> {
    let session_id = uuid::Uuid::new_v4().to_string();

    let reader = {
        let mut manager = state.lock().map_err(|e| e.to_string())?;
        manager.spawn_local(&session_id, cols, rows)?
    };

    // Background thread reads PTY output and emits events to frontend.
    // Uses std::thread because portable-pty readers are blocking I/O.
    let app_handle = app.clone();
    let sid = session_id.clone();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut reader = reader;
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if let Err(e) = app_handle.emit(
                        "pty-output",
                        PtyOutputEvent {
                            session_id: sid.clone(),
                            data: buf[..n].to_vec(),
                        },
                    ) {
                        tracing::warn!(session_id = %sid, "failed to emit pty-output: {e}");
                        break;
                    }
                }
                Err(e) => {
                    tracing::debug!(session_id = %sid, "PTY read ended: {e}");
                    break;
                }
            }
        }
    });

    Ok(session_id)
}

// ── SSH connection ───────────────────────────────────────────────────

#[tauri::command]
pub async fn connect_ssh(
    app: AppHandle,
    state: State<'_, Mutex<AppSessionManager>>,
    host: String,
    port: u16,
    username: String,
    password: String,
    cols: u16,
    rows: u16,
) -> Result<String, String> {
    let session_id = uuid::Uuid::new_v4().to_string();

    // Connect SSH outside the mutex lock to avoid holding MutexGuard across await.
    let (client, data_rx) =
        rustxterm_ssh::SshClient::connect(&host, port, &username, &password, cols, rows)
            .await
            .map_err(|e| e.to_string())?;

    // Store the session under the lock (sync, no await).
    {
        let mut manager = state.lock().map_err(|e| e.to_string())?;
        manager.insert_ssh_session(&session_id, client);
    }

    // Background tokio task reads SSH output and emits same pty-output events.
    let app_handle = app.clone();
    let sid = session_id.clone();
    tokio::spawn(async move {
        let mut data_rx = data_rx;
        while let Some(data) = data_rx.recv().await {
            if let Err(e) = app_handle.emit(
                "pty-output",
                PtyOutputEvent {
                    session_id: sid.clone(),
                    data,
                },
            ) {
                tracing::warn!(session_id = %sid, "failed to emit pty-output: {e}");
                break;
            }
        }
    });

    Ok(session_id)
}

// ── Unified write / resize / close ───────────────────────────────────

#[tauri::command]
pub fn write_to_pty(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.write(&session_id, &data)
}

#[tauri::command]
pub fn resize_pty(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.resize(&session_id, cols, rows)
}

#[tauri::command]
pub fn close_pty(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.close(&session_id);
    Ok(())
}

// ── Session bookmarks ────────────────────────────────────────────────

#[tauri::command]
pub fn list_saved_sessions(
    state: State<'_, Mutex<AppSessionManager>>,
) -> Result<Vec<SessionInfo>, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    manager.session_db.list_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_session(
    state: State<'_, Mutex<AppSessionManager>>,
    session_info: SessionInfo,
) -> Result<i64, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    manager
        .session_db
        .save_session(&session_info)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_saved_session(
    state: State<'_, Mutex<AppSessionManager>>,
    id: i64,
) -> Result<bool, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    manager
        .session_db
        .delete_session(id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_saved_session(
    state: State<'_, Mutex<AppSessionManager>>,
    id: i64,
) -> Result<SessionInfo, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    manager
        .session_db
        .load_session(id)
        .map_err(|e| e.to_string())
}

// ── Credentials ──────────────────────────────────────────────────────

#[tauri::command]
pub fn list_credentials(
    state: State<'_, Mutex<AppSessionManager>>,
) -> Result<Vec<CredentialRecord>, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    manager
        .credential_store
        .list()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_credential(
    state: State<'_, Mutex<AppSessionManager>>,
    name: String,
    username: String,
    password: String,
) -> Result<i64, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    manager
        .credential_store
        .save(&name, &username, &password)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_credential(
    state: State<'_, Mutex<AppSessionManager>>,
    id: i64,
) -> Result<bool, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    manager
        .credential_store
        .delete(id)
        .map_err(|e| e.to_string())
}
