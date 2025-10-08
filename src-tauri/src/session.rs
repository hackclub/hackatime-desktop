use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

use crate::auth::AuthState;
use crate::config::ApiConfig;
use crate::discord_rpc::DiscordRpcService;
use crate::push_log;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeartbeatData {
    pub id: u32,
    pub project: Option<String>,
    pub editor: Option<String>,
    pub language: Option<String>,
    pub entity: Option<String>,
    pub time: f64,
    #[serde(default)]
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionState {
    pub is_active: bool,
    pub start_time: Option<i64>,
    pub last_heartbeat_id: Option<u32>,
    pub heartbeat_count: u32,
    pub project: Option<String>,
    pub editor: Option<String>,
    pub language: Option<String>,
    pub entity: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeartbeatResponse {
    pub heartbeat: Option<HeartbeatData>,
}

#[tauri::command]
pub async fn get_latest_heartbeat(
    api_config: ApiConfig,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
    discord_rpc_state: State<'_, Arc<tauri::async_runtime::Mutex<DiscordRpcService>>>,
    session_state: State<'_, Arc<tauri::async_runtime::Mutex<SessionState>>>,
) -> Result<HeartbeatResponse, String> {
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
    let request_url = format!(
        "{}/api/v1/authenticated/heartbeats/latest",
        base_url
    );
    
    push_log("info", "backend", format!("Fetching latest heartbeat from: {}", request_url));
    
    let response = client
        .get(&request_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            push_log("error", "backend", format!("HTTP request failed: {}", e));
            format!("Failed to get latest heartbeat: {}", e)
        })?;

    let status = response.status();
    push_log("info", "backend", format!("API response status: {} {}", status.as_u16(), status.canonical_reason().unwrap_or("")));
    
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        push_log("error", "backend", format!("API returned error response: {}", error_text));
        
        if status == 429 {
            push_log("warn", "backend", "Rate limited, will retry later".to_string());
            return Err(format!("Rate limited: {}", error_text));
        }

        return Err(format!("Failed to get latest heartbeat: {}", error_text));
    }

    let response_text = response
        .text()
        .await
        .map_err(|e| {
            push_log("error", "backend", format!("Failed to read response body: {}", e));
            format!("Failed to read response: {}", e)
        })?;
    
    push_log("info", "backend", format!("Raw API response: {}", response_text));

    let heartbeat_data: Option<HeartbeatData> = if response_text.trim() == "null" || response_text.trim().is_empty() {
        None
    } else {
        match serde_json::from_str::<HeartbeatData>(&response_text) {
            Ok(mut data) => {
                if data.timestamp == 0 {
                    data.timestamp = data.time as i64;
                }
                push_log("info", "backend", format!("Successfully parsed heartbeat data: {:?}", data));
                Some(data)
            }
            Err(e) => {
                push_log("error", "backend", format!("Failed to parse heartbeat JSON: {}", e));
                None
            }
        }
    };

    let heartbeat_response = HeartbeatResponse {
        heartbeat: heartbeat_data,
    };

    
    push_log("info", "backend", format!("Latest heartbeat response: {:?}", heartbeat_response));
    
    
    if let Some(heartbeat) = &heartbeat_response.heartbeat {
        
        push_log("info", "backend", format!(
            "Heartbeat details - ID: {}, Project: {}, Language: {}, Editor: {}, Time: {}",
            heartbeat.id,
            heartbeat.project.as_ref().unwrap_or(&"None".to_string()),
            heartbeat.language.as_ref().unwrap_or(&"None".to_string()),
            heartbeat.editor.as_ref().unwrap_or(&"None".to_string()),
            heartbeat.timestamp
        ));
        
        let mut session = session_state.lock().await;
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        
        let heartbeat_age = current_time - heartbeat.timestamp;
        let is_recent = heartbeat_age < 180; 
        
        push_log("debug", "backend", format!(
            "Heartbeat age: {} seconds, is_recent: {}",
            heartbeat_age,
            is_recent
        ));

        
        let is_duplicate = session.last_heartbeat_id == Some(heartbeat.id);

        if is_duplicate && is_recent {
            push_log("debug", "backend", "Duplicate heartbeat but still recent, continuing session".to_string());
        } else if is_duplicate && !is_recent {
            push_log("info", "backend", "Duplicate heartbeat and too old, ending session".to_string());
            session.is_active = false;
            session.start_time = None;
            session.last_heartbeat_id = None;
            session.heartbeat_count = 0;
            session.project = None;
            session.editor = None;
            session.language = None;
            session.entity = None;

            
            let mut discord_rpc = discord_rpc_state.lock().await;
            if discord_rpc.is_connected() {
                let _ = discord_rpc.clear_activity();
            }
        } else if is_recent && !session.is_active {
            
            push_log("info", "backend", "Recent heartbeat detected, starting new session".to_string());
            session.is_active = true;
            session.start_time = Some(heartbeat.timestamp);
            session.last_heartbeat_id = Some(heartbeat.id);
            session.heartbeat_count = 1;
            session.project = heartbeat.project.clone();
            session.editor = heartbeat.editor.clone();
            session.language = heartbeat.language.clone();
            session.entity = heartbeat.entity.clone();

            
            let mut discord_rpc = discord_rpc_state.lock().await;
            if discord_rpc.is_connected() {
                if let Err(e) =
                    discord_rpc.update_activity_from_session(heartbeat, heartbeat.timestamp)
                {
                    push_log("warn", "backend", format!("Failed to update Discord RPC: {}", e));
                }
            }
        } else if is_recent && session.is_active {
            
            session.last_heartbeat_id = Some(heartbeat.id);
            session.heartbeat_count += 1;
            session.project = heartbeat.project.clone();
            session.editor = heartbeat.editor.clone();
            session.language = heartbeat.language.clone();
            session.entity = heartbeat.entity.clone();

            
            let mut discord_rpc = discord_rpc_state.lock().await;
            if discord_rpc.is_connected() {
                if let Err(e) = discord_rpc.update_activity_from_session(
                    heartbeat,
                    session.start_time.unwrap_or(heartbeat.timestamp),
                ) {
                    push_log("warn", "backend", format!("Failed to update Discord RPC: {}", e));
                }
            }
        } else if !is_recent && session.is_active {
            
            push_log("info", "backend", "Heartbeat too old, ending session".to_string());
            session.is_active = false;
            session.start_time = None;
            session.last_heartbeat_id = None;
            session.heartbeat_count = 0;
            session.project = None;
            session.editor = None;
            session.language = None;
            session.entity = None;

            
            let mut discord_rpc = discord_rpc_state.lock().await;
            if discord_rpc.is_connected() {
                let _ = discord_rpc.clear_activity();
            }
        }
    } else {
        
        push_log("info", "backend", "No heartbeat data in response (heartbeat is null)".to_string());
        let mut session = session_state.lock().await;
        if session.is_active {
            push_log("info", "backend", "No heartbeat data, ending session".to_string());
            session.is_active = false;
            session.start_time = None;
            session.last_heartbeat_id = None;
            session.heartbeat_count = 0;
            session.project = None;
            session.editor = None;
            session.language = None;
            session.entity = None;

            
            let mut discord_rpc = discord_rpc_state.lock().await;
            if discord_rpc.is_connected() {
                let _ = discord_rpc.clear_activity();
            }
        }
    }

    Ok(heartbeat_response)
}

