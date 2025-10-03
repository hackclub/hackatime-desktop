use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, State, WindowEvent};
use tauri_plugin_deep_link::DeepLinkExt;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

mod database;
mod discord_rpc;
use database::{
    get_hackatime_config_dir, get_hackatime_data_dir, get_hackatime_logs_dir, get_platform_info,
    AuthState as DbAuthState, Database,
};
use discord_rpc::{DiscordActivity, DiscordRpcService, DiscordRpcState};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AuthState {
    is_authenticated: bool,
    access_token: Option<String>,
    user_info: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ApiConfig {
    base_url: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "https://hackatime.hackclub.com".to_string(),
        }
    }
}

impl ApiConfig {
    fn new() -> Self {
        Self {
            base_url: "https://hackatime.hackclub.com".to_string(),
        }
    }
}

fn generate_code_verifier() -> String {
    let mut rng = thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
}

fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    general_purpose::URL_SAFE_NO_PAD.encode(&hash)
}

fn generate_state() -> String {
    let mut rng = thread_rng();
    (0..32)
        .map(|_| rng.sample(Alphanumeric))
        .map(char::from)
        .collect()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PkceState {
    code_verifier: String,
    state: String,
    timestamp: i64,
}

impl PkceState {
    fn new() -> Self {
        let code_verifier = generate_code_verifier();
        let state = generate_state();
        Self {
            code_verifier,
            state,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }
    
    fn is_expired(&self, max_age_seconds: i64) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        current_time - self.timestamp > max_age_seconds
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_api_config(state: State<'_, ApiConfig>) -> Result<ApiConfig, String> {
    Ok(state.inner().clone())
}

#[tauri::command]
async fn set_api_config(
    new_config: ApiConfig,
    state: State<'_, tauri::async_runtime::Mutex<ApiConfig>>,
) -> Result<(), String> {
    let mut config = state.lock().await;
    *config = new_config;
    Ok(())
}

#[tauri::command]
async fn get_auth_state(
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<AuthState, String> {
    let auth_state = state.lock().await;
    Ok(auth_state.clone())
}

#[tauri::command]
async fn authenticate_with_rails(
    api_config: ApiConfig,
    pkce_state: State<'_, Arc<tauri::async_runtime::Mutex<Option<PkceState>>>>,
    _app_handle: tauri::AppHandle,
) -> Result<(), String> {

    let callback_url = "hackatime://auth/callback";
    
    let pkce = PkceState::new();
    let code_challenge = generate_code_challenge(&pkce.code_verifier);
    
    {
        let mut stored_pkce = pkce_state.lock().await;
        *stored_pkce = Some(pkce.clone());
    }
    
    println!("Generated PKCE parameters - verifier: {}, challenge: {}, state: {}", 
             pkce.code_verifier, code_challenge, pkce.state);
    
    let auth_url = format!(
        "{}/oauth/authorize?client_id=BPr5VekIV-xuQ2ZhmxbGaahJ3XVd7gM83pql-HYGYxQ&redirect_uri={}&response_type=code&scope=profile&state={}&code_challenge={}&code_challenge_method=S256",
        api_config.base_url,
        urlencoding::encode(callback_url),
        urlencoding::encode(&pkce.state),
        urlencoding::encode(&code_challenge)
    );

    if let Err(e) = open::that(&auth_url) {
        return Err(format!("Failed to open authentication URL: {}", e));
    }

    println!("OAuth authentication URL opened in browser. Waiting for callback...");
    Ok(())
}

#[tauri::command]
async fn handle_auth_callback(
    token: String,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<(), String> {
    let mut auth_state = state.lock().await;
    auth_state.is_authenticated = true;
    auth_state.access_token = Some(token);
    auth_state.user_info = Some(HashMap::new());

    Ok(())
}

#[tauri::command]
async fn logout(
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<(), String> {
    let mut auth_state = state.lock().await;
    auth_state.is_authenticated = false;
    auth_state.access_token = None;
    auth_state.user_info = None;

    if let Err(e) = clear_auth_state().await {
        eprintln!("Failed to clear auth state: {}", e);
    }

    Ok(())
}

#[tauri::command]
async fn test_auth_callback(
    token: String,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<(), String> {
    let mut auth_state = state.lock().await;
    auth_state.is_authenticated = true;
    auth_state.access_token = Some(token);
    auth_state.user_info = Some(HashMap::new());

    Ok(())
}

#[tauri::command]
async fn get_api_key(
    api_config: ApiConfig,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<String, String> {
    let auth_state = state.lock().await;

    if !auth_state.is_authenticated {
        return Err("Not authenticated".to_string());
    }

    let access_token = auth_state
        .access_token
        .as_ref()
        .ok_or("No access token available")?;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/api/v1/authenticated/api_key", api_config.base_url))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch API key: {}", e))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API key request failed: {}", error_text));
    }

    let api_key_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API key response: {}", e))?;

    let api_key = api_key_response["api_key"]
        .as_str()
        .ok_or("No API key in response")?;

    Ok(api_key.to_string())
}

#[tauri::command]
async fn authenticate_with_direct_oauth(
    oauth_token: String,
    api_config: ApiConfig,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    if oauth_token.starts_with("hackatime://auth/callback") {
        if let Some(query_start) = oauth_token.find('?') {
            let query = &oauth_token[query_start + 1..];
            let params: Vec<&str> = query.split('&').collect();

            let mut found_code = None;
            let mut found_state = None;
            let mut found_error = None;
            
            for param in params {
                if param.starts_with("code=") {
                    found_code = Some(param[5..].to_string()); 
                } else if param.starts_with("state=") {
                    found_state = Some(param[6..].to_string()); 
                } else if param.starts_with("error=") {
                    found_error = Some(param[6..].to_string());
                }
            }

            if let Some(error) = found_error {
                return Err(format!("OAuth error: {}", error));
            }

            if let Some(code) = found_code {
                println!("Extracted authorization code from deep link: {}", code);
                
                return exchange_authorization_code(code, found_state, api_config, state, client).await;
            } else {
                return Err("No authorization code found in deep link URL".to_string());
            }
        } else {
            return Err("Invalid deep link URL format".to_string());
        }
    } else {
        return validate_access_token(oauth_token, api_config, state, client).await;
    }
}

async fn exchange_authorization_code(
    code: String,
    _state: Option<String>,
    api_config: ApiConfig,
    auth_state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
    client: reqwest::Client,
) -> Result<(), String> {
    println!("Exchanging authorization code for access token");
    
    let response = client
        .post(&format!(
            "{}/oauth/token",
            api_config.base_url
        ))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("client_id", "BPr5VekIV-xuQ2ZhmxbGaahJ3XVd7gM83pql-HYGYxQ"),
            ("redirect_uri", "hackatime://auth/callback"),
        ])
        .send()
        .await
        .map_err(|e| format!("Failed to exchange authorization code: {}", e))?;

    println!("Token exchange response status: {}", response.status());

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        println!("Token exchange failed with error: {}", error_text);
        return Err(format!("Token exchange failed: {}", error_text));
    }

    let token_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    let access_token = token_response["access_token"]
        .as_str()
        .ok_or("No access token in response")?;

    let user_response = client
        .get(&format!("{}/api/v1/authenticated/me", api_config.base_url))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user info: {}", e))?;

    let user_info = if user_response.status().is_success() {
        user_response.json::<serde_json::Value>()
            .await
            .unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let mut user_info_map = HashMap::new();
    if let Some(obj) = user_info.as_object() {
        for (key, value) in obj {
            user_info_map.insert(key.clone(), value.clone());
        }
    }

    let mut auth_state = auth_state.lock().await;
    auth_state.is_authenticated = true;
    auth_state.access_token = Some(access_token.to_string());
    auth_state.user_info = Some(user_info_map);

    let auth_state_to_save = auth_state.clone();
    drop(auth_state); 
    if let Err(e) = save_auth_state(auth_state_to_save).await {
        eprintln!("Failed to save auth state: {}", e);
    }

    println!("Direct OAuth authentication completed successfully!");
    Ok(())
}

async fn validate_access_token(
    access_token: String,
    api_config: ApiConfig,
    auth_state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
    client: reqwest::Client,
) -> Result<(), String> {
    println!("Validating access token directly");
    
    let response = client
        .get(&format!("{}/api/v1/authenticated/me", api_config.base_url))
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to validate access token: {}", e))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Access token validation failed: {}", error_text));
    }

    let user_info = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse user info response: {}", e))?;

    let mut user_info_map = HashMap::new();
    if let Some(obj) = user_info.as_object() {
        for (key, value) in obj {
            user_info_map.insert(key.clone(), value.clone());
        }
    }

    let mut auth_state = auth_state.lock().await;
    auth_state.is_authenticated = true;
    auth_state.access_token = Some(access_token);
    auth_state.user_info = Some(user_info_map);

    let auth_state_to_save = auth_state.clone();
    drop(auth_state); 
    if let Err(e) = save_auth_state(auth_state_to_save).await {
        eprintln!("Failed to save auth state: {}", e);
    }

    println!("Access token validation completed successfully!");
    Ok(())
}

