use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Manager, WindowEvent, TitleBarStyle};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tauri_plugin_deep_link::DeepLinkExt;


mod auth;
mod config;
mod database;
mod db_commands;
mod discord_rpc;
mod projects;
mod session;
mod setup;
mod statistics;
mod tray;
mod menu;
mod window;


pub use auth::{AuthState, PkceState};
pub use config::ApiConfig;
pub use discord_rpc::{DiscordRpcService};
pub use session::SessionState;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Clone, serde::Serialize)]
struct LogEntry {
    ts: i64,
    level: String,
    source: String,
    message: String,
}

static LOG_BUFFER: Lazy<Mutex<Vec<LogEntry>>> = Lazy::new(|| Mutex::new(Vec::with_capacity(1024)));

pub fn push_log(level: &str, source: &str, message: String) {
    let mut buf = LOG_BUFFER.lock().unwrap();
    if buf.len() >= 1000 {
        buf.remove(0);
    }
    buf.push(LogEntry {
        ts: chrono::Utc::now().timestamp_millis(),
        level: level.to_string(),
        source: source.to_string(),
        message,
    });
}

#[tauri::command]
fn get_recent_logs() -> Vec<LogEntry> {
    LOG_BUFFER.lock().unwrap().clone()
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
            get_recent_logs,
            
            database::get_platform_info,
            
            config::get_api_config,
            config::set_api_config,
            
            auth::get_auth_state,
            auth::authenticate_with_rails,
            auth::handle_auth_callback,
            auth::handle_deep_link_callback,
            auth::logout,
            auth::test_auth_callback,
            auth::authenticate_with_direct_oauth,
            auth::get_api_key,
            auth::save_auth_state,
            auth::load_auth_state,
            auth::clear_auth_state,
            
            setup::setup_hackatime_macos_linux,
            setup::setup_hackatime_windows,
            setup::test_hackatime_heartbeat,
            setup::setup_hackatime_complete,
            setup::check_wakatime_config,
            setup::apply_wakatime_config,
            
            session::get_latest_heartbeat,
            session::get_current_session,
            session::get_app_status,
            
            db_commands::get_hackatime_directories,
            db_commands::cleanup_old_sessions,
            db_commands::get_session_stats,
            db_commands::test_database_connection,
            db_commands::clear_statistics_cache,
            
            discord_rpc::discord_rpc_connect,
            discord_rpc::discord_rpc_disconnect,
            discord_rpc::discord_rpc_set_activity,
            discord_rpc::discord_rpc_clear_activity,
            discord_rpc::discord_rpc_get_state,
            discord_rpc::discord_rpc_update_from_heartbeat,
            discord_rpc::discord_rpc_auto_connect,
            discord_rpc::discord_rpc_auto_disconnect,
            discord_rpc::get_discord_rpc_enabled,
            discord_rpc::set_discord_rpc_enabled,
            
            projects::get_projects,
            projects::get_project_details,
            
            statistics::get_statistics_data,
            statistics::get_dashboard_stats,
            
            window::show_window,
            window::hide_window,
            window::toggle_window,
        ])
        .setup(|app| {
            push_log("info", "backend", "backend starting".to_string());
            
            if let Some(window) = app.get_webview_window("main") {
                
                #[cfg(target_os = "macos")]
                {
                    window.set_title_bar_style(TitleBarStyle::Transparent).unwrap();
                    
                    
                    #[allow(deprecated)]
                    #[allow(unexpected_cfgs)]
                    {
                        use cocoa::appkit::{NSColor, NSWindow};
                        use cocoa::base::{id, nil};

                        let ns_window = window.ns_window().unwrap() as id;
                        unsafe {
                            use objc::{msg_send, sel, sel_impl};
                            
                            
                        let bg_color = NSColor::clearColor(nil);
                        ns_window.setBackgroundColor_(bg_color);
                        
                        ns_window.setOpaque_(0);
                            
                            let content_view: id = msg_send![ns_window, contentView];
                            let _: () = msg_send![content_view, setWantsLayer: true];
                            
                            let layer: id = msg_send![content_view, layer];
                            let _: () = msg_send![layer, setCornerRadius: 12.0f64];
                            let _: () = msg_send![layer, setMasksToBounds: true];
                        }
                    }
                }
            }

            
            if let Err(e) = menu::setup_app_menu(&app.handle()) {
                eprintln!("Failed to setup app menu: {}", e);
            }

            
            if let Err(e) = tray::setup_tray(&app.handle()) {
                eprintln!("Failed to setup tray: {}", e);
            }

            
            let auth_state = app.state::<Arc<tauri::async_runtime::Mutex<AuthState>>>();
            let auth_state_clone = auth_state.inner().clone();
            
            
            tauri::async_runtime::block_on(async {
                match auth::load_auth_state().await {
                    Ok(Some(saved_auth_state)) => {
                        let mut current_auth_state = auth_state_clone.lock().await;
                        *current_auth_state = saved_auth_state;
                        push_log("info", "backend", "Loaded saved authentication state on startup".to_string());
                    }
                    Ok(None) => {
                        push_log("info", "backend", "No saved authentication state found".to_string());
                    }
                    Err(e) => {
                        push_log("error", "backend", format!("Failed to load saved authentication state: {}", e));
                    }
                }
            });

            
            let discord_rpc_state = app.state::<Arc<tauri::async_runtime::Mutex<DiscordRpcService>>>();
            let discord_rpc_clone = discord_rpc_state.inner().clone();
            
            tauri::async_runtime::spawn(async move {
                let mut rpc_service = discord_rpc_clone.lock().await;
                match rpc_service.auto_connect() {
                    Ok(_) => push_log("info", "backend", "Discord RPC auto-connected on startup".to_string()),
                    Err(e) => push_log("warn", "backend", format!("Discord RPC auto-connect failed (this is optional): {}", e)),
                }
            });
            
            
            #[cfg(any(target_os = "linux", all(debug_assertions, target_os = "windows")))]
            {
                app.deep_link().register_all().unwrap_or_else(|e| {
                push_log("error", "backend", format!("Failed to register deep links: {}", e));
                });
            }

            
            setup_deep_link_handlers(app);
            
            
            if let Some(window) = app.get_webview_window("main") {
                let window_handle = window.clone();
                let _ = window.on_window_event(move |event| {
                    match event {
                        WindowEvent::CloseRequested { api, .. } => {
                            push_log("info", "backend", "🪟 Window close requested - hiding to tray".to_string());
                            api.prevent_close();
                            let _ = window_handle.hide();
                            push_log("info", "backend", "✅ Window hidden to tray".to_string());
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

fn setup_deep_link_handlers(app: &mut tauri::App) {
    
    let app_handle = app.handle().clone();
    app.deep_link().on_open_url(move |event| {
        let urls = event.urls();
        push_log("info", "backend", format!("Deep link received: {:?}", urls));
        
        for url in urls {
            let url_string = url.to_string();
            if url_string.starts_with("hackatime://auth/callback") {
                handle_oauth_callback(&app_handle, &url_string);
            }
        }
    });

    
    if let Some(start_urls) = app.deep_link().get_current().unwrap_or_default() {
        push_log("info", "backend", format!("App started with deep link: {:?}", start_urls));
        let app_handle = app.handle().clone();
        for url in start_urls {
            let url_string = url.to_string();
            if url_string.starts_with("hackatime://auth/callback") {
                handle_oauth_callback(&app_handle, &url_string);
            }
        }
    }
}

fn handle_oauth_callback(app_handle: &tauri::AppHandle, url_string: &str) {
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
            push_log("error", "backend", format!("OAuth error: {}", error));
            return;
        }
        
        if let (Some(code), Some(state)) = (code, state) {
            push_log("info", "backend", format!("Extracted authorization code: {} and state: {}", code, state));
            
            let api_config = app_handle.state::<ApiConfig>();
            let auth_state = app_handle.state::<Arc<tauri::async_runtime::Mutex<AuthState>>>();
            let pkce_state = app_handle.state::<Arc<tauri::async_runtime::Mutex<Option<PkceState>>>>();
            
            let code_clone = code.clone();
            let state_clone = state.clone();
            let api_config_clone = api_config.inner().clone();
            let auth_state_clone = auth_state.inner().clone();
            let pkce_state_clone = pkce_state.inner().clone();
            
            tauri::async_runtime::spawn(async move {
                process_oauth_token_exchange(
                    code_clone,
                    state_clone,
                    api_config_clone,
                    auth_state_clone,
                    pkce_state_clone,
                ).await;
            });
        } else {
            push_log("warn", "backend", "Missing code or state parameter in OAuth callback".to_string());
        }
    }
}

async fn process_oauth_token_exchange(
    code: String,
    state: String,
    api_config: ApiConfig,
    auth_state: Arc<tauri::async_runtime::Mutex<AuthState>>,
    pkce_state: Arc<tauri::async_runtime::Mutex<Option<PkceState>>>,
) {
    let client = reqwest::Client::new();
    
    let stored_pkce = {
        let pkce_guard = pkce_state.lock().await;
        pkce_guard.clone()
    };

    let pkce = match stored_pkce {
        Some(pkce) => {
            if pkce.is_expired(600) {
                eprintln!("PKCE state expired. Please restart authentication.");
                return;
            }
            
            if pkce.state != state {
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
        .post(&format!("{}/oauth/token", api_config.base_url))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
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
                            .get(&format!("{}/api/v1/authenticated/me", api_config.base_url))
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

                        let mut auth_state = auth_state.lock().await;
                        auth_state.is_authenticated = true;
                        auth_state.access_token = Some(access_token.to_string());
                        auth_state.user_info = Some(user_info_map);

                        let auth_state_to_save = auth_state.clone();
                        drop(auth_state);
                        if let Err(e) = auth::save_auth_state(auth_state_to_save).await {
                            eprintln!("Failed to save auth state: {}", e);
                        }

                        {
                            let mut stored_pkce = pkce_state.lock().await;
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
}
