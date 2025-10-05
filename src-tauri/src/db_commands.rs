use crate::database::{get_hackatime_config_dir, get_hackatime_data_dir, get_hackatime_logs_dir, get_platform_info, Database};
use crate::push_log;

#[tauri::command]
pub async fn get_hackatime_directories() -> Result<serde_json::Value, String> {
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
pub async fn cleanup_old_sessions(days_old: i64) -> Result<(), String> {
    let db = Database::new().await?;
    db.cleanup_old_sessions(days_old).await?;
    Ok(())
}

#[tauri::command]
pub async fn clear_statistics_cache() -> Result<(), String> {
    push_log("info", "backend", "Clearing statistics cache...".to_string());
    let db = Database::new().await?;
    
    
    db.cleanup_expired_cache().await?;
    
    
    db.clear_all_cache().await?;
    
    push_log("info", "backend", "Statistics cache cleared successfully".to_string());
    Ok(())
}

#[tauri::command]
pub async fn get_session_stats() -> Result<serde_json::Value, String> {
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
pub async fn test_database_connection() -> Result<serde_json::Value, String> {
    
    let config_dir = get_hackatime_config_dir()?;
    let logs_dir = get_hackatime_logs_dir()?;
    let data_dir = get_hackatime_data_dir()?;

    
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