#[tauri::command]
async fn handle_deep_link_callback(
    authorization_code: String,
    state: String,
    api_config: ApiConfig,
    auth_state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
    pkce_state: State<'_, Arc<tauri::async_runtime::Mutex<Option<PkceState>>>>,
) -> Result<(), String> {
    let stored_pkce = {
        let pkce_guard = pkce_state.lock().await;
        pkce_guard.clone()
    };

    let pkce = match stored_pkce {
        Some(pkce) => {
            if pkce.is_expired(600) {
                return Err("PKCE state expired. Please restart authentication.".to_string());
            }
            
            if pkce.state != state {
                return Err("State parameter mismatch. Possible CSRF attack.".to_string());
            }
            
            pkce
        }
        None => return Err("No PKCE state found. Please restart authentication.".to_string()),
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&format!(
            "{}/oauth/token",
            api_config.base_url
        ))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &authorization_code),
            ("client_id", "BPr5VekIV-xuQ2ZhmxbGaahJ3XVd7gM83pql-HYGYxQ"),
            ("redirect_uri", "hackatime://auth/callback"),
            ("code_verifier", &pkce.code_verifier),
        ])
        .send()
        .await
        .map_err(|e| format!("Failed to exchange token: {}", e))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Token exchange failed: {}", error_text));
    }

    let token_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    let access_token = token_response["access_token"]
        .as_str()
        .ok_or("No access token in response")?;

    let user_response = client
        .get(&format!("{}/api/v1/authenticated/me", api_config.base_url))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user info: {}", e))?;

    let user_info = if user_response.status().is_success() {
        user_response.json::<serde_json::Value>()
            .await
            .unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let mut user_info_map = HashMap::new();
    if let Some(obj) = user_info.as_object() {
        for (key, value) in obj {
            user_info_map.insert(key.clone(), value.clone());
        }
    }

    let mut auth_state = auth_state.lock().await;
    auth_state.is_authenticated = true;
    auth_state.access_token = Some(access_token.to_string());
    auth_state.user_info = Some(user_info_map);

    let auth_state_to_save = auth_state.clone();
    drop(auth_state); 
    if let Err(e) = save_auth_state(auth_state_to_save).await {
        eprintln!("Failed to save auth state: {}", e);
    }

    {
        let mut stored_pkce = pkce_state.lock().await;
        *stored_pkce = None;
    }

    println!("OAuth authentication completed successfully!");
    Ok(())
}

