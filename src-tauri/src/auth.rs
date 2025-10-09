use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use std::sync::Arc;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;

use crate::database::{AuthState as DbAuthState, Database};
use crate::push_log;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub access_token: Option<String>,
    pub user_info: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PkceState {
    pub code_verifier: String,
    pub state: String,
    pub timestamp: i64,
}

impl PkceState {
    pub fn new() -> Self {
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
    
    pub fn is_expired(&self, max_age_seconds: i64) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        current_time - self.timestamp > max_age_seconds
    }
}

fn generate_code_verifier() -> String {
    let mut rng = rand::rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.random()).collect();
    general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
}

fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    general_purpose::URL_SAFE_NO_PAD.encode(&hash)
}

fn generate_state() -> String {
    let mut rng = rand::rng();
    (0..32)
        .map(|_| {
            const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
            let idx = (rng.random::<u8>() as usize) % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect()
}

#[tauri::command]
pub async fn get_auth_state(
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<AuthState, String> {
    let auth_state = state.lock().await;
    Ok(auth_state.clone())
}

#[tauri::command]
pub async fn authenticate_with_rails(
    api_config: crate::config::ApiConfig,
    pkce_state: State<'_, Arc<tauri::async_runtime::Mutex<Option<PkceState>>>>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {

    let callback_url = "hackatime://auth/callback";
    
    let pkce = PkceState::new();
    let code_challenge = generate_code_challenge(&pkce.code_verifier);
    
    {
        let mut stored_pkce = pkce_state.lock().await;
        *stored_pkce = Some(pkce.clone());
    }
    
    push_log("debug", "backend", format!("Generated PKCE parameters - verifier: {}, challenge: {}, state: {}", 
             pkce.code_verifier, code_challenge, pkce.state));
    
    let auth_url = format!(
        "{}/oauth/authorize?client_id=BPr5VekIV-xuQ2ZhmxbGaahJ3XVd7gM83pql-HYGYxQ&redirect_uri={}&response_type=code&scope=profile&state={}&code_challenge={}&code_challenge_method=S256",
        api_config.base_url,
        urlencoding::encode(callback_url),
        urlencoding::encode(&pkce.state),
        urlencoding::encode(&code_challenge)
    );

    // Use Tauri's opener plugin for better cross-platform support
    use tauri_plugin_opener::OpenerExt;
    if let Err(e) = app_handle.opener().open_url(&auth_url, None::<&str>) {
        push_log("error", "backend", format!("Failed to open authentication URL: {}", e));
        // Return the URL so the frontend can show a fallback button
        return Ok(auth_url);
    }

    push_log("info", "backend", "OAuth authentication URL opened in browser. Waiting for callback...".to_string());
    Ok(auth_url)
}

#[tauri::command]
pub async fn handle_auth_callback(
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
pub async fn logout(
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<(), String> {
    let mut auth_state = state.lock().await;
    auth_state.is_authenticated = false;
    auth_state.access_token = None;
    auth_state.user_info = None;

    if let Err(e) = clear_auth_state().await {
        push_log("error", "backend", format!("Failed to clear auth state: {}", e));
    }

    
    push_log("info", "backend", "Clearing statistics cache on logout...".to_string());
    if let Ok(db) = Database::new().await {
        if let Err(e) = db.clear_all_cache().await {
            push_log("error", "backend", format!("Failed to clear statistics cache on logout: {}", e));
        } else {
            push_log("info", "backend", "Statistics cache cleared on logout".to_string());
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn test_auth_callback(
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
pub async fn get_api_key(
    api_config: crate::config::ApiConfig,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<String, String> {
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
        .get(&format!("{}/api/v1/authenticated/api_keys", base_url))
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

    let api_key = api_key_response["token"]
        .as_str()
        .ok_or("No token in response")?;

    Ok(api_key.to_string())
}

#[tauri::command]
pub async fn authenticate_with_direct_oauth(
    oauth_token: String,
    api_config: crate::config::ApiConfig,
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
                push_log("debug", "backend", format!("Extracted authorization code from deep link: {}", code));
                
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
    api_config: crate::config::ApiConfig,
    auth_state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
    client: reqwest::Client,
) -> Result<(), String> {
    push_log("info", "backend", "Exchanging authorization code for access token".to_string());
    
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

    push_log("debug", "backend", format!("Token exchange response status: {}", response.status()));

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        push_log("error", "backend", format!("Token exchange failed with error: {}", error_text));
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
        push_log("error", "backend", format!("Failed to save auth state: {}", e));
    }

    push_log("info", "backend", "Direct OAuth authentication completed successfully!".to_string());
    Ok(())
}

async fn validate_access_token(
    access_token: String,
    api_config: crate::config::ApiConfig,
    auth_state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
    client: reqwest::Client,
) -> Result<(), String> {
    push_log("info", "backend", "Validating access token directly".to_string());
    
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
        push_log("error", "backend", format!("Failed to save auth state: {}", e));
    }

    push_log("info", "backend", "Access token validation completed successfully!".to_string());
    Ok(())
}

#[tauri::command]
pub async fn handle_deep_link_callback(
    authorization_code: String,
    state: String,
    api_config: crate::config::ApiConfig,
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
        push_log("error", "backend", format!("Failed to save auth state: {}", e));
    }

    {
        let mut stored_pkce = pkce_state.lock().await;
        *stored_pkce = None;
    }

    push_log("info", "backend", "OAuth authentication completed successfully!".to_string());
    Ok(())
}

#[tauri::command]
pub async fn save_auth_state(auth_state: AuthState) -> Result<(), String> {
    push_log("debug", "backend", format!(
        "save_auth_state called: authenticated={}, has_token={}",
        auth_state.is_authenticated,
        auth_state.access_token.is_some()
    ));
    let db = Database::new().await?;
    push_log("debug", "backend", "Database connection successful for save".to_string());

    
    let db_auth_state = DbAuthState {
        is_authenticated: auth_state.is_authenticated,
        access_token: auth_state.access_token,
        user_info: auth_state.user_info,
    };

    
    let session_id = db.save_session(&db_auth_state).await?;
    push_log("info", "backend", format!("Session saved with ID: {}", session_id));

    Ok(())
}

#[tauri::command]
pub async fn load_auth_state() -> Result<Option<AuthState>, String> {
    push_log("debug", "backend", "load_auth_state called".to_string());
    let db = Database::new().await?;
    push_log("debug", "backend", "Database connection successful".to_string());

    match db.load_latest_session().await? {
        Some(db_auth_state) => {
            push_log("debug", "backend", format!(
                "Found saved session: authenticated={}, has_token={}",
                db_auth_state.is_authenticated,
                db_auth_state.access_token.is_some()
            ));
            let auth_state = AuthState {
                is_authenticated: db_auth_state.is_authenticated,
                access_token: db_auth_state.access_token,
                user_info: db_auth_state.user_info,
            };
            Ok(Some(auth_state))
        }
        None => {
            push_log("debug", "backend", "No saved sessions found".to_string());
            Ok(None)
        }
    }
}

#[tauri::command]
pub async fn clear_auth_state() -> Result<(), String> {
    let db = Database::new().await?;
    db.clear_sessions().await?;
    Ok(())
}

