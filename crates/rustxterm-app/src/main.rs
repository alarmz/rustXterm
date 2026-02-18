#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod menu;
mod pty_manager;
mod session_manager;
mod sftp_commands;
mod tunnel_commands;

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
            // SFTP
            sftp_commands::open_sftp,
            sftp_commands::close_sftp,
            sftp_commands::sftp_list_dir,
            sftp_commands::sftp_stat,
            sftp_commands::sftp_mkdir,
            sftp_commands::sftp_remove,
            sftp_commands::sftp_rename,
            sftp_commands::sftp_chmod,
            sftp_commands::sftp_read_file,
            sftp_commands::sftp_write_file,
            sftp_commands::sftp_download,
            sftp_commands::sftp_upload,
            sftp_commands::sftp_cancel_transfer,
            sftp_commands::sftp_list_transfers,
            // Local filesystem
            sftp_commands::local_list_dir,
            sftp_commands::local_home_dir,
            // Tunnels
            tunnel_commands::create_local_forward,
            tunnel_commands::create_remote_forward,
            tunnel_commands::create_dynamic_forward,
            tunnel_commands::stop_tunnel,
            tunnel_commands::list_tunnels,
            tunnel_commands::save_tunnel_config,
            tunnel_commands::list_tunnel_configs,
            tunnel_commands::delete_tunnel_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running RustXterm");
}
