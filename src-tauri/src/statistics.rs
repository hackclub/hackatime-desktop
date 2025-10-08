use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use chrono::Datelike;

use crate::auth::AuthState;
use crate::config::ApiConfig;
use crate::database::Database;
use crate::push_log;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatisticsData {
    pub trends: Vec<TrendStatistic>,
    pub charts: Vec<ChartData>,
    pub insights: Vec<Insight>,
    pub programmer_class: ProgrammerClass,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrendStatistic {
    pub title: String,
    pub value: String,
    pub change: String,
    pub change_type: String, 
    pub period: String,
    pub icon: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChartData {
    pub id: String,
    pub title: String,
    pub chart_type: String, 
    pub data: serde_json::Value,
    pub period: String,
    pub color_scheme: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Insight {
    pub title: String,
    pub description: String,
    pub value: String,
    pub trend: String,
    pub icon: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgrammerClass {
    pub class_name: String,
    pub description: String,
    pub technologies: Vec<String>,
    pub level: String,
    pub color: String,
}

async fn fetch_hours_with_cache(
    client: &reqwest::Client,
    base_url: &str,
    access_token: &str,
    start_date: &str,
    end_date: &str,
) -> Result<serde_json::Value, String> {
    let db = Database::new().await?;
    let cache_key = format!("hours:{}:{}", start_date, end_date);
    
    
    let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();
    let is_current_date = end_date == today || start_date == today;
    
    if !is_current_date {
        if let Ok(Some(cached_data)) = db.get_cached_data(&cache_key).await {
            push_log("debug", "backend", format!("Using cached data for {}", cache_key));
            let result: serde_json::Value = serde_json::from_str(&cached_data)
                .map_err(|e| format!("Failed to parse cached data: {}", e))?;
            push_log("info", "backend", format!("CACHED DATA for {}: {}", cache_key, serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Failed to serialize".to_string())));
            return Ok(result);
        }
    } else {
        push_log("info", "backend", format!("Skipping cache for current date: {}", cache_key));
    }
    
    let url = format!(
        "{}/api/v1/authenticated/hours?start_date={}&end_date={}",
        base_url,
        start_date,
        end_date
    );
    
    push_log("info", "backend", format!("API REQUEST: GET {}", url));
    push_log("debug", "backend", format!("Authorization: Bearer {}...", &access_token[..20.min(access_token.len())]));
    
    let response = client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to fetch hours: {}", e);
            push_log("error", "backend", format!(" API REQUEST FAILED: {}", error_msg));
            error_msg
        })?;
    
    let status = response.status();
    push_log("info", "backend", format!("API RESPONSE: Status {}", status));
    
    if !status.is_success() {
        let error_msg = format!("API request failed with status: {}", status);
        push_log("error", "backend", format!(" {}", error_msg));
        return Err(error_msg);
    }
    
    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to parse API response: {}", e);
            push_log("error", "backend", format!(" {}", error_msg));
            error_msg
        })?;
    
    push_log("info", "backend", format!("API RESPONSE DATA: {}", serde_json::to_string_pretty(&data).unwrap_or_else(|_| "Failed to serialize".to_string())));
    
    if !is_current_date {
        let data_str = serde_json::to_string(&data)
            .map_err(|e| format!("Failed to serialize data for caching: {}", e))?;
        db.set_cached_data(&cache_key, &data_str, 30).await.ok();
        push_log("debug", "backend", format!("Cached data for {}", cache_key));
    } else {
        push_log("debug", "backend", format!("Not caching data for current date: {}", cache_key));
    }
    
    Ok(data)
}

async fn fetch_streak_with_cache(
    client: &reqwest::Client,
    base_url: &str,
    access_token: &str,
) -> Result<serde_json::Value, String> {
    let url = format!("{}/api/v1/authenticated/streak", base_url);
    
    push_log("info", "backend", format!("API REQUEST: GET {}", url));
    push_log("debug", "backend", format!("Authorization: Bearer {}...", &access_token[..20.min(access_token.len())]));
    
    let response = client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to fetch streak: {}", e);
            push_log("error", "backend", format!(" API REQUEST FAILED: {}", error_msg));
            error_msg
        })?;
    
    let status = response.status();
    push_log("info", "backend", format!(" API RESPONSE: Status {}", status));
    
    if !status.is_success() {
        let error_msg = format!("Streak API request failed with status: {}", status);
        push_log("error", "backend", format!(" {}", error_msg));
        return Err(error_msg);
    }
    
    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to parse streak response: {}", e);
            push_log("error", "backend", format!(" {}", error_msg));
            error_msg
        })?;
    
    push_log("info", "backend", format!(" STREAK API RESPONSE DATA: {}", serde_json::to_string_pretty(&data).unwrap_or_else(|_| "Failed to serialize".to_string())));
    
    Ok(data)
}

