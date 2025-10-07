use tauri::Manager;

#[tauri::command]
pub async fn show_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .show()
            .map_err(|e| format!("Failed to show window: {}", e))?;
        window
            .set_focus()
            .map_err(|e| format!("Failed to focus window: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn hide_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .hide()
            .map_err(|e| format!("Failed to hide window: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            window
                .hide()
                .map_err(|e| format!("Failed to hide window: {}", e))?;
        } else {
            window
                .show()
                .map_err(|e| format!("Failed to show window: {}", e))?;
            window
                .set_focus()
                .map_err(|e| format!("Failed to focus window: {}", e))?;
        }
    }
    Ok(())
}