#[tauri::command]
pub async fn get_current_session(
    session_state: State<'_, Arc<tauri::async_runtime::Mutex<SessionState>>>,
) -> Result<SessionState, String> {
    let session = session_state.lock().await;
    Ok(session.clone())
}

#[tauri::command]
pub async fn get_app_status(
    auth_state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
    session_state: State<'_, Arc<tauri::async_runtime::Mutex<SessionState>>>,
    discord_rpc_state: State<'_, Arc<tauri::async_runtime::Mutex<DiscordRpcService>>>,
) -> Result<serde_json::Value, String> {
    let auth = auth_state.lock().await;
    let session = session_state.lock().await;
    let discord_rpc = discord_rpc_state.lock().await;

    Ok(serde_json::json!({
        "authenticated": auth.is_authenticated,
        "session_active": session.is_active,
        "session_duration": if session.is_active && session.start_time.is_some() {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            current_time - session.start_time.unwrap()
        } else {
            0
        },
        "project": session.project.clone().unwrap_or_else(|| "No project".to_string()),
        "editor": session.editor.clone().unwrap_or_else(|| "No editor".to_string()),
        "language": session.language.clone().unwrap_or_else(|| "No language".to_string()),
        "discord_connected": discord_rpc.is_connected(),
        "heartbeat_count": session.heartbeat_count
    }))
}