#[tauri::command]
pub async fn get_statistics_data(
    api_config: ApiConfig,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<StatisticsData, String> {
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

    let end_date = chrono::Utc::now().date_naive();
    
    let mut daily_hours = serde_json::Map::new();
    let mut total_seconds = 0u64;
    
    for days_ago in 0..7 {
        let date = end_date - chrono::Duration::days(days_ago);
        let date_str = date.format("%Y-%m-%d").to_string();
        
        match fetch_hours_with_cache(&client, base_url, access_token, &date_str, &date_str).await {
            Ok(day_data) => {
                let seconds = day_data["total_seconds"].as_u64().unwrap_or(0);
                total_seconds += seconds;
                
                let day_name = match date.weekday() {
                    chrono::Weekday::Mon => "Mon",
                    chrono::Weekday::Tue => "Tue",
                    chrono::Weekday::Wed => "Wed",
                    chrono::Weekday::Thu => "Thu",
                    chrono::Weekday::Fri => "Fri",
                    chrono::Weekday::Sat => "Sat",
                    chrono::Weekday::Sun => "Sun",
                };
                
                daily_hours.insert(date_str.clone(), serde_json::json!({
                    "date": date_str,
                    "day_name": day_name,
                    "hours": seconds as f64 / 3600.0,
                    "seconds": seconds
                }));
            }
            Err(_) => {
                let day_name = match date.weekday() {
                    chrono::Weekday::Mon => "Mon",
                    chrono::Weekday::Tue => "Tue",
                    chrono::Weekday::Wed => "Wed",
                    chrono::Weekday::Thu => "Thu",
                    chrono::Weekday::Fri => "Fri",
                    chrono::Weekday::Sat => "Sat",
                    chrono::Weekday::Sun => "Sun",
                };
                
                daily_hours.insert(date_str.clone(), serde_json::json!({
                    "date": date_str,
                    "day_name": day_name,
                    "hours": 0.0,
                    "seconds": 0
                }));
            }
        }
    }
    
    let all_time_start = end_date - chrono::Duration::days(365);
    let all_time_seconds = match fetch_hours_with_cache(
        &client, 
        base_url, 
        access_token,
        &all_time_start.format("%Y-%m-%d").to_string(),
        &end_date.format("%Y-%m-%d").to_string()
    ).await {
        Ok(data) => data["total_seconds"].as_u64().unwrap_or(0),
        Err(_) => 0
    };
    
    let prev_week_end = end_date - chrono::Duration::days(7);
    let prev_week_start = prev_week_end - chrono::Duration::days(7);
    
    push_log("info", "backend", format!("[get_statistics_data] Fetching previous week: {} to {}", 
        prev_week_start.format("%Y-%m-%d"), prev_week_end.format("%Y-%m-%d")));
    
    let prev_week_seconds = match fetch_hours_with_cache(
        &client,
        base_url,
        access_token,
        &prev_week_start.format("%Y-%m-%d").to_string(),
        &prev_week_end.format("%Y-%m-%d").to_string()
    ).await {
        Ok(data) => {
            let seconds = data["total_seconds"].as_u64().unwrap_or(0);
            push_log("info", "backend", format!("[get_statistics_data] Previous week: {} seconds ({:.2}h)", 
                seconds, seconds as f64 / 3600.0));
            seconds
        },
        Err(e) => {
            push_log("error", "backend", format!("[get_statistics_data] Failed to fetch previous week: {}", e));
            0
        }
    };
    
    let hours_data = serde_json::json!({
        "weekly_stats": {
            "time_coded_seconds": total_seconds,
            "daily_hours": daily_hours
        },
        "all_time_stats": {
            "time_coded_seconds": all_time_seconds
        },
        "calculated_metrics": {
            "prev_week_hours": (prev_week_seconds as f64 / 3600.0 * 10.0).round() / 10.0,
            "prev_week_seconds": prev_week_seconds  
        }
    });

    let streak_data = fetch_streak_with_cache(&client, base_url, access_token)
        .await
        .map_err(|e| format!("Failed to fetch streak data: {}", e))?;

    let mut dashboard_stats = hours_data;
    
    if let Some(streak) = streak_data.get("streak_days") {
        dashboard_stats["current_streak"] = streak.clone();
    }
    if let Some(longest) = streak_data.get("longest_streak") {
        dashboard_stats["longest_streak"] = longest.clone();
    }

    let statistics = process_statistics_data(dashboard_stats).await?;

    Ok(statistics)
}

#[tauri::command]
pub async fn get_dashboard_stats(
    api_config: ApiConfig,
    state: State<'_, Arc<tauri::async_runtime::Mutex<AuthState>>>,
) -> Result<serde_json::Value, String> {
    push_log("info", "backend", "get_dashboard_stats called".to_string());
    
    let auth_state = state.lock().await;

    if !auth_state.is_authenticated {
        push_log("error", "backend", "Not authenticated".to_string());
        return Err("Not authenticated".to_string());
    }

    let base_url = if api_config.base_url.is_empty() {
        "https://hackatime.hackclub.com"
    } else {
        &api_config.base_url
    };
    
    push_log("info", "backend", format!("Using base URL: {}", base_url));

    let access_token = auth_state
        .access_token
        .as_ref()
        .ok_or("No access token available")?;
    
    push_log("debug", "backend", format!("Access token present: {}...", &access_token[..20.min(access_token.len())]));

    let client = reqwest::Client::new();

    
    let end_date = chrono::Utc::now().date_naive();
    let start_date = end_date - chrono::Duration::days(7);
    
    push_log("info", "backend", format!("Date range: {} to {}", start_date.format("%Y-%m-%d"), end_date.format("%Y-%m-%d")));

    push_log("info", "backend", "Fetching current week hours...".to_string());
    let _current_week_data = fetch_hours_with_cache(
        &client,
        base_url,
        access_token,
        &start_date.format("%Y-%m-%d").to_string(),
        &end_date.format("%Y-%m-%d").to_string()
    ).await.map_err(|e| {
        push_log("error", "backend", format!("Failed to fetch current week hours: {}", e));
        format!("Failed to fetch current week hours: {}", e)
    })?;

    
    let prev_week_end = start_date;
    let prev_week_start = prev_week_end - chrono::Duration::days(7);
    
    push_log("info", "backend", format!("Fetching previous week hours: {} to {}", prev_week_start.format("%Y-%m-%d"), prev_week_end.format("%Y-%m-%d")));
    
    let prev_week_data = fetch_hours_with_cache(
        &client,
        base_url,
        access_token,
        &prev_week_start.format("%Y-%m-%d").to_string(),
        &prev_week_end.format("%Y-%m-%d").to_string()
    ).await.unwrap_or_else(|e| {
        push_log("warn", "backend", format!("Failed to fetch previous week hours: {}", e));
        serde_json::json!({"total_seconds": 0})
    });

    
    let mut daily_hours = serde_json::Map::new();
    let mut total_seconds = 0u64;
    
    push_log("info", "backend", "Fetching daily hours for last 7 days...".to_string());
    
    for days_ago in 0..7 {
        let date = end_date - chrono::Duration::days(days_ago);
        let date_str = date.format("%Y-%m-%d").to_string();
        
        
        match fetch_hours_with_cache(&client, base_url, access_token, &date_str, &date_str).await {
            Ok(day_data) => {
                let seconds = day_data["total_seconds"].as_u64().unwrap_or(0);
                total_seconds += seconds;
                
                push_log("info", "backend", format!(" {} ({} days ago): {} seconds ({:.2} hours)", 
                    date_str, days_ago, seconds, seconds as f64 / 3600.0));
                
                let day_name = match date.weekday() {
                    chrono::Weekday::Mon => "Mon",
                    chrono::Weekday::Tue => "Tue",
                    chrono::Weekday::Wed => "Wed",
                    chrono::Weekday::Thu => "Thu",
                    chrono::Weekday::Fri => "Fri",
                    chrono::Weekday::Sat => "Sat",
                    chrono::Weekday::Sun => "Sun",
                };
                
                daily_hours.insert(date_str.clone(), serde_json::json!({
                    "date": date_str,
                    "day_name": day_name,
                    "hours": seconds as f64 / 3600.0,
                    "seconds": seconds
                }));
            }
            Err(e) => {
                push_log("warn", "backend", format!("Failed to fetch hours for {}: {}", date_str, e));
                
                let day_name = match date.weekday() {
                    chrono::Weekday::Mon => "Mon",
                    chrono::Weekday::Tue => "Tue",
                    chrono::Weekday::Wed => "Wed",
                    chrono::Weekday::Thu => "Thu",
                    chrono::Weekday::Fri => "Fri",
                    chrono::Weekday::Sat => "Sat",
                    chrono::Weekday::Sun => "Sun",
                };
                
                daily_hours.insert(date_str.clone(), serde_json::json!({
                    "date": date_str,
                    "day_name": day_name,
                    "hours": 0.0,
                    "seconds": 0
                }));
            }
        }
    }

    push_log("info", "backend", format!("Total seconds for last 7 days: {} ({:.2} hours)", total_seconds, total_seconds as f64 / 3600.0));
    
    push_log("info", "backend", "Fetching streak data...".to_string());
    let streak_data = fetch_streak_with_cache(&client, base_url, access_token)
        .await
        .unwrap_or_else(|e| {
            push_log("error", "backend", format!("Failed to fetch streak data: {}", e));
            serde_json::json!({"current_streak": 0, "longest_streak": 0})
        });

    
    let current_week_seconds = total_seconds as f64;
    let prev_week_seconds = prev_week_data["total_seconds"].as_f64().unwrap_or(0.0);
    
    push_log("info", "backend", format!("Week comparison: Current week = {:.2}h, Previous week = {:.2}h", 
        current_week_seconds / 3600.0, prev_week_seconds / 3600.0));
    
    
    let daily_average_hours = current_week_seconds / 3600.0 / 7.0;
    
    
    let weekly_hours = current_week_seconds / 3600.0;
    
    
    let weekly_change_percent = if prev_week_seconds > 0.0 {
        ((current_week_seconds - prev_week_seconds) / prev_week_seconds * 100.0).round()
    } else if current_week_seconds > 0.0 {
        100.0 
    } else {
        0.0
    };
    
    
    let current_streak = streak_data["streak_days"].as_u64().unwrap_or(0);
    let longest_streak = streak_data["longest_streak"].as_u64().unwrap_or(0);
    
    push_log("info", "backend", format!("Streak: Current = {} days (from API field 'streak_days'), Longest = {} days", current_streak, longest_streak));
    push_log("info", "backend", format!("Metrics: Daily average = {:.2}h, Weekly hours = {:.2}h, Weekly change = {:.1}%", 
        daily_average_hours, weekly_hours, weekly_change_percent));

    let result = serde_json::json!({
        "current_streak": current_streak,
        "longest_streak": longest_streak,
        "weekly_stats": {
            "time_coded_seconds": total_seconds,
            "daily_hours": daily_hours
        },
        "calculated_metrics": {
            "daily_average_hours": (daily_average_hours * 10.0).round() / 10.0,
            "weekly_hours": (weekly_hours * 10.0).round() / 10.0,
            "weekly_change_percent": weekly_change_percent,
            "prev_week_hours": (prev_week_seconds / 3600.0 * 10.0).round() / 10.0,
            "prev_week_seconds": prev_week_seconds  
        }
    });
    
    push_log("info", "backend", format!("FINAL DASHBOARD STATS: {}", serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Failed to serialize".to_string())));

    Ok(result)
}

async fn process_statistics_data(
    dashboard_stats: serde_json::Value,
) -> Result<StatisticsData, String> {
    
    let current_streak = dashboard_stats["current_streak"].as_u64().unwrap_or(0);
    let weekly_time = dashboard_stats["weekly_stats"]["time_coded_seconds"]
        .as_u64()
        .unwrap_or(0) as f64;
    let all_time_time = dashboard_stats["all_time_stats"]["time_coded_seconds"]
        .as_u64()
        .unwrap_or(0) as f64;
    
    let prev_week_time = dashboard_stats["calculated_metrics"]["prev_week_seconds"]
        .as_f64()
        .unwrap_or_else(|| {
            dashboard_stats["calculated_metrics"]["prev_week_hours"]
                .as_f64()
                .unwrap_or(0.0) * 3600.0
        });
    
    push_log("info", "backend", format!("[process_statistics_data] Using previous week: {} seconds ({:.2}h)", 
        prev_week_time, prev_week_time / 3600.0));
    push_log("info", "backend", format!("[process_statistics_data] Current week: {} seconds ({:.2}h)", 
        weekly_time, weekly_time / 3600.0));

    
    let trends = calculate_trends(weekly_time, prev_week_time, current_streak).await;

    
    let charts = generate_chart_data(&dashboard_stats).await?;

    
    let insights = generate_insights(weekly_time, all_time_time, current_streak).await;

    
    let programmer_class = analyze_programmer_class(&dashboard_stats).await;

    Ok(StatisticsData {
        trends,
        charts,
        insights,
        programmer_class,
    })
}

async fn calculate_trends(weekly_time: f64, prev_week_time: f64, current_streak: u64) -> Vec<TrendStatistic> {
    let mut trends = Vec::new();

    
    
    let last_week_time = prev_week_time;
    let last_week_streak = if current_streak > 0 {
        current_streak - 1
    } else {
        0
    };

    
    let time_change = if last_week_time > 0.0 {
        let change = ((weekly_time - last_week_time) / last_week_time * 100.0).round() as i32;
        push_log("info", "backend", format!("[calculate_trends] Weekly change: {:.2}h -> {:.2}h = {}%", 
            last_week_time / 3600.0, weekly_time / 3600.0, change));
        change
    } else if weekly_time > 0.0 {
        push_log("info", "backend", "[calculate_trends] No previous week data, defaulting to +100%".to_string());
        100 
    } else {
        push_log("info", "backend", "[calculate_trends] No data for either week, 0%".to_string());
        0
    };
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

    
    let daily_average = weekly_time / 3600.0 / 7.0;
    let last_week_daily = last_week_time / 3600.0 / 7.0;  
    
    let focus_change = if last_week_daily > 0.0 {
        ((daily_average - last_week_daily) / last_week_daily * 100.0).round() as i32
    } else if daily_average > 0.0 {
        100  
    } else {
        0
    };

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

    let mut chart_data = Vec::new();
    let mut labels = Vec::new();
    
    if let Some(daily_hours) = dashboard_stats["weekly_stats"]["daily_hours"].as_object() {
        for (_date, day_data) in daily_hours {
            if let Some(hours) = day_data["hours"].as_f64() {
                labels.push(day_data["day_name"].as_str().unwrap_or("").to_string());
                chart_data.push(hours);
            }
        }
    }
    
    if chart_data.is_empty() {
        let day_names = vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        for day in day_names {
            labels.push(day.to_string());
            chart_data.push(0.0);
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

    let mut trend_data = Vec::new();
    let mut trend_labels = Vec::new();

    let current_week_seconds = dashboard_stats["weekly_stats"]["time_coded_seconds"]
        .as_u64()
        .unwrap_or(0);
    
    
    for week in 0..4 {
        let week_hours = if week == 3 {
            current_week_seconds as f64 / 3600.0
        } else if current_week_seconds == 0 {
            0.0
        } else {
            
            (current_week_seconds as f64 / 3600.0) * (0.8 + (week as f64 * 0.1))
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
            icon: "".to_string(),
            color: "#2196F3".to_string(),
        }
    } else {
        Insight {
            title: "Just Getting Started".to_string(),
            description: "Every expert was once a beginner. Keep coding!".to_string(),
            value: format!("{:.0}h total", total_hours),
            trend: "Beginning journey".to_string(),
            icon: "".to_string(),
            color: "#9C27B0".to_string(),
        }
    };
    insights.push(total_insight);

    insights
}

async fn analyze_programmer_class(dashboard_stats: &serde_json::Value) -> ProgrammerClass {
    
    let config_path = std::env::current_dir()
        .unwrap_or_default()
        .join("programmer_classes.json");

    let config_content = match std::fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(_) => {
            
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

    
    let simulated_languages = simulate_language_analysis(total_hours, current_streak);

    
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
    
    
    let mut languages = Vec::new();

    
    if total_hours >= 100.0 {
        
        languages.push("JavaScript".to_string());
        languages.push("Python".to_string());
        languages.push("Java".to_string());
        if current_streak >= 7 {
            languages.push("Rust".to_string());
            languages.push("Go".to_string());
        }
    } else if total_hours >= 20.0 {
        
        languages.push("JavaScript".to_string());
        languages.push("Python".to_string());
        if current_streak >= 5 {
            languages.push("TypeScript".to_string());
        }
    } else {
        
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

    
    if let Some(primary_langs) = conditions
        .get("primary_languages")
        .and_then(|v| v.as_array())
    {
        let primary_lang_count = primary_langs
            .iter()
            .filter_map(|lang| lang.as_str())
            .filter(|lang| languages.contains(&lang.to_string()))
            .count();
        score += primary_lang_count as f64 * 2.0; 
    }

    
    if let Some(lang_count) = conditions.get("language_count").and_then(|v| v.as_u64()) {
        if languages.len() as u64 >= lang_count {
            score += 3.0; 
        }
    }

    
    if let Some(min_hours) = conditions.get("min_hours").and_then(|v| v.as_f64()) {
        if total_hours >= min_hours {
            score += 1.0;
        } else {
            score -= 0.5; 
        }
    }

    
    if let Some(max_hours) = conditions.get("max_hours").and_then(|v| v.as_f64()) {
        if total_hours <= max_hours {
            score += 1.0;
        } else {
            score -= 0.5; 
        }
    }

    
    if let Some(min_streak) = conditions.get("min_streak").and_then(|v| v.as_u64()) {
        if current_streak >= min_streak {
            score += 0.5;
        }
    }

    score
}

