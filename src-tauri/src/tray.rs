use tauri::{AppHandle, Manager};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let status_text = {
        let state = app.state::<std::sync::Arc<tauri::async_runtime::Mutex<crate::SessionState>>>();
        let state_arc = state.inner().clone();
        tauri::async_runtime::block_on(async {
            let guard = state_arc.lock().await;
            if guard.is_active {
                let project = guard.project.clone().unwrap_or_else(|| "Unknown".to_string());
                format!("Status: Active — {}", project)
            } else {
                "No active session".to_string()
            }
        })
    };
    let status_item = MenuItem::with_id(app, "status", &status_text, false, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    
    let menu = Menu::with_items(app, &[
        &status_item,
        &tauri::menu::PredefinedMenuItem::separator(app)?,
        &quit_item,
    ])?;
    
    
    let icon = app.default_window_icon()
        .ok_or("No default window icon found")?
        .clone();
    
    let _tray_icon = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                            #[cfg(target_os = "macos")]
                            let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                        } else {
                            #[cfg(target_os = "macos")]
                            let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
                            
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
                TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                } => {
                    let app = tray.app_handle();
                    
                    #[cfg(target_os = "macos")]
                    let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
                    
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {
                }
            }
        })
        .build(app)?;

    Ok(())
}

