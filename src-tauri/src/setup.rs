use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct WakatimeConfigCheck {
    pub exists: bool,
    pub matches: bool,
    pub expected_content: String,
    pub actual_content: Option<String>,
    pub config_path: String,
}

fn get_wakatime_config_path() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let userprofile = std::env::var("USERPROFILE")
            .map_err(|_| "Failed to get USERPROFILE directory")?;
        Ok(format!("{}\\.wakatime.cfg", userprofile))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let home_dir = std::env::var("HOME")
            .map_err(|_| "Failed to get home directory")?;
        Ok(format!("{}/.wakatime.cfg", home_dir))
    }
}

fn get_expected_config_content(api_key: &str, api_url: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        format!(
            "[settings]\r\napi_url = {}\r\napi_key = {}\r\nheartbeat_rate_limit_seconds = 30\r\n",
            api_url, api_key
        )
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        format!(
            "[settings]\napi_url = {}\napi_key = {}\nheartbeat_rate_limit_seconds = 30\n",
            api_url, api_key
        )
    }
}

fn normalize_config_content(content: &str) -> String {
    
    content
        .replace("\r\n", "\n")
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[tauri::command]
pub async fn check_wakatime_config(api_key: String, api_url: String) -> Result<WakatimeConfigCheck, String> {
    let config_path = get_wakatime_config_path()?;
    let expected_content = get_expected_config_content(&api_key, &api_url);
    
    let exists = Path::new(&config_path).exists();
    let actual_content = if exists {
        match fs::read_to_string(&config_path) {
            Ok(content) => Some(content),
            Err(e) => return Err(format!("Failed to read config file: {}", e)),
        }
    } else {
        None
    };
    
    let matches = if let Some(ref actual) = actual_content {
        normalize_config_content(actual) == normalize_config_content(&expected_content)
    } else {
        false
    };
    
    Ok(WakatimeConfigCheck {
        exists,
        matches,
        expected_content,
        actual_content,
        config_path,
    })
}

#[tauri::command]
pub async fn apply_wakatime_config(api_key: String, api_url: String) -> Result<String, String> {
    let config_path = get_wakatime_config_path()?;
    let backup_path = format!("{}.bak", config_path);
    
    
    if Path::new(&config_path).exists() {
        if let Err(e) = fs::copy(&config_path, &backup_path) {
            return Err(format!("Failed to backup existing config: {}", e));
        }
    }
    
    let config_content = get_expected_config_content(&api_key, &api_url);
    
    if let Err(e) = fs::write(&config_path, &config_content) {
        return Err(format!("Failed to write config file: {}", e));
    }
    
    Ok(format!("Config file successfully written to {}", config_path))
}

#[tauri::command]
pub async fn setup_hackatime_macos_linux(api_key: String, api_url: String) -> Result<String, String> {
    let home_dir = std::env::var("HOME").map_err(|_| "Failed to get home directory")?;

    let config_path = format!("{}/.wakatime.cfg", home_dir);
    let backup_path = format!("{}/.wakatime.cfg.bak", home_dir);

    if Path::new(&config_path).exists() {
        if let Err(e) = fs::rename(&config_path, &backup_path) {
            return Err(format!("Failed to backup existing config: {}", e));
        }
    }

    let config_content = format!(
        "[settings]\napi_url = {}\napi_key = {}\nheartbeat_rate_limit_seconds = 30\n",
        api_url, api_key
    );

    if let Err(e) = fs::write(&config_path, config_content) {
        return Err(format!("Failed to write config file: {}", e));
    }

    if !Path::new(&config_path).exists() {
        return Err("Config file was not created".to_string());
    }

    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let lines: Vec<&str> = config_content.lines().collect();
    let mut found_api_url = false;
    let mut found_api_key = false;
    let mut found_heartbeat_rate = false;

    for line in lines {
        if line.starts_with("api_url =") {
            found_api_url = true;
        } else if line.starts_with("api_key =") {
            found_api_key = true;
        } else if line.starts_with("heartbeat_rate_limit_seconds =") {
            found_heartbeat_rate = true;
        }
    }

    if !found_api_url || !found_api_key || !found_heartbeat_rate {
        return Err("Config file is missing required fields".to_string());
    }

    Ok(format!(
        "Config file created successfully at {}",
        config_path
    ))
}

#[tauri::command]
pub async fn setup_hackatime_windows(api_key: String, api_url: String) -> Result<String, String> {
    let userprofile =
        std::env::var("USERPROFILE").map_err(|_| "Failed to get USERPROFILE directory")?;

    let config_path = format!("{}\\.wakatime.cfg", userprofile);
    let backup_path = format!("{}\\.wakatime.cfg.bak", userprofile);

    if Path::new(&config_path).exists() {
        if let Err(e) = fs::rename(&config_path, &backup_path) {
            return Err(format!("Failed to backup existing config: {}", e));
        }
    }

    let config_content = format!(
        "[settings]\r\napi_url = {}\r\napi_key = {}\r\nheartbeat_rate_limit_seconds = 30\r\n",
        api_url, api_key
    );

    if let Err(e) = fs::write(&config_path, config_content) {
        return Err(format!("Failed to write config file: {}", e));
    }

    if !Path::new(&config_path).exists() {
        return Err("Config file was not created".to_string());
    }

    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let lines: Vec<&str> = config_content.lines().collect();
    let mut found_api_url = false;
    let mut found_api_key = false;
    let mut found_heartbeat_rate = false;

    for line in lines {
        if line.starts_with("api_url =") {
            found_api_url = true;
        } else if line.starts_with("api_key =") {
            found_api_key = true;
        } else if line.starts_with("heartbeat_rate_limit_seconds =") {
            found_heartbeat_rate = true;
        }
    }

    if !found_api_url || !found_api_key || !found_heartbeat_rate {
        return Err("Config file is missing required fields".to_string());
    }

    Ok(format!(
        "Config file created successfully at {}",
        config_path
    ))
}

#[tauri::command]
pub async fn test_hackatime_heartbeat(api_key: String, api_url: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let heartbeat_data = serde_json::json!([{
        "type": "file",
        "time": current_time,
        "entity": "test.txt",
        "language": "Text"
    }]);

    let response = client
        .post(&format!("{}/users/current/heartbeats", api_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&heartbeat_data)
        .send()
        .await
        .map_err(|e| format!("Failed to send heartbeat: {}", e))?;

    if response.status().is_success() {
        Ok("Test heartbeat sent successfully!".to_string())
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Heartbeat failed: {}", error_text))
    }
}

#[tauri::command]
pub async fn setup_hackatime_complete(api_key: String, api_url: String) -> Result<String, String> {
    
    if cfg!(target_os = "windows") {
        setup_hackatime_windows(api_key, api_url).await
    } else {
        setup_hackatime_macos_linux(api_key, api_url).await
    }
}

