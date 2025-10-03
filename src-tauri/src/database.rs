use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, Row};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::env;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub access_token: Option<String>,
    pub user_info: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct SessionRecord {
    pub id: String,
    pub is_authenticated: bool,
    pub access_token: Option<String>,
    pub user_info: Option<String>, // JSON string
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
}

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self, String> {
        let db_path = get_hackatime_db_path()?;
        
        // Ensure the hackatime directory exists
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create hackatime directory: {}", e))?;
                println!("Created directory: {}", parent.display());
            }
        }
        
        // Ensure the parent directory exists and is writable
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                return Err(format!("Parent directory does not exist: {}", parent.display()));
            }
            
            // Test if we can write to the directory
            let test_file = parent.join(".write_test");
            if let Err(e) = fs::write(&test_file, "test") {
                return Err(format!("Cannot write to directory {}: {}", parent.display(), e));
            }
            // Clean up test file
            let _ = fs::remove_file(&test_file);
            
            println!("Directory is writable: {}", parent.display());
        }
        
        // Create the database file if it doesn't exist
        if !db_path.exists() {
            println!("Database file doesn't exist, creating: {}", db_path.display());
            // Touch the file to ensure it exists
            if let Err(e) = fs::write(&db_path, "") {
                return Err(format!("Cannot create database file {}: {}", db_path.display(), e));
            }
        } else {
            println!("Database file already exists: {}", db_path.display());
        }
        
        // Check file permissions
        if let Ok(metadata) = fs::metadata(&db_path) {
            println!("Database file metadata: {:?}", metadata);
        }
        
        // Try using SqlitePool::connect_with instead of connect
        let database_url = format!("sqlite:{}", db_path.display());
        println!("Connecting to database at: {}", database_url);
        
        // First try the standard connect method
        let pool_result = SqlitePool::connect(&database_url).await;
        
        let pool = match pool_result {
            Ok(pool) => pool,
            Err(e) => {
                println!("Standard connect failed: {}, trying connect_with", e);
                // Try with explicit options
                let options = sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(&db_path)
                    .create_if_missing(true);
                
                SqlitePool::connect_with(options)
                    .await
                    .map_err(|e| format!("Failed to connect to database at {} with connect_with: {}", db_path.display(), e))?
            }
        };
        
        let db = Database { pool };
        db.migrate().await?;
        
        println!("Database initialized successfully");
        Ok(db)
    }
    
    async fn migrate(&self) -> Result<(), String> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                is_authenticated INTEGER NOT NULL DEFAULT 0,
                access_token TEXT,
                user_info TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_accessed_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create sessions table: {}", e))?;
        
        Ok(())
    }
    
    pub async fn save_session(&self, auth_state: &AuthState) -> Result<String, String> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let user_info_json = match &auth_state.user_info {
            Some(info) => Some(serde_json::to_string(info)
                .map_err(|e| format!("Failed to serialize user info: {}", e))?),
            None => None,
        };
        
        sqlx::query(
            r#"
            INSERT INTO sessions (id, is_authenticated, access_token, user_info, created_at, updated_at, last_accessed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&session_id)
        .bind(auth_state.is_authenticated as i32)
        .bind(&auth_state.access_token)
        .bind(&user_info_json)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save session: {}", e))?;
        
        Ok(session_id)
    }
    
    #[allow(dead_code)]
    pub async fn update_session(&self, session_id: &str, auth_state: &AuthState) -> Result<(), String> {
        let now = Utc::now();
        
        let user_info_json = match &auth_state.user_info {
            Some(info) => Some(serde_json::to_string(info)
                .map_err(|e| format!("Failed to serialize user info: {}", e))?),
            None => None,
        };
        
        sqlx::query(
            r#"
            UPDATE sessions 
            SET is_authenticated = ?, access_token = ?, user_info = ?, updated_at = ?, last_accessed_at = ?
            WHERE id = ?
            "#,
        )
        .bind(auth_state.is_authenticated as i32)
        .bind(&auth_state.access_token)
        .bind(&user_info_json)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .bind(session_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update session: {}", e))?;
        
        Ok(())
    }
    
    pub async fn load_latest_session(&self) -> Result<Option<AuthState>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, is_authenticated, access_token, user_info, last_accessed_at
            FROM sessions 
            ORDER BY last_accessed_at DESC 
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to load latest session: {}", e))?;
        
        match row {
            Some(row) => {
                let session_id: String = row.get("id");
                let is_authenticated: i32 = row.get("is_authenticated");
                let access_token: Option<String> = row.get("access_token");
                let user_info_json: Option<String> = row.get("user_info");
                
                let user_info = match user_info_json {
                    Some(json) => {
                        match serde_json::from_str::<HashMap<String, serde_json::Value>>(&json) {
                            Ok(info) => Some(info),
                            Err(_) => None, // Skip invalid JSON
                        }
                    }
                    None => None,
                };
                
                // Update last_accessed_at
                self.update_last_accessed(&session_id).await?;
                
                Ok(Some(AuthState {
                    is_authenticated: is_authenticated != 0,
                    access_token,
                    user_info,
                }))
            }
            None => Ok(None),
        }
    }
    
    async fn update_last_accessed(&self, session_id: &str) -> Result<(), String> {
        let now = Utc::now();
        
        sqlx::query(
            "UPDATE sessions SET last_accessed_at = ? WHERE id = ?"
        )
        .bind(now.to_rfc3339())
        .bind(session_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update last accessed time: {}", e))?;
        
        Ok(())
    }
    
    pub async fn clear_sessions(&self) -> Result<(), String> {
        sqlx::query("DELETE FROM sessions")
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to clear sessions: {}", e))?;
        
        Ok(())
    }
    
    pub async fn cleanup_old_sessions(&self, days_old: i64) -> Result<(), String> {
        let cutoff = Utc::now() - chrono::Duration::days(days_old);
        
        sqlx::query("DELETE FROM sessions WHERE last_accessed_at < ?")
            .bind(cutoff.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to cleanup old sessions: {}", e))?;
        
        Ok(())
    }
}

fn get_hackatime_db_path() -> Result<std::path::PathBuf, String> {
    let app_data_dir = get_app_data_dir()?;
    let db_path = app_data_dir.join("sessions.db");
    
    println!("Database path: {}", db_path.display());
    println!("Parent directory exists: {}", db_path.parent().map_or(false, |p| p.exists()));
    
    Ok(db_path)
}

fn get_app_data_dir() -> Result<std::path::PathBuf, String> {
    if cfg!(target_os = "windows") {
        // Windows: %APPDATA%\.hackatime\
        let appdata = env::var("APPDATA").map_err(|_| "Failed to get APPDATA directory")?;
        Ok(Path::new(&appdata).join(".hackatime"))
    } else if cfg!(target_os = "macos") {
        // macOS: ~/Library/Application Support/.hackatime/
        let home = env::var("HOME").map_err(|_| "Failed to get HOME directory")?;
        Ok(Path::new(&home).join("Library").join("Application Support").join(".hackatime"))
    } else {
        // Linux: ~/.local/share/.hackatime/
        let home = env::var("HOME").map_err(|_| "Failed to get HOME directory")?;
        Ok(Path::new(&home).join(".local").join("share").join(".hackatime"))
    }
}

pub fn get_hackatime_config_dir() -> Result<std::path::PathBuf, String> {
    let app_data_dir = get_app_data_dir()?;
    
    // Create the directory if it doesn't exist
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create hackatime directory: {}", e))?;
    }
    
    Ok(app_data_dir)
}

pub fn get_hackatime_logs_dir() -> Result<std::path::PathBuf, String> {
    let config_dir = get_hackatime_config_dir()?;
    let logs_dir = config_dir.join("logs");
    
    // Create the logs directory if it doesn't exist
    if !logs_dir.exists() {
        fs::create_dir_all(&logs_dir)
            .map_err(|e| format!("Failed to create logs directory: {}", e))?;
    }
    
    Ok(logs_dir)
}

pub fn get_hackatime_data_dir() -> Result<std::path::PathBuf, String> {
    let config_dir = get_hackatime_config_dir()?;
    let data_dir = config_dir.join("data");
    
    // Create the data directory if it doesn't exist
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
    }
    
    Ok(data_dir)
}

pub fn get_platform_info() -> Result<serde_json::Value, String> {
    let app_data_dir = get_app_data_dir()?;
    
    let platform_info = serde_json::json!({
        "platform": std::env::consts::OS,
        "app_data_dir": app_data_dir.to_string_lossy(),
        "description": if cfg!(target_os = "windows") {
            "Windows AppData directory"
        } else if cfg!(target_os = "macos") {
            "macOS Application Support directory"
        } else {
            "Linux XDG data directory"
        }
    });
    
    Ok(platform_info)
}
