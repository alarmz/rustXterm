#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod menu;
mod pty_manager;

use pty_manager::PtyManager;
use std::sync::Mutex;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting RustXterm application");

    tauri::Builder::default()
        .manage(Mutex::new(PtyManager::new()))
        .setup(|app| {
            menu::setup_menu(app)?;
            Ok(())
        })
        .on_menu_event(menu::handle_menu_event)
        .invoke_handler(tauri::generate_handler![
            commands::spawn_shell,
            commands::write_to_pty,
            commands::resize_pty,
            commands::close_pty,
        ])
        .run(tauri::generate_context!())
        .expect("error while running RustXterm");
}
