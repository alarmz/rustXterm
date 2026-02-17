use tauri::menu::{MenuBuilder, SubmenuBuilder};
use tauri::{App, AppHandle, Emitter};

pub fn setup_menu(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let file_menu = SubmenuBuilder::new(app, "File")
        .text("new-tab", "New Tab\tCtrl+T")
        .text("new-ssh", "New SSH Connection\tCtrl+Shift+S")
        .text("close-tab", "Close Tab\tCtrl+W")
        .separator()
        .text("quit", "Quit\tCtrl+Q")
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .text("copy", "Copy\tCtrl+Shift+C")
        .text("paste", "Paste\tCtrl+Shift+V")
        .separator()
        .text("preferences", "Preferences")
        .build()?;

    let view_menu = SubmenuBuilder::new(app, "View")
        .text("toggle-sidebar", "Toggle Sidebar\tCtrl+B")
        .separator()
        .text("zoom-in", "Zoom In\tCtrl++")
        .text("zoom-out", "Zoom Out\tCtrl+-")
        .text("zoom-reset", "Reset Zoom\tCtrl+0")
        .build()?;

    let help_menu = SubmenuBuilder::new(app, "Help")
        .text("about", "About RustXterm")
        .build()?;

    let menu = MenuBuilder::new(app)
        .items(&[&file_menu, &edit_menu, &view_menu, &help_menu])
        .build()?;

    app.set_menu(menu)?;
    Ok(())
}

pub fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    if let Err(e) = app.emit("menu-event", event.id().0.clone()) {
        tracing::warn!("failed to emit menu-event: {e}");
    }
}
