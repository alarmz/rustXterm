#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod menu;
mod pty_manager;
mod session_manager;

use session_manager::AppSessionManager;
use std::sync::Mutex;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting RustXterm application");

    let manager = AppSessionManager::new().expect("Failed to initialize session manager");

    tauri::Builder::default()
        .manage(Mutex::new(manager))
        .setup(|app| {
            menu::setup_menu(app)?;
            Ok(())
        })
        .on_menu_event(menu::handle_menu_event)
        .invoke_handler(tauri::generate_handler![
            // Local shell
            commands::spawn_shell,
            // SSH
            commands::connect_ssh,
            // Unified I/O
            commands::write_to_pty,
            commands::resize_pty,
            commands::close_pty,
            // Session bookmarks
            commands::list_saved_sessions,
            commands::save_session,
            commands::delete_saved_session,
            commands::get_saved_session,
            // Credentials
            commands::list_credentials,
            commands::save_credential,
            commands::delete_credential,
        ])
        .run(tauri::generate_context!())
        .expect("error while running RustXterm");
}
