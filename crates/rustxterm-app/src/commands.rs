use crate::pty_manager::PtyManager;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

#[derive(Clone, Serialize)]
struct PtyOutputEvent {
    session_id: String,
    data: Vec<u8>,
}

#[tauri::command]
pub fn spawn_shell(
    app: AppHandle,
    state: State<'_, Mutex<PtyManager>>,
    cols: u16,
    rows: u16,
) -> Result<String, String> {
    let session_id = uuid::Uuid::new_v4().to_string();

    let reader = {
        let mut manager = state.lock().map_err(|e| e.to_string())?;
        manager
            .spawn(&session_id, cols, rows)
            .map_err(|e| e.to_string())?
    };

    // Spawn a background thread to read PTY output and emit events to frontend.
    // Uses std::thread (not tokio) because portable-pty readers are blocking I/O.
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
                    let _ = app_handle.emit(
                        "pty-output",
                        PtyOutputEvent {
                            session_id: sid.clone(),
                            data: buf[..n].to_vec(),
                        },
                    );
                }
                Err(_) => break,
            }
        }
    });

    Ok(session_id)
}

#[tauri::command]
pub fn write_to_pty(
    state: State<'_, Mutex<PtyManager>>,
    session_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager
        .write(&session_id, &data)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resize_pty(
    state: State<'_, Mutex<PtyManager>>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager
        .resize(&session_id, cols, rows)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn close_pty(
    state: State<'_, Mutex<PtyManager>>,
    session_id: String,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.close(&session_id);
    Ok(())
}
