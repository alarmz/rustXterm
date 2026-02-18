use std::sync::Mutex;

use rustxterm_session::TunnelConfig;
use rustxterm_tunnel::dynamic::DynamicForwardConfig;
use rustxterm_tunnel::local_forward::LocalForwardConfig;
use rustxterm_tunnel::remote_forward::RemoteForwardConfig;
use rustxterm_tunnel::TunnelInfo;
use tauri::State;

use crate::session_manager::{ActiveSession, AppSessionManager};

// ── Tunnel creation ─────────────────────────────────────────────────

#[tauri::command]
pub fn create_local_forward(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
    local_host: String,
    local_port: u16,
    remote_host: String,
    remote_port: u16,
) -> Result<String, String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;

    let handle = match manager.active.get(&session_id) {
        Some(ActiveSession::Ssh { client }) => client.handle(),
        _ => return Err(format!("Not an SSH session: {session_id}")),
    };

    let config = LocalForwardConfig {
        local_host,
        local_port,
        remote_host,
        remote_port,
    };

    manager
        .tunnel_manager
        .start_local(handle, config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_remote_forward(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
    remote_port: u16,
    local_host: String,
    local_port: u16,
) -> Result<String, String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;

    let handle = match manager.active.get(&session_id) {
        Some(ActiveSession::Ssh { client }) => client.handle(),
        _ => return Err(format!("Not an SSH session: {session_id}")),
    };

    let config = RemoteForwardConfig::new(remote_port, local_host, local_port);

    manager
        .tunnel_manager
        .start_remote(handle, config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_dynamic_forward(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: String,
    local_host: String,
    local_port: u16,
) -> Result<String, String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;

    let handle = match manager.active.get(&session_id) {
        Some(ActiveSession::Ssh { client }) => client.handle(),
        _ => return Err(format!("Not an SSH session: {session_id}")),
    };

    let config = DynamicForwardConfig {
        local_host,
        local_port,
    };

    manager
        .tunnel_manager
        .start_dynamic(handle, config)
        .map_err(|e| e.to_string())
}

// ── Tunnel management ───────────────────────────────────────────────

#[tauri::command]
pub fn stop_tunnel(
    state: State<'_, Mutex<AppSessionManager>>,
    tunnel_id: String,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager
        .tunnel_manager
        .stop(&tunnel_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_tunnels(
    state: State<'_, Mutex<AppSessionManager>>,
) -> Result<Vec<TunnelInfo>, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    Ok(manager.tunnel_manager.list())
}

// ── Saved tunnel configs ────────────────────────────────────────────

#[tauri::command]
pub fn save_tunnel_config(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: i64,
    tunnel_type: String,
    local_port: Option<i64>,
    remote_host: Option<String>,
    remote_port: Option<i64>,
    local_host: Option<String>,
    auto_start: bool,
    name: Option<String>,
) -> Result<i64, String> {
    let config = TunnelConfig {
        id: 0,
        session_id,
        tunnel_type,
        local_port,
        remote_host,
        remote_port,
        local_host,
        auto_start,
        name,
        sort_order: 0,
    };

    let manager = state.lock().map_err(|e| e.to_string())?;
    manager
        .session_db
        .save_tunnel_config(&config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_tunnel_configs(
    state: State<'_, Mutex<AppSessionManager>>,
    session_id: i64,
) -> Result<Vec<TunnelConfig>, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    manager
        .session_db
        .list_tunnel_configs(session_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tunnel_config(
    state: State<'_, Mutex<AppSessionManager>>,
    config_id: i64,
) -> Result<bool, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    manager
        .session_db
        .delete_tunnel_config(config_id)
        .map_err(|e| e.to_string())
}
