use tauri::{AppHandle, Manager};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use crate::push_log;

pub fn setup_app_menu(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    
    let about_quit = MenuItem::with_id(app, "quit", "Quit Hackatime", true, None::<&str>)?;
    let about_menu = Submenu::with_items(
        app,
        "About",
        true,
        &[
            &about_quit,
        ],
    )?;

    
    let file_new = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let file_hide = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
    let file_menu = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &file_new,
            &file_hide,
        ],
    )?;

    
    let edit_undo = PredefinedMenuItem::undo(app, Some("Undo"))?;
    let edit_redo = PredefinedMenuItem::redo(app, Some("Redo"))?;
    let edit_cut = PredefinedMenuItem::cut(app, Some("Cut"))?;
    let edit_copy = PredefinedMenuItem::copy(app, Some("Copy"))?;
    let edit_paste = PredefinedMenuItem::paste(app, Some("Paste"))?;
    let edit_select_all = PredefinedMenuItem::select_all(app, Some("Select All"))?;
    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &edit_undo,
            &edit_redo,
            &PredefinedMenuItem::separator(app)?,
            &edit_cut,
            &edit_copy,
            &edit_paste,
            &PredefinedMenuItem::separator(app)?,
            &edit_select_all,
        ],
    )?;

    
    let help_item = MenuItem::with_id(app, "help", "Help", true, None::<&str>)?;
    let help_menu = Submenu::with_items(app, "Help", true, &[&help_item])?;

    
    let app_menu = Menu::with_items(app, &[&about_menu, &file_menu, &edit_menu, &help_menu])?;

    app.set_menu(app_menu)?;

    
    app.on_menu_event(|app, event| {
        match event.id.as_ref() {
            "quit" => app.exit(0),
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            "help" => {
                push_log("info", "backend", "📖 Help: Window closes to tray. Use the menu bar or tray icon to reopen.".to_string());
            }
            _ => {}
        }
    });

    Ok(())
}