#[tauri::command]
async fn setup_hackatime_macos_linux(api_key: String, api_url: String) -> Result<String, String> {
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
async fn setup_hackatime_windows(api_key: String, api_url: String) -> Result<String, String> {
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
async fn test_hackatime_heartbeat(api_key: String, api_url: String) -> Result<String, String> {
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
async fn setup_hackatime_complete(api_key: String, api_url: String) -> Result<String, String> {
    // Detect operating system and use appropriate setup function
    if cfg!(target_os = "windows") {
        setup_hackatime_windows(api_key, api_url).await
    } else {
        setup_hackatime_macos_linux(api_key, api_url).await
    }
}

#[tauri::command]
async fn save_auth_state(auth_state: AuthState) -> Result<(), String> {
    println!(
        "save_auth_state called: authenticated={}, has_token={}",
        auth_state.is_authenticated,
        auth_state.access_token.is_some()
    );
    let db = Database::new().await?;
    println!("Database connection successful for save");

    // Convert to database AuthState format
    let db_auth_state = DbAuthState {
        is_authenticated: auth_state.is_authenticated,
        access_token: auth_state.access_token,
        user_info: auth_state.user_info,
    };

    // Save to database
    let session_id = db.save_session(&db_auth_state).await?;
    println!("Session saved with ID: {}", session_id);

    Ok(())
}

#[tauri::command]
async fn load_auth_state() -> Result<Option<AuthState>, String> {
    println!("load_auth_state called");
    let db = Database::new().await?;
    println!("Database connection successful");

    match db.load_latest_session().await? {
        Some(db_auth_state) => {
            println!(
                "Found saved session: authenticated={}, has_token={}",
                db_auth_state.is_authenticated,
                db_auth_state.access_token.is_some()
            );
            let auth_state = AuthState {
                is_authenticated: db_auth_state.is_authenticated,
                access_token: db_auth_state.access_token,
                user_info: db_auth_state.user_info,
            };
            Ok(Some(auth_state))
        }
        None => {
            println!("No saved sessions found");
            Ok(None)
        }
    }
}

#[tauri::command]
async fn clear_auth_state() -> Result<(), String> {
    let db = Database::new().await?;
    db.clear_sessions().await?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
struct PresenceNotification {
    r#type: String,
    message: Option<String>,
    user_id: u32,
    username: String,
    session_data: SessionData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct HeartbeatData {
    id: u32,
    project: Option<String>,
    editor: Option<String>,
    language: Option<String>,
    entity: Option<String>,
    time: f64,
    timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SessionState {
    is_active: bool,
    start_time: Option<i64>,
    last_heartbeat_id: Option<u32>,
    heartbeat_count: u32,
    project: Option<String>,
    editor: Option<String>,
    language: Option<String>,
    entity: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct HeartbeatResponse {
    heartbeat: Option<HeartbeatData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
struct SessionData {
    project: Option<String>,
    editor: Option<String>,
    language: Option<String>,
    entity: Option<String>,
    start_time: i64,
    last_heartbeat_time: i64,
    heartbeat_count: u32,
}

#[tauri::command]
async fn register_presence_connection(
    api_config: ApiConfig,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<(), String> {
    let auth_state = state.lock().await;

    if !auth_state.is_authenticated {
        return Err("Not authenticated".to_string());
    }

    let access_token = auth_state
        .access_token
        .as_ref()
        .ok_or("No access token available")?;

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/v1/presence/register", api_config.base_url))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to register presence connection: {}", e))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Presence registration failed: {}", error_text));
    }

    Ok(())
}

#[tauri::command]
async fn get_latest_heartbeat(
    api_config: ApiConfig,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
    discord_rpc_state: State<'_, Arc<tauri::async_runtime::Mutex<DiscordRpcService>>>,
    session_state: State<'_, Arc<tauri::async_runtime::Mutex<SessionState>>>,
) -> Result<HeartbeatResponse, String> {
    let auth_state = state.lock().await;

    if !auth_state.is_authenticated {
        return Err("Not authenticated".to_string());
    }

    let access_token = auth_state
        .access_token
        .as_ref()
        .ok_or("No access token available")?;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!(
            "{}/api/v1/presence/latest_heartbeat",
            api_config.base_url
        ))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to get latest heartbeat: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        // Handle rate limiting gracefully
        if status == 429 {
            println!("Rate limited, will retry later");
            return Err(format!("Rate limited: {}", error_text));
        }

        return Err(format!("Failed to get latest heartbeat: {}", error_text));
    }

    let heartbeat_response: HeartbeatResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse heartbeat response: {}", e))?;

    // Process session logic
    if let Some(heartbeat) = &heartbeat_response.heartbeat {
        let mut session = session_state.lock().await;
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Check if heartbeat is less than 2 minutes old
        let heartbeat_age = current_time - heartbeat.timestamp;
        let is_recent = heartbeat_age < 120; // 2 minutes

        // Check for duplicate heartbeat (same ID as last one)
        let is_duplicate = session.last_heartbeat_id == Some(heartbeat.id);

        if is_duplicate {
            // Duplicate heartbeat detected - end the session
            println!("Duplicate heartbeat detected, ending session");
            session.is_active = false;
            session.start_time = None;
            session.last_heartbeat_id = None;
            session.heartbeat_count = 0;
            session.project = None;
            session.editor = None;
            session.language = None;
            session.entity = None;

            // Clear Discord RPC activity
            let mut discord_rpc = discord_rpc_state.lock().await;
            if discord_rpc.is_connected() {
                let _ = discord_rpc.clear_activity();
            }
        } else if is_recent && !session.is_active {
            // Start new session
            println!("Recent heartbeat detected, starting new session");
            session.is_active = true;
            session.start_time = Some(heartbeat.timestamp);
            session.last_heartbeat_id = Some(heartbeat.id);
            session.heartbeat_count = 1;
            session.project = heartbeat.project.clone();
            session.editor = heartbeat.editor.clone();
            session.language = heartbeat.language.clone();
            session.entity = heartbeat.entity.clone();

            // Update Discord RPC with session start time
            let mut discord_rpc = discord_rpc_state.lock().await;
            if discord_rpc.is_connected() {
                if let Err(e) =
                    discord_rpc.update_activity_from_session(heartbeat, heartbeat.timestamp)
                {
                    eprintln!("Failed to update Discord RPC: {}", e);
                }
            }
        } else if is_recent && session.is_active {
            // Continue existing session
            session.last_heartbeat_id = Some(heartbeat.id);
            session.heartbeat_count += 1;
            session.project = heartbeat.project.clone();
            session.editor = heartbeat.editor.clone();
            session.language = heartbeat.language.clone();
            session.entity = heartbeat.entity.clone();

            // Update Discord RPC with session start time
            let mut discord_rpc = discord_rpc_state.lock().await;
            if discord_rpc.is_connected() {
                if let Err(e) = discord_rpc.update_activity_from_session(
                    heartbeat,
                    session.start_time.unwrap_or(heartbeat.timestamp),
                ) {
                    eprintln!("Failed to update Discord RPC: {}", e);
                }
            }
        } else if !is_recent && session.is_active {
            // Heartbeat is too old, end the session
            println!("Heartbeat too old, ending session");
            session.is_active = false;
            session.start_time = None;
            session.last_heartbeat_id = None;
            session.heartbeat_count = 0;
            session.project = None;
            session.editor = None;
            session.language = None;
            session.entity = None;

            // Clear Discord RPC activity
            let mut discord_rpc = discord_rpc_state.lock().await;
            if discord_rpc.is_connected() {
                let _ = discord_rpc.clear_activity();
            }
        }
    } else {
        // No heartbeat data - end session if active
        let mut session = session_state.lock().await;
        if session.is_active {
            println!("No heartbeat data, ending session");
            session.is_active = false;
            session.start_time = None;
            session.last_heartbeat_id = None;
            session.heartbeat_count = 0;
            session.project = None;
            session.editor = None;
            session.language = None;
            session.entity = None;

            // Clear Discord RPC activity
            let mut discord_rpc = discord_rpc_state.lock().await;
            if discord_rpc.is_connected() {
                let _ = discord_rpc.clear_activity();
            }
        }
    }

    Ok(heartbeat_response)
}

#[tauri::command]
async fn ping_presence_connection(
    api_config: ApiConfig,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<(), String> {
    let auth_state = state.lock().await;

    if !auth_state.is_authenticated {
        return Err("Not authenticated".to_string());
    }

    let access_token = auth_state
        .access_token
        .as_ref()
        .ok_or("No access token available")?;

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/v1/presence/ping", api_config.base_url))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to ping presence connection: {}", e))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Presence ping failed: {}", error_text));
    }

    Ok(())
}

#[allow(dead_code)]
fn get_config_dir() -> Result<std::path::PathBuf, String> {
    // Use hackatime directory structure
    get_hackatime_config_dir()
}

#[tauri::command]
async fn get_hackatime_directories() -> Result<serde_json::Value, String> {
    let config_dir = get_hackatime_config_dir()?;
    let logs_dir = get_hackatime_logs_dir()?;
    let data_dir = get_hackatime_data_dir()?;

    Ok(serde_json::json!({
        "config_dir": config_dir.to_string_lossy(),
        "logs_dir": logs_dir.to_string_lossy(),
        "data_dir": data_dir.to_string_lossy()
    }))
}

#[tauri::command]
async fn cleanup_old_sessions(days_old: i64) -> Result<(), String> {
    let db = Database::new().await?;
    db.cleanup_old_sessions(days_old).await?;
    Ok(())
}

#[tauri::command]
async fn get_session_stats() -> Result<serde_json::Value, String> {
    let platform_info = get_platform_info()?;

    Ok(serde_json::json!({
        "platform_info": platform_info,
        "database_path": get_hackatime_config_dir()?.join("sessions.db").to_string_lossy(),
        "directories_created": {
            "config": get_hackatime_config_dir()?.exists(),
            "logs": get_hackatime_logs_dir()?.exists(),
            "data": get_hackatime_data_dir()?.exists()
        }
    }))
}

#[tauri::command]
async fn test_database_connection() -> Result<serde_json::Value, String> {
    // Test directory creation
    let config_dir = get_hackatime_config_dir()?;
    let logs_dir = get_hackatime_logs_dir()?;
    let data_dir = get_hackatime_data_dir()?;

    // Test database connection
    let db_result = Database::new().await;
    let db_success = db_result.is_ok();
    let db_error = if let Err(e) = db_result {
        Some(e)
    } else {
        None
    };

    Ok(serde_json::json!({
        "directories": {
            "config_exists": config_dir.exists(),
            "logs_exists": logs_dir.exists(),
            "data_exists": data_dir.exists(),
            "config_path": config_dir.to_string_lossy(),
            "logs_path": logs_dir.to_string_lossy(),
            "data_path": data_dir.to_string_lossy()
        },
        "database": {
            "connection_success": db_success,
            "error": db_error,
            "db_path": config_dir.join("sessions.db").to_string_lossy()
        }
    }))
}

// Discord RPC Commands
#[tauri::command]
async fn discord_rpc_connect(
    client_id: String,
    state: State<'_, Arc<tauri::async_runtime::Mutex<DiscordRpcService>>>,
) -> Result<(), String> {
    let mut rpc_service = state.lock().await;
    rpc_service.connect(&client_id)
}

#[tauri::command]
async fn discord_rpc_disconnect(
    state: State<'_, Arc<tauri::async_runtime::Mutex<DiscordRpcService>>>,
) -> Result<(), String> {
    let mut rpc_service = state.lock().await;
    rpc_service.disconnect()
}

#[tauri::command]
async fn discord_rpc_set_activity(
    activity: DiscordActivity,
    state: State<'_, Arc<tauri::async_runtime::Mutex<DiscordRpcService>>>,
) -> Result<(), String> {
    let mut rpc_service = state.lock().await;
    rpc_service.set_activity(activity)
}

#[tauri::command]
async fn discord_rpc_clear_activity(
    state: State<'_, Arc<tauri::async_runtime::Mutex<DiscordRpcService>>>,
) -> Result<(), String> {
    let mut rpc_service = state.lock().await;
    rpc_service.clear_activity()
}

#[tauri::command]
async fn discord_rpc_get_state(
    state: State<'_, Arc<tauri::async_runtime::Mutex<DiscordRpcService>>>,
) -> Result<DiscordRpcState, String> {
    let rpc_service = state.lock().await;
    Ok(rpc_service.get_state())
}

#[tauri::command]
async fn discord_rpc_update_from_heartbeat(
    heartbeat_data: HeartbeatData,
    state: State<'_, Arc<tauri::async_runtime::Mutex<DiscordRpcService>>>,
) -> Result<(), String> {
    let mut rpc_service = state.lock().await;
    rpc_service.update_activity_from_heartbeat(&heartbeat_data)
}

#[tauri::command]
async fn discord_rpc_auto_connect(
    client_id: String,
    discord_rpc_state: State<'_, Arc<tauri::async_runtime::Mutex<DiscordRpcService>>>,
) -> Result<(), String> {
    let mut rpc_service = discord_rpc_state.lock().await;
    rpc_service.connect(&client_id)
}

#[tauri::command]
async fn discord_rpc_auto_disconnect(
    discord_rpc_state: State<'_, Arc<tauri::async_runtime::Mutex<DiscordRpcService>>>,
) -> Result<(), String> {
    let mut rpc_service = discord_rpc_state.lock().await;
    rpc_service.disconnect()
}

#[tauri::command]
async fn get_current_session(
    session_state: State<'_, Arc<tauri::async_runtime::Mutex<SessionState>>>,
) -> Result<SessionState, String> {
    let session = session_state.lock().await;
    Ok(session.clone())
}

#[tauri::command]
async fn get_projects(
    api_config: ApiConfig,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<serde_json::Value, String> {
    let auth_state = state.lock().await;

    if !auth_state.is_authenticated {
        return Err("Not authenticated".to_string());
    }

    let access_token = auth_state
        .access_token
        .as_ref()
        .ok_or("No access token available")?;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!(
            "{}/api/v1/authenticated/projects",
            api_config.base_url
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

    Ok(projects_response)
}

#[tauri::command]
async fn get_project_details(
    project_name: String,
    api_config: ApiConfig,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<serde_json::Value, String> {
    let auth_state = state.lock().await;

    if !auth_state.is_authenticated {
        return Err("Not authenticated".to_string());
    }

    let access_token = auth_state
        .access_token
        .as_ref()
        .ok_or("No access token available")?;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!(
            "{}/api/v1/authenticated/projects/{}",
            api_config.base_url,
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

#[tauri::command]
async fn get_discord_rpc_enabled(
    discord_rpc_state: State<'_, Arc<tauri::async_runtime::Mutex<DiscordRpcService>>>,
) -> Result<bool, String> {
    let rpc_service = discord_rpc_state.lock().await;
    Ok(rpc_service.is_connected())
}

#[tauri::command]
async fn set_discord_rpc_enabled(
    enabled: bool,
    discord_rpc_state: State<'_, Arc<tauri::async_runtime::Mutex<DiscordRpcService>>>,
) -> Result<(), String> {
    let mut rpc_service = discord_rpc_state.lock().await;

    if enabled {
        // Try to connect with a default client ID (you might want to make this configurable)
        let default_client_id = "1234567890123456789"; // Replace with your Discord app client ID
        rpc_service.connect(default_client_id)
    } else {
        rpc_service.disconnect()
    }
}

// Statistics and Trends Processing
#[derive(Debug, Serialize, Deserialize, Clone)]
struct StatisticsData {
    trends: Vec<TrendStatistic>,
    charts: Vec<ChartData>,
    insights: Vec<Insight>,
    programmer_class: ProgrammerClass,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TrendStatistic {
    title: String,
    value: String,
    change: String,
    change_type: String, // "increase", "decrease", "neutral"
    period: String,
    icon: String,
    color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChartData {
    id: String,
    title: String,
    chart_type: String, // "line", "bar", "pie", "area", "radar"
    data: serde_json::Value,
    period: String,
    color_scheme: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Insight {
    title: String,
    description: String,
    value: String,
    trend: String,
    icon: String,
    color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProgrammerClass {
    class_name: String,
    description: String,
    technologies: Vec<String>,
    level: String,
    color: String,
}

#[tauri::command]
async fn get_statistics_data(
    api_config: ApiConfig,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<StatisticsData, String> {
    let auth_state = state.lock().await;

    if !auth_state.is_authenticated {
        return Err("Not authenticated".to_string());
    }

    let access_token = auth_state
        .access_token
        .as_ref()
        .ok_or("No access token available")?;

    let client = reqwest::Client::new();

    // Get dashboard stats from Ruby API
    let response = client
        .get(&format!(
            "{}/api/v1/authenticated/dashboard_stats",
            api_config.base_url
        ))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch dashboard stats: {}", e))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Dashboard stats request failed: {}", error_text));
    }

    let dashboard_stats: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse dashboard stats: {}", e))?;

    // Process the data in Rust for heavy computations
    let statistics = process_statistics_data(dashboard_stats).await?;

    Ok(statistics)
}

// Tray-related commands
#[tauri::command]
async fn show_window(app: tauri::AppHandle) -> Result<(), String> {
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
async fn hide_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .hide()
            .map_err(|e| format!("Failed to hide window: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
async fn toggle_window(app: tauri::AppHandle) -> Result<(), String> {
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

#[tauri::command]
async fn get_app_status(
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

async fn process_statistics_data(
    dashboard_stats: serde_json::Value,
) -> Result<StatisticsData, String> {
    // Extract data from dashboard stats
    let current_streak = dashboard_stats["current_streak"].as_u64().unwrap_or(0);
    let weekly_time = dashboard_stats["weekly_stats"]["time_coded_seconds"]
        .as_u64()
        .unwrap_or(0) as f64;
    let all_time_time = dashboard_stats["all_time_stats"]["time_coded_seconds"]
        .as_u64()
        .unwrap_or(0) as f64;

    // Calculate trends (comparing this week to last week)
    let trends = calculate_trends(weekly_time, current_streak).await;

    // Generate chart data
    let charts = generate_chart_data(&dashboard_stats).await?;

    // Generate insights
    let insights = generate_insights(weekly_time, all_time_time, current_streak).await;

    // Analyze programmer class
    let programmer_class = analyze_programmer_class(&dashboard_stats).await;

    Ok(StatisticsData {
        trends,
        charts,
        insights,
        programmer_class,
    })
}

async fn calculate_trends(weekly_time: f64, current_streak: u64) -> Vec<TrendStatistic> {
    let mut trends = Vec::new();

    // Simulate last week's data (in a real app, you'd fetch this from the API)
    let last_week_time = weekly_time * 0.85; // Simulate 15% increase
    let last_week_streak = if current_streak > 0 {
        current_streak - 1
    } else {
        0
    };

    // Weekly coding time trend
    let time_change = ((weekly_time - last_week_time) / last_week_time * 100.0).round() as i32;
    let time_trend = if time_change > 0 {
        TrendStatistic {
            title: "Weekly Coding Time".to_string(),
            value: format!("{:.1}h", weekly_time / 3600.0),
            change: format!("+{}%", time_change),
            change_type: "increase".to_string(),
            period: "vs last week".to_string(),
            icon: "".to_string(),
            color: "#4CAF50".to_string(),
        }
    } else if time_change < 0 {
        TrendStatistic {
            title: "Weekly Coding Time".to_string(),
            value: format!("{:.1}h", weekly_time / 3600.0),
            change: format!("{}%", time_change),
            change_type: "decrease".to_string(),
            period: "vs last week".to_string(),
            icon: "".to_string(),
            color: "#F44336".to_string(),
        }
    } else {
        TrendStatistic {
            title: "Weekly Coding Time".to_string(),
            value: format!("{:.1}h", weekly_time / 3600.0),
            change: "No change".to_string(),
            change_type: "neutral".to_string(),
            period: "vs last week".to_string(),
            icon: "".to_string(),
            color: "#FF9800".to_string(),
        }
    };
    trends.push(time_trend);

    // Streak trend
    let streak_change = current_streak as i32 - last_week_streak as i32;
    let streak_trend = if streak_change > 0 {
        TrendStatistic {
            title: "Coding Streak".to_string(),
            value: format!("{} days", current_streak),
            change: format!("+{} days", streak_change),
            change_type: "increase".to_string(),
            period: "vs last week".to_string(),
            icon: "".to_string(),
            color: "#FF5722".to_string(),
        }
    } else if streak_change < 0 {
        TrendStatistic {
            title: "Coding Streak".to_string(),
            value: format!("{} days", current_streak),
            change: format!("{} days", streak_change),
            change_type: "decrease".to_string(),
            period: "vs last week".to_string(),
            icon: "".to_string(),
            color: "#F44336".to_string(),
        }
    } else {
        TrendStatistic {
            title: "Coding Streak".to_string(),
            value: format!("{} days", current_streak),
            change: "Maintained".to_string(),
            change_type: "neutral".to_string(),
            period: "vs last week".to_string(),
            icon: "".to_string(),
            color: "#FF9800".to_string(),
        }
    };
    trends.push(streak_trend);

    // Focus time trend (replaces productivity)
    let daily_average = weekly_time / 3600.0 / 7.0;
    let last_week_daily = daily_average * 0.9;
    let focus_change = ((daily_average - last_week_daily) / last_week_daily * 100.0).round() as i32;

    let focus_trend = if focus_change > 0 {
        TrendStatistic {
            title: "Daily Focus Time".to_string(),
            value: format!("{:.1}h/day", daily_average),
            change: format!("+{}%", focus_change),
            change_type: "increase".to_string(),
            period: "vs last week".to_string(),
            icon: "".to_string(),
            color: "#4CAF50".to_string(),
        }
    } else if focus_change < 0 {
        TrendStatistic {
            title: "Daily Focus Time".to_string(),
            value: format!("{:.1}h/day", daily_average),
            change: format!("{}%", focus_change),
            change_type: "decrease".to_string(),
            period: "vs last week".to_string(),
            icon: "".to_string(),
            color: "#F44336".to_string(),
        }
    } else {
        TrendStatistic {
            title: "Daily Focus Time".to_string(),
            value: format!("{:.1}h/day", daily_average),
            change: "No change".to_string(),
            change_type: "neutral".to_string(),
            period: "vs last week".to_string(),
            icon: "".to_string(),
            color: "#FF9800".to_string(),
        }
    };
    trends.push(focus_trend);

    trends
}

async fn generate_chart_data(
    dashboard_stats: &serde_json::Value,
) -> Result<Vec<ChartData>, String> {
    let mut charts = Vec::new();

    // Daily hours chart
    if let Some(daily_hours) = dashboard_stats["weekly_stats"]["daily_hours"].as_object() {
        let mut chart_data = Vec::new();
        let mut labels = Vec::new();

        for (_date, day_data) in daily_hours {
            if let Some(hours) = day_data["hours"].as_f64() {
                labels.push(day_data["day_name"].as_str().unwrap_or("").to_string());
                chart_data.push(hours);
            }
        }

        charts.push(ChartData {
            id: "daily_hours".to_string(),
            title: "Daily Coding Hours".to_string(),
            chart_type: "bar".to_string(),
            data: serde_json::json!({
                "labels": labels,
                "datasets": [{
                    "label": "Hours",
                    "data": chart_data,
                    "backgroundColor": "#FB4B20",
                    "borderColor": "#FB4B20",
                    "borderWidth": 1
                }]
            }),
            period: "Last 7 days".to_string(),
            color_scheme: "orange".to_string(),
        });
    }

    // Language distribution pie chart
    if let Some(top_language) = dashboard_stats["weekly_stats"]["top_language"].as_object() {
        let language_name = top_language["name"].as_str().unwrap_or("Unknown");
        let language_seconds = top_language["seconds"].as_u64().unwrap_or(0) as f64;
        let total_seconds = dashboard_stats["weekly_stats"]["time_coded_seconds"]
            .as_u64()
            .unwrap_or(1) as f64;
        let percentage = (language_seconds / total_seconds * 100.0).round() as i32;

        charts.push(ChartData {
            id: "language_distribution".to_string(),
            title: "Top Language".to_string(),
            chart_type: "doughnut".to_string(),
            data: serde_json::json!({
                "labels": [language_name, "Others"],
                "datasets": [{
                    "data": [percentage, 100 - percentage],
                    "backgroundColor": ["#FB4B20", "#E0E0E0"],
                    "borderWidth": 0
                }]
            }),
            period: "This week".to_string(),
            color_scheme: "orange".to_string(),
        });
    }

    // Weekly trend line chart
    let mut trend_data = Vec::new();
    let mut trend_labels = Vec::new();

    // Simulate 4 weeks of data
    for week in 0..4 {
        let week_hours = if week == 3 {
            dashboard_stats["weekly_stats"]["time_coded_seconds"]
                .as_u64()
                .unwrap_or(0) as f64
                / 3600.0
        } else {
            // Simulate previous weeks
            (dashboard_stats["weekly_stats"]["time_coded_seconds"]
                .as_u64()
                .unwrap_or(0) as f64
                / 3600.0)
                * (0.8 + (week as f64 * 0.1))
        };

        trend_data.push(week_hours);
        trend_labels.push(format!("Week {}", 4 - week));
    }

    charts.push(ChartData {
        id: "weekly_trend".to_string(),
        title: "Weekly Trend".to_string(),
        chart_type: "line".to_string(),
        data: serde_json::json!({
            "labels": trend_labels,
            "datasets": [{
                "label": "Hours",
                "data": trend_data,
                "borderColor": "#FB4B20",
                "backgroundColor": "rgba(251, 75, 32, 0.1)",
                "fill": true,
                "tension": 0.4
            }]
        }),
        period: "Last 4 weeks".to_string(),
        color_scheme: "orange".to_string(),
    });

    Ok(charts)
}

async fn generate_insights(
    weekly_time: f64,
    all_time_time: f64,
    current_streak: u64,
) -> Vec<Insight> {
    let mut insights = Vec::new();

    // Coding consistency insight
    let daily_average = weekly_time / 3600.0 / 7.0;
    let consistency_insight = if daily_average >= 2.0 {
        Insight {
            title: "Consistent Coder".to_string(),
            description: "You've been coding consistently every day this week!".to_string(),
            value: format!("{:.1}h/day", daily_average),
            trend: "Great consistency".to_string(),
            icon: "".to_string(),
            color: "#4CAF50".to_string(),
        }
    } else if daily_average >= 1.0 {
        Insight {
            title: "Steady Progress".to_string(),
            description: "You're maintaining a good coding rhythm.".to_string(),
            value: format!("{:.1}h/day", daily_average),
            trend: "Keep it up".to_string(),
            icon: "".to_string(),
            color: "#FF9800".to_string(),
        }
    } else {
        Insight {
            title: "Room for Growth".to_string(),
            description: "Try to code a bit more each day to build momentum.".to_string(),
            value: format!("{:.1}h/day", daily_average),
            trend: "Build momentum".to_string(),
            icon: "".to_string(),
            color: "#2196F3".to_string(),
        }
    };
    insights.push(consistency_insight);

    // Streak insight
    let streak_insight = if current_streak >= 30 {
        Insight {
            title: "Streak Master".to_string(),
            description: "Incredible! You've been coding for over a month straight!".to_string(),
            value: format!("{} days", current_streak),
            trend: "Amazing dedication".to_string(),
            icon: "".to_string(),
            color: "#FFD700".to_string(),
        }
    } else if current_streak >= 7 {
        Insight {
            title: "Week Warrior".to_string(),
            description: "You've been coding for a full week! Great job!".to_string(),
            value: format!("{} days", current_streak),
            trend: "Excellent progress".to_string(),
            icon: "".to_string(),
            color: "#FF5722".to_string(),
        }
    } else if current_streak > 0 {
        Insight {
            title: "Getting Started".to_string(),
            description: "You're building a coding habit! Keep it going!".to_string(),
            value: format!("{} days", current_streak),
            trend: "Building momentum".to_string(),
            icon: "".to_string(),
            color: "#4CAF50".to_string(),
        }
    } else {
        Insight {
            title: "Fresh Start".to_string(),
            description: "Ready to start your coding journey? Let's begin!".to_string(),
            value: "0 days".to_string(),
            trend: "Start today".to_string(),
            icon: "".to_string(),
            color: "#9C27B0".to_string(),
        }
    };
    insights.push(streak_insight);

    // Total time insight
    let total_hours = all_time_time / 3600.0;
    let total_insight = if total_hours >= 1000.0 {
        Insight {
            title: "Coding Veteran".to_string(),
            description: "You've logged over 1000 hours of coding! Incredible dedication!"
                .to_string(),
            value: format!("{:.0}h total", total_hours),
            trend: "Expert level".to_string(),
            icon: "".to_string(),
            color: "#FFD700".to_string(),
        }
    } else if total_hours >= 100.0 {
        Insight {
            title: "Experienced Coder".to_string(),
            description: "You've put in serious time coding! Keep up the great work!".to_string(),
            value: format!("{:.0}h total", total_hours),
            trend: "Strong foundation".to_string(),
            icon: "".to_string(),
            color: "#4CAF50".to_string(),
        }
    } else if total_hours >= 10.0 {
        Insight {
            title: "Learning Journey".to_string(),
            description: "You're building your coding skills! Every hour counts.".to_string(),
            value: format!("{:.0}h total", total_hours),
            trend: "Growing skills".to_string(),
            icon: "📚".to_string(),
            color: "#2196F3".to_string(),
        }
    } else {
        Insight {
            title: "Just Getting Started".to_string(),
            description: "Every expert was once a beginner. Keep coding!".to_string(),
            value: format!("{:.0}h total", total_hours),
            trend: "Beginning journey".to_string(),
            icon: "🌱".to_string(),
            color: "#9C27B0".to_string(),
        }
    };
    insights.push(total_insight);

    insights
}

async fn analyze_programmer_class(dashboard_stats: &serde_json::Value) -> ProgrammerClass {
    // Load programmer classes configuration
    let config_path = std::env::current_dir()
        .unwrap_or_default()
        .join("programmer_classes.json");

    let config_content = match std::fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(_) => {
            // Fallback to default class if config file is not found
            return ProgrammerClass {
                class_name: "Code Explorer".to_string(),
                description: "An enthusiastic learner discovering the vast world of programming."
                    .to_string(),
                technologies: vec![
                    "HTML".to_string(),
                    "CSS".to_string(),
                    "JavaScript".to_string(),
                ],
                level: "Learning".to_string(),
                color: "#9C27B0".to_string(),
            };
        }
    };

    let config: serde_json::Value = match serde_json::from_str(&config_content) {
        Ok(config) => config,
        Err(_) => {
            // Fallback to default class if config is invalid
            return ProgrammerClass {
                class_name: "Code Explorer".to_string(),
                description: "An enthusiastic learner discovering the vast world of programming."
                    .to_string(),
                technologies: vec![
                    "HTML".to_string(),
                    "CSS".to_string(),
                    "JavaScript".to_string(),
                ],
                level: "Learning".to_string(),
                color: "#9C27B0".to_string(),
            };
        }
    };

    let total_hours = dashboard_stats["all_time_stats"]["time_coded_seconds"]
        .as_u64()
        .unwrap_or(0) as f64
        / 3600.0;

    let current_streak = dashboard_stats["current_streak"].as_u64().unwrap_or(0);

    // Simulate language analysis - in a real app, you'd analyze actual language data from the API
    let simulated_languages = simulate_language_analysis(total_hours, current_streak);

    // Find the best matching class
    let empty_vec = vec![];
    let classes = config["classes"].as_array().unwrap_or(&empty_vec);
    let mut best_match: Option<&serde_json::Value> = None;
    let mut best_score = 0.0;

    for class in classes {
        if let Some(conditions) = class["conditions"].as_object() {
            let score = calculate_class_score(
                &conditions,
                &simulated_languages,
                total_hours,
                current_streak,
            );
            if score > best_score {
                best_score = score;
                best_match = Some(class);
            }
        }
    }

    // Return the best match or default
    if let Some(class) = best_match {
        ProgrammerClass {
            class_name: class["name"].as_str().unwrap_or("Unknown").to_string(),
            description: class["description"].as_str().unwrap_or("").to_string(),
            technologies: class["technologies"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|t| t.as_str())
                .map(|s| s.to_string())
                .collect(),
            level: class["level"].as_str().unwrap_or("Unknown").to_string(),
            color: class["color"].as_str().unwrap_or("#9C27B0").to_string(),
        }
    } else {
        // Default fallback
        ProgrammerClass {
            class_name: "Code Explorer".to_string(),
            description: "An enthusiastic learner discovering the vast world of programming."
                .to_string(),
            technologies: vec![
                "HTML".to_string(),
                "CSS".to_string(),
                "JavaScript".to_string(),
            ],
            level: "Learning".to_string(),
            color: "#9C27B0".to_string(),
        }
    }
}

fn simulate_language_analysis(total_hours: f64, current_streak: u64) -> Vec<String> {
    // Simulate language usage based on coding patterns
    // In a real app, this would come from actual language data from the API
    let mut languages = Vec::new();

    // Simulate language distribution based on experience level
    if total_hours >= 100.0 {
        // Experienced developers
        languages.push("JavaScript".to_string());
        languages.push("Python".to_string());
        languages.push("Java".to_string());
        if current_streak >= 7 {
            languages.push("Rust".to_string());
            languages.push("Go".to_string());
        }
    } else if total_hours >= 20.0 {
        // Intermediate developers
        languages.push("JavaScript".to_string());
        languages.push("Python".to_string());
        if current_streak >= 5 {
            languages.push("TypeScript".to_string());
        }
    } else {
        // Beginners
        languages.push("HTML".to_string());
        languages.push("CSS".to_string());
        languages.push("JavaScript".to_string());
    }

    languages
}

fn calculate_class_score(
    conditions: &serde_json::Map<String, serde_json::Value>,
    languages: &[String],
    total_hours: f64,
    current_streak: u64,
) -> f64 {
    let mut score = 0.0;

    // Check primary languages match
    if let Some(primary_langs) = conditions
        .get("primary_languages")
        .and_then(|v| v.as_array())
    {
        let primary_lang_count = primary_langs
            .iter()
            .filter_map(|lang| lang.as_str())
            .filter(|lang| languages.contains(&lang.to_string()))
            .count();
        score += primary_lang_count as f64 * 2.0; // Weight primary languages heavily
    }

    // Check language count for polyglot
    if let Some(lang_count) = conditions.get("language_count").and_then(|v| v.as_u64()) {
        if languages.len() as u64 >= lang_count {
            score += 3.0; // Bonus for being a polyglot
        }
    }

    // Check minimum hours
    if let Some(min_hours) = conditions.get("min_hours").and_then(|v| v.as_f64()) {
        if total_hours >= min_hours {
            score += 1.0;
        } else {
            score -= 0.5; // Penalty for not meeting minimum
        }
    }

    // Check maximum hours for beginners
    if let Some(max_hours) = conditions.get("max_hours").and_then(|v| v.as_f64()) {
        if total_hours <= max_hours {
            score += 1.0;
        } else {
            score -= 0.5; // Penalty for being too experienced
        }
    }

    // Check minimum streak
    if let Some(min_streak) = conditions.get("min_streak").and_then(|v| v.as_u64()) {
        if current_streak >= min_streak {
            score += 0.5;
        }
    }

    score
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .manage(ApiConfig::new())
        .manage(Arc::new(tauri::async_runtime::Mutex::new(AuthState {
            is_authenticated: false,
            access_token: None,
            user_info: None,
        })))
        .manage(Arc::new(tauri::async_runtime::Mutex::new(Option::<PkceState>::None)))
        .manage(Arc::new(tauri::async_runtime::Mutex::new(DiscordRpcService::new())))
        .manage(Arc::new(tauri::async_runtime::Mutex::new(SessionState {
            is_active: false,
            start_time: None,
            last_heartbeat_id: None,
            heartbeat_count: 0,
            project: None,
            editor: None,
            language: None,
            entity: None,
        })))
        .invoke_handler(tauri::generate_handler![
            greet,
            get_api_config,
            set_api_config,
            get_auth_state,
            authenticate_with_rails,
            handle_auth_callback,
            handle_deep_link_callback,
            logout,
            test_auth_callback,
            get_api_key,
            authenticate_with_direct_oauth,
            setup_hackatime_macos_linux,
            setup_hackatime_windows,
            test_hackatime_heartbeat,
            setup_hackatime_complete,
            save_auth_state,
            load_auth_state,
            clear_auth_state,
            register_presence_connection,
            get_latest_heartbeat,
            ping_presence_connection,
            get_hackatime_directories,
            cleanup_old_sessions,
            get_session_stats,
            test_database_connection,
            discord_rpc_connect,
            discord_rpc_disconnect,
            discord_rpc_set_activity,
            discord_rpc_clear_activity,
            discord_rpc_get_state,
            discord_rpc_update_from_heartbeat,
            discord_rpc_auto_connect,
            discord_rpc_auto_disconnect,
            get_current_session,
            get_projects,
            get_project_details,
            get_discord_rpc_enabled,
            set_discord_rpc_enabled,
            get_statistics_data,
            show_window,
            hide_window,
            toggle_window,
            get_app_status
        ])
        .setup(|app| {
            // Create system tray menu items
            let show_item = MenuItem::with_id(app, "show", "Show Hackatime", true, None::<&str>)?;
            let status_item = MenuItem::with_id(app, "status", "📊 Session Status", true, None::<&str>)?;
            let discord_item = MenuItem::with_id(app, "discord", "🎮 Discord RPC", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "⚙️ Settings", true, None::<&str>)?;
            let about_item = MenuItem::with_id(app, "about", "ℹ️ About", true, None::<&str>)?;
            let help_item = MenuItem::with_id(app, "help", "📖 Help", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "❌ Quit", true, None::<&str>)?;
            
            // Create menu with items
            let menu = Menu::with_items(app, &[
                &show_item,
                &tauri::menu::PredefinedMenuItem::separator(app)?,
                &status_item,
                &discord_item,
                &settings_item,
                &tauri::menu::PredefinedMenuItem::separator(app)?,
                &about_item,
                &help_item,
                &tauri::menu::PredefinedMenuItem::separator(app)?,
                &quit_item,
            ])?;
            
            // Create tray icon with menu and event handlers
            let _tray_icon = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "status" => {
                            println!("📊 Session Status:");
                            println!("   - Authentication: Checking...");
                            println!("   - Active Session: Checking...");
                            println!("   - Discord RPC: Checking...");
                            println!("   - Last Heartbeat: Checking...");
                        }
                        "discord" => {
                            println!("🎮 Discord RPC Status:");
                            println!("   - Connection: Checking...");
                            println!("   - Activity: Checking...");
                        }
                        "settings" => {
                            println!("⚙️ Opening Settings...");
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "about" => {
                            println!("ℹ️ Hackatime Desktop v0.1.0");
                            println!("   - Coding Time Tracker");
                            println!("   - Discord RPC Integration");
                            println!("   - Cross-platform Support");
                        }
                        "help" => {
                            println!("📖 Help & Documentation:");
                            println!("   - Left-click tray icon to toggle window");
                            println!("   - Right-click for menu options");
                            println!("   - Window closes to tray (not taskbar)");
                        }
                        "quit" => {
                            println!("❌ Quitting Hackatime Desktop...");
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
                            println!("🪟 Left click on tray icon");
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    println!("🪟 Hiding window to tray");
                                    let _ = window.hide();
                                } else {
                                    println!("🪟 Showing window from tray");
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                        TrayIconEvent::DoubleClick {
                            button: MouseButton::Left,
                            ..
                        } => {
                            println!("🪟 Double-click: showing window");
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {
                            println!("🖱️ Other tray event: {:?}", event);
                        }
                    }
                })
                .build(app)?;

            // Load saved auth state on startup synchronously
            let auth_state = app.state::<Arc<tauri::async_runtime::Mutex<AuthState>>>();
            let auth_state_clone = auth_state.inner().clone();
            
            // Load auth state immediately on startup
            tauri::async_runtime::block_on(async {
                match load_auth_state().await {
                    Ok(Some(saved_auth_state)) => {
                        let mut current_auth_state = auth_state_clone.lock().await;
                        *current_auth_state = saved_auth_state;
                        println!("Loaded saved authentication state on startup");
                    }
                    Ok(None) => {
                        println!("No saved authentication state found");
                    }
                    Err(e) => {
                        println!("Failed to load saved authentication state: {}", e);
                    }
                }
            });

            // Auto-connect Discord RPC on startup
            let discord_rpc_state = app.state::<Arc<tauri::async_runtime::Mutex<DiscordRpcService>>>();
            let discord_rpc_clone = discord_rpc_state.inner().clone();
            
            tauri::async_runtime::spawn(async move {
                let mut rpc_service = discord_rpc_clone.lock().await;
                match rpc_service.auto_connect() {
                    Ok(_) => println!("Discord RPC auto-connected on startup"),
                    Err(e) => println!("Discord RPC auto-connect failed (this is optional): {}", e),
                }
            });
            
            // Register deep link scheme for development
            #[cfg(any(target_os = "linux", all(debug_assertions, target_os = "windows")))]
            {
                app.deep_link().register_all().unwrap_or_else(|e| {
                    eprintln!("Failed to register deep links: {}", e);
                });
            }

            // Handle deep links when app is already running
            let app_handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                let urls = event.urls();
                println!("Deep link received: {:?}", urls);
                
                for url in urls {
                    let url_string = url.to_string();
                    if url_string.starts_with("hackatime://auth/callback") {
                        if let Some(query_start) = url_string.find('?') {
                            let query = &url_string[query_start + 1..];
                            let params: Vec<&str> = query.split('&').collect();
                            
                            let mut code = None;
                            let mut state = None;
                            let mut error = None;
                            
                            for param in params {
                                if param.starts_with("code=") {
                                    code = Some(param[5..].to_string());
                                } else if param.starts_with("state=") {
                                    state = Some(param[6..].to_string());
                                } else if param.starts_with("error=") {
                                    error = Some(param[6..].to_string());
                                }
                            }
                            
                            if let Some(error) = error {
                                println!("OAuth error: {}", error);
                                continue;
                            }
                            
                            if let Some(code) = code {
                                if let Some(state) = state {
                                    println!("Extracted authorization code: {} and state: {}", code, state);
                                    
                                    let api_config = app_handle.state::<ApiConfig>();
                                    let auth_state = app_handle.state::<Arc<tauri::async_runtime::Mutex<AuthState>>>();
                                    let pkce_state = app_handle.state::<Arc<tauri::async_runtime::Mutex<Option<PkceState>>>>();
                                    
                                    let code_clone = code.clone();
                                    let state_clone = state.clone();
                                    let api_config_clone = api_config.inner().clone();
                                    let auth_state_clone = auth_state.inner().clone();
                                    let pkce_state_clone = pkce_state.inner().clone();
                                    
                                    tauri::async_runtime::spawn(async move {
                                        let client = reqwest::Client::new();
                                        
                                        let stored_pkce = {
                                            let pkce_guard = pkce_state_clone.lock().await;
                                            pkce_guard.clone()
                                        };

                                        let pkce = match stored_pkce {
                                            Some(pkce) => {
                                                if pkce.is_expired(600) {
                                                    eprintln!("PKCE state expired. Please restart authentication.");
                                                    return;
                                                }
                                                
                                                if pkce.state != state_clone {
                                                    eprintln!("State parameter mismatch. Possible CSRF attack.");
                                                    return;
                                                }
                                                
                                                pkce
                                            }
                                            None => {
                                                eprintln!("No PKCE state found. Please restart authentication.");
                                                return;
                                            }
                                        };

                                        let response = client
                                            .post(&format!("{}/oauth/token", api_config_clone.base_url))
                                            .form(&[
                                                ("grant_type", "authorization_code"),
                                                ("code", &code_clone),
                                                ("client_id", "BPr5VekIV-xuQ2ZhmxbGaahJ3XVd7gM83pql-HYGYxQ"),
                                                ("redirect_uri", "hackatime://auth/callback"),
                                                ("code_verifier", &pkce.code_verifier),
                                            ])
                                            .send()
                                            .await;

                                        match response {
                                            Ok(resp) => {
                                                if resp.status().is_success() {
                                                    if let Ok(token_response) = resp.json::<serde_json::Value>().await {
                                                        if let Some(access_token) = token_response["access_token"].as_str() {
                                                            let user_response = client
                                                                .get(&format!("{}/api/v1/authenticated/me", api_config_clone.base_url))
                                                                .bearer_auth(access_token)
                                                                .send()
                                                                .await;

                                                            let user_info = match user_response {
                                                                Ok(resp) if resp.status().is_success() => {
                                                                    resp.json::<serde_json::Value>().await.unwrap_or_else(|_| serde_json::json!({}))
                                                                }
                                                                _ => serde_json::json!({})
                                                            };

                                                            let mut user_info_map = HashMap::new();
                                                            if let Some(obj) = user_info.as_object() {
                                                                for (key, value) in obj {
                                                                    user_info_map.insert(key.clone(), value.clone());
                                                                }
                                                            }

                                                            let mut auth_state = auth_state_clone.lock().await;
                                                            auth_state.is_authenticated = true;
                                                            auth_state.access_token = Some(access_token.to_string());
                                                            auth_state.user_info = Some(user_info_map);

                                                            let auth_state_to_save = auth_state.clone();
                                                            drop(auth_state); // Release the lock before the async call
                                                            if let Err(e) = save_auth_state(auth_state_to_save).await {
                                                                eprintln!("Failed to save auth state: {}", e);
                                                            }

                                                            {
                                                                let mut stored_pkce = pkce_state_clone.lock().await;
                                                                *stored_pkce = None;
                                                            }

                                                            println!("OAuth authentication successful!");
                                                        }
                                                    }
                                                } else {
                                                    eprintln!("Token exchange failed with status: {}", resp.status());
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to exchange token: {}", e);
                                            }
                                        }
                                    });
                                } else {
                                    println!("No state parameter found in OAuth callback");
                                }
                            }
                        }
                    }
                }
            });

            if let Some(start_urls) = app.deep_link().get_current().unwrap_or_default() {
                println!("App started with deep link: {:?}", start_urls);
                for url in start_urls {
                    let url_string = url.to_string();
                    if url_string.starts_with("hackatime://auth/callback") {
                        if let Some(query_start) = url_string.find('?') {
                            let query = &url_string[query_start + 1..];
                            let params: Vec<&str> = query.split('&').collect();
                            
                            let mut code = None;
                            let mut state = None;
                            let mut error = None;
                            
                            for param in params {
                                if param.starts_with("code=") {
                                    code = Some(param[5..].to_string());
                                } else if param.starts_with("state=") {
                                    state = Some(param[6..].to_string());
                                } else if param.starts_with("error=") {
                                    error = Some(param[6..].to_string());
                                }
                            }
                            
                            if let Some(error) = error {
                                println!("OAuth error on startup: {}", error);
                                continue;
                            }
                            
                            if let Some(code) = code {
                                if let Some(state) = state {
                                    println!("Startup deep link authorization code: {} and state: {}", code, state);
                                    
                                    let api_config = app.state::<ApiConfig>();
                                    let auth_state = app.state::<Arc<tauri::async_runtime::Mutex<AuthState>>>();
                                    let pkce_state = app.state::<Arc<tauri::async_runtime::Mutex<Option<PkceState>>>>();
                                    
                                    let code_clone = code.clone();
                                    let state_clone = state.clone();
                                    let api_config_clone = api_config.inner().clone();
                                    let auth_state_clone = auth_state.inner().clone();
                                    let pkce_state_clone = pkce_state.inner().clone();
                                    
                                    tauri::async_runtime::spawn(async move {
                                        let client = reqwest::Client::new();
                                        
                                        let stored_pkce = {
                                            let pkce_guard = pkce_state_clone.lock().await;
                                            pkce_guard.clone()
                                        };

                                        let pkce = match stored_pkce {
                                            Some(pkce) => {
                                                if pkce.is_expired(600) {
                                                    eprintln!("PKCE state expired. Please restart authentication.");
                                                    return;
                                                }
                                                
                                                if pkce.state != state_clone {
                                                    eprintln!("State parameter mismatch. Possible CSRF attack.");
                                                    return;
                                                }
                                                
                                                pkce
                                            }
                                            None => {
                                                eprintln!("No PKCE state found. Please restart authentication.");
                                                return;
                                            }
                                        };

                                        let response = client
                                            .post(&format!("{}/oauth/token", api_config_clone.base_url))
                                            .form(&[
                                                ("grant_type", "authorization_code"),
                                                ("code", &code_clone),
                                                ("client_id", "BPr5VekIV-xuQ2ZhmxbGaahJ3XVd7gM83pql-HYGYxQ"),
                                                ("redirect_uri", "hackatime://auth/callback"),
                                                ("code_verifier", &pkce.code_verifier),
                                            ])
                                            .send()
                                            .await;

                                        match response {
                                            Ok(resp) => {
                                                if resp.status().is_success() {
                                                    if let Ok(token_response) = resp.json::<serde_json::Value>().await {
                                                        if let Some(access_token) = token_response["access_token"].as_str() {
                                                            let user_response = client
                                                                .get(&format!("{}/api/v1/authenticated/me", api_config_clone.base_url))
                                                                .bearer_auth(access_token)
                                                                .send()
                                                                .await;

                                                            let user_info = match user_response {
                                                                Ok(resp) if resp.status().is_success() => {
                                                                    resp.json::<serde_json::Value>().await.unwrap_or_else(|_| serde_json::json!({}))
                                                                }
                                                                _ => serde_json::json!({})
                                                            };

                                                            let mut user_info_map = HashMap::new();
                                                            if let Some(obj) = user_info.as_object() {
                                                                for (key, value) in obj {
                                                                    user_info_map.insert(key.clone(), value.clone());
                                                                }
                                                            }

                                                            let mut auth_state = auth_state_clone.lock().await;
                                                            auth_state.is_authenticated = true;
                                                            auth_state.access_token = Some(access_token.to_string());
                                                            auth_state.user_info = Some(user_info_map);

                                                            let auth_state_to_save = auth_state.clone();
                                                            drop(auth_state);
                                                            if let Err(e) = save_auth_state(auth_state_to_save).await {
                                                                eprintln!("Failed to save auth state: {}", e);
                                                            }

                                                            {
                                                                let mut stored_pkce = pkce_state_clone.lock().await;
                                                                *stored_pkce = None;
                                                            }

                                                            println!("Startup OAuth authentication successful!");
                                                        }
                                                    }
                                                } else {
                                                    eprintln!("Startup token exchange failed with status: {}", resp.status());
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to handle startup auth callback: {}", e);
                                            }
                                        }
                                    });
                                } else {
                                    println!("No state parameter found in startup OAuth callback");
                                }
                            }
                        }
                    }
                }
            }
            
            // Handle window close events to hide to tray instead of closing
            if let Some(window) = app.get_webview_window("main") {
                let window_handle = window.clone();
                let _ = window.on_window_event(move |event| {
                    match event {
                        WindowEvent::CloseRequested { api, .. } => {
                            println!("🪟 Window close requested - hiding to tray");
                            api.prevent_close();
                            let _ = window_handle.hide();
                            println!("✅ Window hidden to tray");
                        }
                        _ => {}
                    }
                });
            }
            
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
