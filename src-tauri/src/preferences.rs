use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;
use crate::database::get_hackatime_config_dir;
use crate::push_log;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Preferences {
    pub autostart_enabled: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            autostart_enabled: false,
        }
    }
}

fn get_preferences_path() -> Result<PathBuf, String> {
    let config_dir = get_hackatime_config_dir()?;
    Ok(config_dir.join("preferences.json"))
}

pub fn load_preferences() -> Result<Preferences, String> {
    let path = get_preferences_path()?;
    
    if !path.exists() {
        push_log("info", "backend", "No preferences file found, using defaults".to_string());
        return Ok(Preferences::default());
    }

    let contents = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read preferences file: {}", e))?;

    let preferences: Preferences = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse preferences: {}", e))?;

    push_log("info", "backend", "Loaded preferences successfully".to_string());
    Ok(preferences)
}

pub fn save_preferences(preferences: &Preferences) -> Result<(), String> {
    let path = get_preferences_path()?;
    
    let contents = serde_json::to_string_pretty(preferences)
        .map_err(|e| format!("Failed to serialize preferences: {}", e))?;

    fs::write(&path, contents)
        .map_err(|e| format!("Failed to write preferences file: {}", e))?;

    push_log("info", "backend", "Saved preferences successfully".to_string());
    Ok(())
}

#[tauri::command]
pub fn get_preferences() -> Result<Preferences, String> {
    load_preferences()
}

#[tauri::command]
pub fn set_autostart_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut preferences = load_preferences().unwrap_or_default();
    preferences.autostart_enabled = enabled;
    save_preferences(&preferences)?;
    
    let autolaunch_manager = app.autolaunch();
    if enabled {
        autolaunch_manager.enable()
            .map_err(|e| format!("Failed to enable autostart: {}", e))?;
        push_log("info", "backend", "Autostart enabled".to_string());
    } else {
        autolaunch_manager.disable()
            .map_err(|e| format!("Failed to disable autostart: {}", e))?;
        push_log("info", "backend", "Autostart disabled".to_string());
    }
    
    Ok(())
}

#[tauri::command]
pub fn get_autostart_enabled() -> Result<bool, String> {
    let preferences = load_preferences().unwrap_or_default();
    Ok(preferences.autostart_enabled)
}

