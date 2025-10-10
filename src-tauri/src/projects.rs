use std::sync::Arc;
use tauri::State;

use crate::auth::AuthState;
use crate::config::ApiConfig;
use crate::push_log;

#[tauri::command]
pub async fn get_projects(
    api_config: ApiConfig,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<serde_json::Value, String> {
    let auth_state = state.lock().await;

    if !auth_state.is_authenticated {
        return Err("Not authenticated".to_string());
    }

    let base_url = if api_config.base_url.is_empty() {
        "https://hackatime.hackclub.com"
    } else {
        &api_config.base_url
    };

    let access_token = auth_state
        .access_token
        .as_ref()
        .ok_or("No access token available")?;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!(
            "{}/api/v1/authenticated/projects",
            base_url
        ))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch projects: {}", e))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Projects request failed: {}", error_text));
    }

    let projects_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse projects response: {}", e))?;

    push_log(
        "info",
        "backend",
        format!(
            "RAW PROJECTS API RESPONSE: {}",
            serde_json::to_string_pretty(&projects_response)
                .unwrap_or_else(|_| "Failed to serialize".to_string())
        ),
    );

    Ok(projects_response)
}

#[tauri::command]
pub async fn get_project_details(
    project_name: String,
    api_config: ApiConfig,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<serde_json::Value, String> {
    let auth_state = state.lock().await;

    if !auth_state.is_authenticated {
        return Err("Not authenticated".to_string());
    }

    let base_url = if api_config.base_url.is_empty() {
        "https://hackatime.hackclub.com"
    } else {
        &api_config.base_url
    };

    let access_token = auth_state
        .access_token
        .as_ref()
        .ok_or("No access token available")?;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!(
            "{}/api/v1/authenticated/projects/{}",
            base_url,
            urlencoding::encode(&project_name)
        ))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch project details: {}", e))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Project details request failed: {}", error_text));
    }

    let project_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse project response: {}", e))?;

    Ok(project_response)
}

