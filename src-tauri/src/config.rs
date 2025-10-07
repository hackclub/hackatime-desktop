use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConfig {
    pub base_url: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "https://hackatime.hackclub.com".to_string(),
        }
    }
}

impl ApiConfig {
    pub fn new() -> Self {
        Self {
            base_url: "https://hackatime.hackclub.com".to_string(),
        }
    }
}

#[tauri::command]
pub async fn get_api_config(state: State<'_, ApiConfig>) -> Result<ApiConfig, String> {
    Ok(state.inner().clone())
}

#[tauri::command]
pub async fn set_api_config(
    new_config: ApiConfig,
    state: State<'_, tauri::async_runtime::Mutex<ApiConfig>>,
) -> Result<(), String> {
    let mut config = state.lock().await;
    *config = new_config;
    Ok(())
}

